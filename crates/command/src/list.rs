use std::any::Any;
use crate::init_hyperscoop;
use crate::utils::get_file_or_dir_metadata::get_dir_updated_time;
use crate::utils::safe_check::is_directory_empty;
use crossterm::style::Stylize;
use rayon::prelude::*;
use regex::Regex;
use std::fs::{read_dir, remove_dir_all};
use std::io::read_to_string;
pub fn list_specific_installed_apps(query: &String) {
    let package = list_all_installed_apps();
    let app_name_list = package.0;
    let app_version = package.1;
    let app_source_bucket = package.2;
    let app_update_date = package.3;
    // let (mut app_name, mut version, mut source,
    //   mut update_date) = (String::new(), String::new(), String::new(), String::new());
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

    for i in 0..app_name_list.len() {
        if app_name_list[i].to_lowercase() == query.clone().to_lowercase()
            || app_name_list[i].contains(query)
        {
            println!(
                "{:<30}\t{:<23}\t{:<20}\t{:<10} ",
                app_name_list[i], app_version[i], app_source_bucket[i], app_update_date[i]
            );
        };
    }
}

pub fn get_all_installed_apps_name() -> Vec<String> {
  let apps_path = init_hyperscoop().unwrap().apps_path;
  let app_name_list: Vec<String> = read_dir(&apps_path)
    .unwrap()
    .par_bridge() // 将标准迭代器转换为并行迭代器
    .filter_map(|entry| {
      let entry = entry.ok()?;
      let  file_type = entry.file_type().ok()?;
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
  return app_name_list;
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
            if  file_type.is_dir() {
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
pub fn display_app_info() {
    let package = list_all_installed_apps();
    let app_name_list = package.0;
    let app_version = package.1;
    let app_source_bucket = package.2;
    let app_update_date = package.3;
    println!("Installed Apps Count:{} \n", app_name_list.len());
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

    for i in 0..app_name_list.len() {
        println!(
            "{:<30}\t{:<23}\t{:<20}\t{:<10} ",
            app_name_list[i], app_version[i], app_source_bucket[i], app_update_date[i]
        );
    }
}
fn get_apps_update_date(apps_path: &String) -> Vec<String> {
    let mut app_update_date = vec![];
    for apps_file in read_dir(apps_path).unwrap() {
        let apps_file = apps_file.unwrap();
      let  file_type= apps_file.file_type().unwrap();
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
    return app_update_date;
}

fn get_apps_source_bucket(apps_path: &String) -> Vec<String> {
    let mut app_source_bucket = Vec::new();
    for apps_file in read_dir(apps_path).unwrap() {
        let apps_file = apps_file.unwrap();
      let  file_type= apps_file.file_type().unwrap();
        let apps_file = apps_file.path();
           if file_type.is_dir()  {
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

            if  install_file.exists() {
                let install_file = install_file.to_str().unwrap().to_string();
                let reader = std::io::BufReader::new(std::fs::File::open(install_file).unwrap());
                let install_file = read_to_string(reader).expect("Unable to read install file");
                let source = serde_json::from_str::<serde_json::Value>(&install_file)
                    .expect("Unable to parse install file");
                let version = source
                    .get("bucket")
                    .expect("获取 Source Bucket 失败 ")
                    .to_string();
                let re = Regex::new(r#"^\"|\"$"#).unwrap(); // 匹配字符串开头和结尾的双引号
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

    return app_source_bucket;
}

fn get_apps_version(apps_path: &String) -> Vec<String> {
    let mut app_version: Vec<String> = Vec::new();
    for entry in read_dir(apps_path).unwrap() {
        let entry = entry.unwrap();
        let  file_type =entry.file_type().unwrap();
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
                let  file_type = version_dir.file_type().unwrap();
                let version_path = version_dir.path();

                if   file_type.is_dir()  {
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
    return app_version;
}
