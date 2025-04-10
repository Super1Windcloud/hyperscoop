use chrono::{DateTime, Local};
use crossterm::style::Stylize;
use crossterm::terminal;
use dashmap::DashSet;
use rayon::prelude::*;
use serde_json::Value;
use std::fs;
use std::io;

pub fn display_app_info(app_name: String, bucket_paths: Vec<String>) {
    let app_name = app_name.trim().to_lowercase();
    if let Some((bucket, name)) = app_name.split_once('/') {
        log::info!("name: {}", name);
        log::info!("bucket: {}", bucket);
        display_specific_app_info(name, bucket, bucket_paths);
        return;
    }
    let infos_set = DashSet::new();

    bucket_paths.par_iter().for_each(|bucket_path| {
        let manifest_path = format!("{}\\bucket", bucket_path);
        // log::info!("manifest_path: {}", manifest_path);

        if let Ok(entries) = fs::read_dir(&manifest_path) {
            entries.par_bridge().for_each(|entry| {
                if let Ok(file) = entry {
                    let file_type = file.file_type().unwrap();
                    let file_path = file.path();
                    if file_type.is_file()
                        && file_path.extension().map_or(false, |ext| ext == "json")
                    {
                        if let Some(file_name) = file_path.file_stem().and_then(|s| s.to_str()) {
                            if file_name == app_name {
                                let result =
                                    process_manifest_file(&file_path, &bucket_path, &app_name);
                                match result {
                                    Ok(info) => {
                                        infos_set.insert(info);
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "Failed to process file {}: {}",
                                            file_path.display(),
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    });
    print_pretty_info(infos_set);
}

fn display_specific_app_info(app_name: &str, bucket_name: &str, bucket_paths: Vec<String>) {
    if bucket_paths.is_empty() || bucket_name.is_empty() {
        println!("No bucket found.");
        return;
    }
    let info_set = DashSet::new();
    bucket_paths.par_iter().for_each(|bucket_path| {
        if let Some(bucket) = bucket_path.split('\\').nth(3) {
            if bucket == bucket_name {
                let bucket_path = format!("{}\\bucket", bucket_path);
                if let Ok(entries) = fs::read_dir(&bucket_path) {
                    entries.par_bridge().for_each(|entry| {
                        if let Ok(file) = entry {
                            let file_type = file.file_type().unwrap();
                            let file_path = file.path();
                            if file_type.is_file()
                                && file_path.extension().map_or(false, |ext| ext == "json")
                            {
                                if let Some(file_name) =
                                    file_path.file_stem().and_then(|s| s.to_str())
                                {
                                    if file_name == app_name {
                                        let result = process_manifest_file(
                                            &file_path,
                                            &bucket_path,
                                            app_name,
                                        );
                                        match result {
                                            Ok(info) => {
                                                info_set.insert(info);
                                                return;
                                            }
                                            Err(e) => {
                                                log::error!(
                                                    "Failed to process file {}: {}",
                                                    file_path.display(),
                                                    e
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            }
        }
    });
    print_pretty_info(info_set);
}

trait DisplayInfo {
    fn display_info(&self) -> Vec<String>;
}
impl DisplayInfo for &str {
    fn display_info(&self) -> Vec<String> {
        vec![self.to_string()]
    }
}
impl DisplayInfo for &Vec<Value> {
    fn display_info(&self) -> Vec<String> {
        let mut result = Vec::new();
        for (_, value) in self.iter().enumerate() {
            let value = value.to_string();
            result.push(value);
        }
        result
    }
}

impl DisplayInfo for Vec<Value> {
    fn display_info(&self) -> Vec<String> {
        let mut result = Vec::new();
        for (_, value) in self.iter().enumerate() {
            let value = value.to_string();
            result.push(value);
        }
        result
    }
}
impl DisplayInfo for &[Value] {
    fn display_info(&self) -> Vec<String> {
        let mut result = Vec::new();
        for value in *self {
            let value = value.to_string();
            result.push(value);
        }
        result
    }
}

fn process_manifest_file(
    file_path: &std::path::Path,
    manifest_path: &str,
    app_name: &str,
) -> anyhow::Result<Vec<(String, String)>> {
    let content = fs::read_to_string(file_path)?;
    let serde_obj: Value = serde_json::from_str(&content)?;

    let description = serde_obj["description"].as_str().unwrap_or_default();
    let version = serde_obj["version"].as_str().unwrap_or_default();
    let bucket_name = manifest_path
        .split('\\')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .nth(2)
        .unwrap_or("");
    let website = serde_obj["homepage"].as_str().unwrap_or_default();
    let license = serde_obj["license"].as_str().unwrap_or_default();
    let update_at = get_file_modified_time(file_path.to_str().unwrap_or(""))?;
    let binary = serde_obj["bin"].as_str().unwrap_or_default();
    let binary: Box<dyn DisplayInfo> = if binary.is_empty() {
        match serde_obj["bin"] {
            Value::Array(ref arr) => Box::new(arr.as_slice()) as Box<dyn DisplayInfo>,
            _ => Box::new(vec![Value::Null]) as Box<dyn DisplayInfo>,
        }
    } else {
        Box::new(binary) as Box<dyn DisplayInfo>
    };
    let short_str = serde_obj["shortcuts"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|shortcut| shortcut.as_array())
                .map(|inner| {
                    inner
                        .iter()
                        .map(|val| val.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                })
                .collect::<Vec<String>>()
                .join(" | ")
        })
        .unwrap_or_default();

    let notes = serde_obj["notes"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<&str>>()
                .join("\n\t\t")
        })
        .unwrap_or_default();

    let info = vec![
        ("Name\t".to_string(), app_name.to_string()),
        ("Description".to_string(), description.to_string()),
        ("Version\t".to_string(), version.to_string()),
        ("Bucket\t".to_string(), bucket_name.to_string()),
        ("Website\t".to_string(), website.to_string()),
        ("License\t".to_string(), license.to_string()),
        ("UpdateAt".to_string(), update_at.to_string()),
        ("Binary\t".to_string(), binary.display_info().join(", ")),
        ("Shortcuts".to_string(), short_str.to_string()),
        ("Notes\t".to_string(), notes.to_string()),
    ];
    Ok(info)
}

fn print_pretty_info(info: DashSet<Vec<(String, String)>>) {
    let max_key_length = info
        .iter()
        .flat_map(|vec| vec.iter().map(|(key, _)| key.len()).collect::<Vec<_>>())
        .max()
        .unwrap_or(0);
    let all_keys_values_width = info
        .iter()
        .flat_map(|vec| {
            vec.iter()
                .filter(|(key, _)| key.trim() != "Notes")
                .map(|(_, value)| value.len())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<usize>>();
    let max_value_length = info
        .iter()
        .flat_map(|vec| {
            vec.iter()
                .map(|(keys, value)| {
                    if keys.trim() == "Notes" {
                        value.lines().map(|line| line.len()).max().unwrap_or(0)
                    } else {
                        value.len()
                    }
                })
                .collect::<Vec<_>>()
        })
        .max()
        .unwrap_or(0);
    let mut i = 0;
    let terminal_max_width = terminal::size()
        .map(|(width, _)| width as usize)
        .unwrap_or(80);
    let all_width = max_value_length + max_key_length + 6;
    for vec in info {
        for (key, value) in vec {
            if !value.is_empty() {
                if key.trim() == "Notes" {
                    let lines: Vec<&str> = value.split('\n').collect(); /* 包含的\n 会产生空字符串 */
                    let lines = lines.iter().map(|line| line.trim()).collect::<Vec<&str>>();
                    let lines_width = lines.iter().map(|line| line.len()).collect::<Vec<usize>>();

                    let len = lines.len();
                    if all_width > terminal_max_width {
                        let lines = lines
                            .iter()
                            .filter(|line| !line.is_empty())
                            .collect::<Vec<_>>(); // true的保留
                        let merge_notes = lines
                            .iter()
                            .map(|line| line.to_string())
                            .collect::<Vec<String>>()
                            .join(" ");
                        let align_width = terminal_max_width - max_key_length - 6;
                        // println!(
                        //  "{}",
                        //  "-".repeat(align_width).dark_magenta().bold(),
                        // );

                        let align_text = textwrap::wrap(&merge_notes, align_width)
                            .iter()
                            .map(|item| item.to_string())
                            .collect::<Vec<String>>();
                        let len = align_text.len();
                        let lines_width = align_text
                            .iter()
                            .map(|line| line.len())
                            .collect::<Vec<usize>>();
                        println!(
                            "{:<width$}\t{:<width$}",
                            key.dark_green().bold(),
                            ": ".to_string() + &align_text[0],
                            width = lines_width[0] + 1
                        );
                        for i in 1..len {
                            println!(
                                "{}   {:<width$}",
                                " ".repeat(max_key_length + 4),
                                align_text[i],
                                width = lines_width[i] + 1
                            );
                        }
                        continue;
                    }
                    println!(
                        "{:<width$}\t{:<width$}",
                        key.dark_green().bold(),
                        ": ".to_string() + lines[0],
                        width = lines_width[0] + 1
                    );
                    for i in 1..len {
                        println!(
                            "{}   {:<width$}",
                            " ".repeat(max_key_length + 4),
                            lines[i],
                            width = lines_width[i] + 1
                        );
                    }
                    continue;
                }
                println!(
                    "{}\t{:<width2$}",
                    key.dark_green().bold(),
                    ": ".to_string() + &value,
                    width2 = all_keys_values_width[i]
                );
                i += 1;
            }
        }
        if all_width < terminal_max_width {
            println!("{}", "-".repeat(all_width).dark_magenta().bold(),);
        } else {
            println!("{}", "-".repeat(terminal_max_width).dark_magenta().bold());
        }
    }
}

fn get_file_modified_time(file_path: &str) -> io::Result<String> {
    let metadata = fs::metadata(file_path)?;
    let time = metadata.modified()?;
    let datetime: DateTime<Local> = time.into();
    Ok(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
}

mod test_align {

    #[test]
    fn test_align_terminal_cutoff() {
        let text = "textwrap: a small library for wrapping text.";
        let text = textwrap::wrap(text, 18);
        println!("{:?}", text);
    }
}
