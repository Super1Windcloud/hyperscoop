use crate::utils::system::get_system_default_arch;
use anyhow::{bail, Context};
use chrono::{DateTime, Local};
use crossterm::style::Stylize;
use crossterm::terminal;
use dashmap::DashSet;
use rayon::prelude::*;
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// 宏定义：简化键值对格式化输出
macro_rules! format_key_value {
    ($key:expr, $value:expr,  $max_key_length:expr, $terminal_max_width:expr) => {
        if let Err(e) = format_and_print($key, $value, $max_key_length, $terminal_max_width) {
            eprintln!("Error formatting {}: {}", $key, e);
        }
    };
}


pub fn display_app_info(app_name: String, bucket_paths: Vec<String>) -> anyhow::Result<()> {
    validate_app_name(&app_name)?;
    if let Some((bucket, name)) = app_name.split_once('/') {
        log::info!("name: {}", name);
        log::info!("bucket: {}", bucket);
        display_specific_bucket_app_info(name, bucket, bucket_paths)?;
        return Ok(());
    }
    let infos_set = DashSet::new();

    let result = bucket_paths.par_iter().try_for_each(|bucket_path| {
        let manifest_path = format!("{}\\bucket", bucket_path);
        if !Path::new(&manifest_path).exists() {
            bail!("Bucket dir {} not exists", bucket_path);
        }
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
        Ok(())
    });

    if let Err(e) = result {
        bail!(e.to_string());
    }
    print_pretty_info(infos_set);
    Ok(())
}

pub  fn validate_app_name(app_name: &str) -> anyhow::Result<()> {
    let re = Regex::new(r"^[a-zA-Z0-9]+([_\-/][a-zA-Z0-9]*)*$")?;
    if !re.is_match(app_name) {
        bail!("Invalid app name: {}", app_name);
    }
    let count = app_name.split('/').count();
    if count > 2 {
        bail!("Invalid app name: {}", app_name);
    }
    Ok(())
}

fn display_specific_bucket_app_info(
    app_name: &str,
    bucket_name: &str,
    bucket_paths: Vec<String>,
) -> anyhow::Result<()> {
    if bucket_paths.is_empty() || bucket_name.is_empty() {
        println!("No bucket found.");
        return Ok(());
    }
    let info_set = DashSet::new();
    let special_bucket_path = bucket_paths.iter().find(|path| path.contains(bucket_name));

    if special_bucket_path.is_none() {
        bail!("Bucket '{}' dir  not exists", bucket_name);
    }
    let result = bucket_paths.par_iter().try_for_each(|bucket_path| {
        if let Some(bucket) = Path::new(bucket_path).file_name().and_then(|s| s.to_str()) {
            if bucket == bucket_name {
                let bucket_path = format!("{}\\bucket", bucket_path);
                if !Path::new(&bucket_path).exists() {
                    bail!("Bucket dir {} not exists", bucket_path);
                }
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
        Ok(())
    });
    if let Err(e) = result {
        bail!(e.to_string());
    }

    print_pretty_info(info_set);
    Ok(())
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
    file_path: &Path,
    bucket_root_dir: &str,
    app_name: &str,
) -> anyhow::Result<Vec<(String, String)>> {
    let content = fs::read_to_string(file_path)
      .context(format!("Failed to read file {} at line 207", file_path.display() ))?;
    let serde_obj: Value = serde_json::from_str(&content)
      .context(format!("Failed to parse file {} at line 209", file_path.display() ))?;

    let description = serde_obj["description"].as_str().unwrap_or_default();
    let version = serde_obj["version"].as_str().unwrap_or_default();
    let bucket_name = Path::new(bucket_root_dir)
        .file_name()
        .unwrap_or("unknown".as_ref())
        .to_str()
        .unwrap();
    let website = serde_obj["homepage"].as_str().unwrap_or_default();
    let license = serde_obj["license"].as_str().unwrap_or_default();
    let update_at = get_file_modified_time(file_path.to_str().unwrap_or(""))?;
    let binary = serde_obj["bin"].as_str().unwrap_or_default();
    let architecture = serde_obj.get("architecture");
    let binary: Box<dyn DisplayInfo> = if binary.is_empty() {
        match serde_obj["bin"] {
            Value::Array(ref arr) => Box::new(arr.as_slice()) as Box<dyn DisplayInfo>,
            _ => {
                if architecture.is_some() {
                    let architecture = architecture.unwrap();
                    let arch = get_system_default_arch()?;
                    match arch.as_str() {
                        "64bit" => {
                            let x64 = architecture.get("64bit");
                            if x64.is_some() {
                                let bin = x64.unwrap().get("bin");
                                if bin.is_some() {
                                    let bin_str = bin.clone().unwrap().as_str().unwrap_or_default();
                                    if bin_str.is_empty() {
                                        let bin_arr = bin.unwrap().as_array().unwrap();
                                        Box::new(bin_arr.as_slice()) as Box<dyn DisplayInfo>
                                    } else {
                                        Box::new(vec![Value::String(bin_str.to_string())])
                                            as Box<dyn DisplayInfo>
                                    }
                                } else {
                                    Box::new(vec![Value::Null]) as Box<dyn DisplayInfo>
                                }
                            } else {
                                Box::new(vec![Value::Null]) as Box<dyn DisplayInfo>
                            }
                        }
                        "32bit" => {
                            let x32 = architecture.get("32bit");
                            if x32.is_some() {
                                let bin = x32.unwrap().get("bin");
                                if bin.is_some() {
                                    let bin_str = bin.clone().unwrap().as_str().unwrap_or_default();
                                    if bin_str.is_empty() {
                                        let bin_arr = bin.unwrap().as_array().unwrap();
                                        Box::new(bin_arr.as_slice()) as Box<dyn DisplayInfo>
                                    } else {
                                        Box::new(vec![Value::String(bin_str.to_string())])
                                            as Box<dyn DisplayInfo>
                                    }
                                } else {
                                    Box::new(vec![Value::Null]) as Box<dyn DisplayInfo>
                                }
                            } else {
                                Box::new(vec![Value::Null]) as Box<dyn DisplayInfo>
                            }
                        }
                        "arm64" => {
                            let arm64 = architecture.get("arm64");
                            if arm64.is_some() {
                                let bin = arm64.unwrap().get("arm64");
                                if bin.is_some() {
                                    let bin_str = bin.clone().unwrap().as_str().unwrap_or_default();
                                    if bin_str.is_empty() {
                                        let bin_arr = bin.unwrap().as_array().unwrap();
                                        Box::new(bin_arr.as_slice()) as Box<dyn DisplayInfo>
                                    } else {
                                        Box::new(vec![Value::String(bin_str.to_string())])
                                            as Box<dyn DisplayInfo>
                                    }
                                } else {
                                    Box::new(vec![Value::Null]) as Box<dyn DisplayInfo>
                                }
                            } else {
                                Box::new(vec![Value::Null]) as Box<dyn DisplayInfo>
                            }
                        }
                        _ => Box::new(vec![Value::Null]) as Box<dyn DisplayInfo>,
                    }
                } else {
                    Box::new(binary) as Box<dyn DisplayInfo>
                }
            }
        }
    } else {
        Box::new(vec![Value::String(binary.to_string())]) as Box<dyn DisplayInfo>
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

    let short_str = if short_str.is_empty() {
        if architecture.is_some() {
            let architecture = architecture.unwrap();
            let arch = get_system_default_arch()?;
            match arch.as_str() {
                "64bit" => {
                    let x64 = architecture.get("64bit");
                    if x64.is_some() {
                        let shortcut = x64.unwrap().get("shortcuts");
                        if shortcut.is_some() {
                            let shortcut_str =
                                shortcut.clone().unwrap().as_str().unwrap_or_default();
                            if shortcut_str.is_empty() {
                                let shortcut_arr = shortcut.unwrap().as_array().unwrap();
                                shortcut_arr
                                    .iter()
                                    .map(|val| val.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            } else {
                                shortcut_str.to_string()
                            }
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                }
                "32bit" => {
                    let x32 = architecture.get("32bit");
                    if x32.is_some() {
                        let shortcut = x32.unwrap().get("shortcuts");
                        if shortcut.is_some() {
                            let shortcut_str =
                                shortcut.clone().unwrap().as_str().unwrap_or_default();
                            if shortcut_str.is_empty() {
                                let shortcut_arr = shortcut.unwrap().as_array().unwrap();
                                shortcut_arr
                                    .iter()
                                    .map(|val| val.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            } else {
                                shortcut_str.to_string()
                            }
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                }
                "arm64" => {
                    let arm64 = architecture.get("arm64");
                    if arm64.is_some() {
                        let shortcut = arm64.unwrap().get("shortcuts");
                        if shortcut.is_some() {
                            let shortcut_str =
                                shortcut.clone().unwrap().as_str().unwrap_or_default();
                            if shortcut_str.is_empty() {
                                let shortcut_arr = shortcut.unwrap().as_array().unwrap();
                                shortcut_arr
                                    .iter()
                                    .map(|val| val.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            } else {
                                shortcut_str.to_string()
                            }
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                }
                _ => String::new(),
            }
        } else {
            String::new()
        }
    } else {
        short_str
    };

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
    let count = info.len();
    if count == 0 {
        println!("{}", "No app found!".dark_cyan().bold());
        return;
    }
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
    let   i = 0;
    let terminal_max_width = terminal::size()
        .map(|(width, _)| width as usize)
        .unwrap_or(80);
    let all_width = max_value_length + max_key_length + 6;
    for vec in info {
        for (key, value) in vec {
            if !value.is_empty() {

                match key.as_str().trim()  {
                    "Notes" => {
                        let lines: Vec<&str> = value.split('\n').collect(); /* 包含的\n 会产生空字符串 */
                        let lines = lines.iter().map(|line| line.trim()).collect::<Vec<&str>>();
                        let lines_width =
                            lines.iter().map(|line| line.len()).collect::<Vec<usize>>();

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
                    "Binary" =>
                      {format_key_value!(&key, &value, max_key_length, terminal_max_width);

                      },
                    "Shortcuts" => {
                        format_key_value!(&key, &value, max_key_length, terminal_max_width)
                    }

                    "Description" => {
                         format_key_value!(&key, &value, max_key_length, terminal_max_width)
                    }
                    _ => println!(
                        "{}\t{:<width2$}",
                        key.clone().dark_green().bold(),
                        ": ".to_string() + &value,
                        width2 = all_keys_values_width[i]
                    ),
                }
            }
        }
        if all_width < terminal_max_width {
            println!("{}", "-".repeat(all_width).dark_magenta().bold(),);
        } else {
            println!("{}", "-".repeat(terminal_max_width).dark_magenta().bold());
        }
    }
}




fn format_and_print(
    key: &str,
    value: &str,
    max_key_length: usize,
    terminal_max_width: usize,
) -> anyhow::Result<() > {

    let lines: Vec<&str> = value.split('\n').collect();
    let lines: Vec<&str> = lines.iter().map(|line| line.trim()).collect();

    if lines.is_empty() {
       bail!("Empty input value".to_string());
    }

    let lines_width: Vec<usize> = lines.iter().map(|line| line.len()).collect();
    let all_width = max_key_length + lines_width[0] + 6; // 6是": "和边距的固定宽度

    if all_width > terminal_max_width {
        let non_empty_lines: Vec<&str> = lines
            .iter()
            .filter(|line| !line.is_empty())
            .copied()
            .collect();

        if non_empty_lines.is_empty() {
            bail!("No valid content after filtering empty lines".to_string());
        }

        let merged_content = non_empty_lines.join(" ");
        let align_width = terminal_max_width.saturating_sub(max_key_length + 6);

        let align_text = textwrap::wrap(&merged_content, align_width)
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<String>>();

        let wrapped_lines_width: Vec<usize> = align_text.iter().map(|line| line.len()).collect();

        println!(
            "{:<width$}\t{:<width$}",
            key.dark_green().bold(),
            ": ".to_string() + &align_text[0],
            width = wrapped_lines_width[0] + 1
        );

        for i in 1..align_text.len() {
            println!(
                "{}   {:<width$}",
                " ".repeat(max_key_length + 4),
                align_text[i],
                width = wrapped_lines_width[i] + 1
            );
        }

        return Ok(());
    }

    println!(
        "{:<width$}\t{:<width$}",
        key.dark_green().bold(),
        ": ".to_string() + lines[0],
        width = lines_width[0] + 1
    );

    for i in 1..lines.len() {
        println!(
            "{}   {:<width$}",
            " ".repeat(max_key_length + 4),
            lines[i],
            width = lines_width[i] + 1
        );
    }

    Ok(())
}

fn get_file_modified_time(file_path: &str) -> anyhow::Result<String> {
    let metadata = fs::metadata(file_path)
      .context(format!("Failed to get metadata of file {} at line 647", file_path))?;
    let time = metadata.modified()
      .context(format!("Failed to get modified time of file {} at line 649", file_path))?;
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
