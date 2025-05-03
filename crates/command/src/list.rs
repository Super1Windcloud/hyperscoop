use crate::init_env::{
    get_app_dir_install_json, get_app_dir_manifest_json, get_apps_path, get_apps_path_global,
};
use crate::init_hyperscoop;
use crate::utils::get_file_or_dir_metadata::get_dir_updated_time;
use crate::utils::safe_check::is_directory_empty;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use crossterm::style::Stylize;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::{read_dir, remove_dir_all};
use std::io::read_to_string;
use std::path::Path;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArchType {
  #[serde(rename = "32bit")]
  X86,
  #[serde(rename = "64bit")]
  X64,
  #[serde(rename = "arm64")]
  Arm64,
}

impl Default for ArchType {
  fn default() -> Self {
    ArchType::X64
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionJSON {
    pub bucket: Option<String>,
    pub version: Option<String>, 
    #[serde(skip)]
    pub architecture: Option<ArchType>,
}

pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub bucket: String,
    pub update_date: String,
}

impl AppInfo {
    pub fn new(name: String, version: String, bucket: String, update_date: String) -> Self {
        Self {
            name,
            version,
            bucket,
            update_date,
        }
    }
    pub fn get_field_widths(&self) -> (usize, usize, usize, usize) {
        (
            self.name.len(),
            self.version.len(),
            self.bucket.len(),
            self.update_date.len(),
        )
    }
}

pub fn list_all_installed_apps_refactor(is_global: bool) -> anyhow::Result<Vec<AppInfo>> {
    let apps_dir = if is_global {
        get_apps_path_global()
    } else {
        get_apps_path()
    };
    let all_apps_path = read_dir(&apps_dir)?
        .par_bridge() // 将标准迭代器转换为并行迭代器
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_type = entry.file_type().ok()?;
            if file_type.is_dir() {
                let path = entry.path();
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let all_apps_path = all_apps_path
        .par_iter()
        .filter_map(|path| {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if name == "scoop" {
                return None;
            }
            return Some(path.clone());
        })
        .collect::<Vec<_>>();

    let all_app_infos = all_apps_path
        .par_iter()
        .filter_map(|path| {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let install_json = get_app_dir_install_json(&name);
            let manifest_json = get_app_dir_manifest_json(&name);
            let bucket = get_install_json_bucket(&install_json).unwrap();
            let update_data = get_dir_updated_time(&path);
            let version = get_install_json_version(&manifest_json).unwrap_or_else(|e| {
                log::warn!("Failed to get install json version: {}", e);
                #[cfg(debug_assertions)] // 编译器排除
                println!("path is {}", manifest_json);
                return "unknown".to_string();
            });
            Some(AppInfo::new(name, version, bucket, update_data))
        })
        .collect::<Vec<_>>();
    Ok(all_app_infos)
}

pub fn get_install_json_version(manifest_json: &String) -> anyhow::Result<String> {
    let path = Path::new(manifest_json);
    if !path.exists() {
        return Ok("unknown".to_string());
    }
    let content = std::fs::read_to_string(manifest_json)?;
    let obj: VersionJSON = serde_json::from_str(&content)?;
    let version = obj.version;
    if version.is_none() {
        return Ok("unknown".to_string());
    }
    let version = version.unwrap();
    Ok(version)
}

pub fn get_install_json_bucket(install_json: &String) -> anyhow::Result<String> {
    if Path::new(install_json).exists() == false {
        return Ok("unknown".to_string());
    }
    let content = std::fs::read_to_string(install_json)?;
    let obj: VersionJSON = serde_json::from_str(&content)?;
    let bucket = obj.bucket;
    if bucket.is_none() {
        return Ok("unknown".to_string());
    }
    Ok(bucket.unwrap())
}

pub fn list_specific_installed_apps_extra(
    query: Vec<String>,
    is_global: bool,
) -> anyhow::Result<()> {
    let package = list_all_installed_apps_refactor(is_global)?;
    let mut filtered_apps = package
        .into_iter()
        .filter(|app| {
            query
                .iter()
                .any(|q| app.name.to_lowercase() == q.to_lowercase() || app.name.contains(q))
        })
        .collect::<Vec<_>>();
    filtered_apps.sort_by(|a, b| a.name.cmp(&b.name));

    let count = filtered_apps.len();
    if count == 0 {
        println!("{}", "No app To Found !".to_string().dark_cyan().bold());
        return Ok(());
    }

    let installed_apps = filtered_apps
        .iter()
        .map(|app| {
            vec![
                app.name.clone(),
                app.version.clone(),
                app.bucket.clone(),
                app.update_date.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("AppName")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
            Cell::new("AppVersion")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
            Cell::new("SourceBucket")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
            Cell::new("UpdatedTime")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
        ])
        .add_rows(installed_apps.as_slice());

    println!("{table}");

    Ok(())
}
pub fn list_specific_installed_apps(query: Vec<String>, is_global: bool) -> anyhow::Result<()> {
    let mut package = list_all_installed_apps_refactor(is_global)?;
    let apps_name_list = package
        .iter()
        .map(|app| app.name.clone())
        .collect::<Vec<_>>();
    let found_flag = apps_name_list.par_iter().any(|name| {
        let flag = query.iter().any(|q| name == q);
        return flag;
    });
    if !found_flag {
        println!("{}", "No app To Found !".to_string().dark_cyan().bold());
        return Ok(());
    }
    println!(
        "{:<30}\t\t\t\t{:<30}\t\t\t{:<30}\t\t\t{:<30} ",
        "Name".dark_green().bold(),
        "Version".dark_green().bold(),
        "Bucket".dark_green().bold(),
        "UpDate".dark_green().bold()
    );
    println!(
        "{:<30}\t\t\t\t{:<30}\t\t\t{:<30}\t\t\t{:<30} ",
        "____".dark_green().bold(),
        "_______".dark_green().bold(),
        "______".dark_green().bold(),
        "______".dark_green().bold()
    );
    package.sort_by(|a, b| a.name.cmp(&b.name));
    for item in package.iter() {
        for query in query.iter() {
            if item.name.to_lowercase() == query.clone().to_lowercase() || item.name.contains(query)
            {
                println!(
                    "{:<30}\t{:<23}\t{:<20}\t{:<10} ",
                    item.name, item.version, item.bucket, item.update_date
                );
            }
        }
    }
    Ok(())
}

pub fn get_all_installed_apps_name() -> Vec<String> {
    let apps_path = init_hyperscoop().unwrap().apps_path;
    let app_name_list: Vec<String> = read_dir(&apps_path)
        .unwrap()
        .par_bridge() // 将标准迭代器转换为并行迭代器
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_type = entry.file_type().ok()?;
            let path = entry.path();
            if file_type.is_dir() {
                let app_name = path.file_name()?.to_str()?;
                if app_name != "scoop" {
                    return Some(app_name.to_string());
                }
            }
            None
        })
        .collect();
    app_name_list
}
pub fn list_all_installed_apps() -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let apps_path = init_hyperscoop().unwrap().apps_path;
    let app_name_list: Vec<String> = read_dir(&apps_path)
        .unwrap()
        .par_bridge() // 将标准迭代器转换为并行迭代器
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_type = entry.file_type().ok()?;
            let path = entry.path();
            if file_type.is_dir() {
                let app_name = path.file_name()?.to_str()?;
                if app_name != "scoop" {
                    return Some(app_name.to_string());
                }
            }
            None
        })
        .collect();
    let app_version = get_apps_version(&apps_path);
    let app_source_bucket = get_apps_source_bucket(&apps_path);
    let app_update_date = get_apps_update_date(&apps_path);

    let package = (
        app_name_list,
        app_version,
        app_source_bucket,
        app_update_date,
    );
    // rust 文件系统IO默认是异步非阻塞的 , 所有一定尽可能的明确判断边界条件和空值检查
    package
}
pub fn get_max_field_widths(apps: &[AppInfo]) -> (usize, usize, usize, usize) {
    apps.iter().fold(
        (0, 0, 0, 0),
        |(max_name, max_ver, max_bucket, max_date), app| {
            let (name, ver, bucket, date) = app.get_field_widths();
            (
                max_name.max(name),
                max_ver.max(ver),
                max_bucket.max(bucket),
                max_date.max(date),
            )
        },
    )
}

pub fn display_apps_info_extra(is_global: bool) -> anyhow::Result<()> {
    let mut package = list_all_installed_apps_refactor(is_global)?;
    package.sort_by(|a, b| a.name.cmp(&b.name));
    let installed_apps = package
        .iter()
        .map(|app| {
            vec![
                app.name.clone(),
                app.version.clone(),
                app.bucket.clone(),
                app.update_date.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let counts = package.len();
    println!(
        "{} :{} \n",
        "Installed Apps Count".dark_cyan().bold(),
        counts.to_string().dark_cyan().bold()
    );

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("AppName")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
            Cell::new("AppVersion")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
            Cell::new("SourceBucket")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
            Cell::new("UpdatedTime")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
        ])
        .add_rows(installed_apps.as_slice());

    println!("{table}");
    Ok(())
}
pub fn display_app_info(is_global: bool) -> anyhow::Result<()> {
    let mut package = list_all_installed_apps_refactor(is_global)?;
    package.sort_by(|a, b| a.name.cmp(&b.name));

    let counts = package.len();
    let col_widths = get_max_field_widths(&package);
    let app_name_list = col_widths.0 + 3;
    let app_version = col_widths.1 + 3;
    let app_source_bucket = col_widths.2 + 3;
    let app_update_date = col_widths.3 + 3;
    let col_widths = [
        app_name_list,
        app_version,
        app_source_bucket,
        app_update_date,
    ];
    println!(
        "{} :{} \n",
        "Installed Apps Count".dark_cyan().bold(),
        counts.to_string().dark_cyan().bold()
    );

    let all_widths = app_name_list + app_version + app_source_bucket + app_update_date - 6;
    println!(
        "{}",
        "-".to_string().repeat(all_widths + 8).dark_green().bold()
    );
    println!(
        "{:<width1$} {:<width2$} {:<width3$} {:<width4$}",
        "| Name",
        "| Version",
        "| Bucket",
        "| Update",
        width1 = col_widths[0],
        width2 = col_widths[1],
        width3 = col_widths[2],
        width4 = col_widths[3]
    );
    println!(
        "{:<width1$} {:<width2$} {:<width3$} {:<width4$}",
        "| ____",
        "| _______",
        "| ______",
        "| ______",
        width1 = col_widths[0],
        width2 = col_widths[1],
        width3 = col_widths[2],
        width4 = col_widths[3]
    );
    for item in package.iter() {
        println!(
            "{:<width1$} {:<width2$} {:<width3$} {:<width4$}",
            "| ".to_string() + &item.name,
            "| ".to_string() + &item.version,
            "| ".to_string() + &item.bucket,
            "| ".to_string() + &item.update_date,
            width1 = col_widths[0],
            width2 = col_widths[1],
            width3 = col_widths[2],
            width4 = col_widths[3]
        );
    }
    println!(
        "{}",
        "-".to_string().repeat(all_widths + 8).dark_green().bold()
    );
    Ok(())
}
fn get_apps_update_date(apps_path: &String) -> Vec<String> {
    let mut app_update_date = vec![];
    for apps_file in read_dir(apps_path).unwrap() {
        let apps_file = apps_file.unwrap();
        let file_type = apps_file.file_type().unwrap();
        let apps_file = apps_file.path();
        if apps_file.file_name().unwrap().to_str().unwrap() == "scoop" {
            continue;
        }
        if file_type.is_dir() && !is_directory_empty(&apps_file) {
            // 获取目录更新日期
            let time = get_dir_updated_time(&apps_file);
            app_update_date.push(time);
        }
    }
    app_update_date
}

fn get_apps_source_bucket(apps_path: &String) -> Vec<String> {
    let mut app_source_bucket = Vec::new();
    for apps_file in read_dir(apps_path).unwrap() {
        let apps_file = apps_file.unwrap();
        let file_type = apps_file.file_type().unwrap();
        let apps_file = apps_file.path();
        if file_type.is_dir() {
            //检测目录安全性
            if is_directory_empty(&apps_file) {
                println!("{} is empty, removing it", apps_file.to_str().unwrap());
                remove_dir_all(&apps_file).unwrap(); // 删除空目录
                continue;
            }
            if apps_file.file_name().unwrap().to_str().unwrap() == "scoop" {
                continue;
            }
            let install_file = apps_file.join("current\\install.json");

            if install_file.exists() {
                let install_file = install_file.to_str().unwrap().to_string();
                let reader = std::io::BufReader::new(std::fs::File::open(install_file).unwrap());
                let install_file = read_to_string(reader).expect("Unable to read install file");
                let source = serde_json::from_str::<serde_json::Value>(&install_file)
                    .expect("Unable to parse install file");
                let version = source
                    .get("bucket")
                    .expect("获取 Source Bucket 失败 ")
                    .to_string();
                let re = Regex::new(r#"^"|"$"#).unwrap(); // 匹配字符串开头和结尾的双引号
                let mut unquoted_str = re.replace_all(&version, "").to_string();
                if unquoted_str.is_empty() {
                    unquoted_str = "unknown".to_string();
                }
                app_source_bucket.push(unquoted_str);
            } else {
                app_source_bucket.push("unknown".to_string());
            }
        }
    }

    app_source_bucket
}

fn get_apps_version(apps_path: &String) -> Vec<String> {
    let mut app_version: Vec<String> = Vec::new();
    for entry in read_dir(apps_path).unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        let path = entry.path();

        if is_directory_empty(&path) {
            println!("{} is empty, removing it", path.to_str().unwrap());
            remove_dir_all(&path).unwrap(); // 删除空目录
            continue;
        }
        if path.file_name().unwrap().to_str().unwrap() == "scoop" {
            continue;
        }
        if file_type.is_dir() {
            let mut max_version = String::new();

            for version_dir in read_dir(path).unwrap() {
                let version_dir = version_dir.unwrap();
                let file_type = version_dir.file_type().unwrap();
                let version_path = version_dir.path();

                if file_type.is_dir() {
                    // 检查目录安全性
                    if is_directory_empty(&version_path) {
                        println!("{} is empty, removing it", version_path.to_str().unwrap());
                        remove_dir_all(&version_path).unwrap(); // 删除空目录
                        continue;
                    }
                    let version_name = version_path.file_name().unwrap().to_str().unwrap();
                    if version_name != "current" {
                        // 只选择最高版本
                        if version_name.to_string() >= max_version {
                            max_version = version_name.to_string();
                        }
                    }
                }
            }

            app_version.push(String::from(max_version));
        }
    }
    app_version
}

mod test_list {

    #[allow(unused_imports)]
    use super::*;
    #[test]
    fn test_filter_apps_path() {
        list_all_installed_apps_refactor(false).unwrap();
    }
}
