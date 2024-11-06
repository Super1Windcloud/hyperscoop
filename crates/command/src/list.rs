use std::fs::{read_dir, remove_dir, remove_dir_all};
use std::io::read_to_string;
use crossterm::style::Stylize;
use regex::Regex;
use crate::init_hyperscoop;
use crate::utils::get_file_or_dir_metadata::get_dir_updated_time;
use crate::utils::safe_check::is_directory_empty;


pub fn list_specific_installed_apps(query: &String) {
  todo!()
}

pub fn list_all_installed_apps() {
  let apps_path = init_hyperscoop().unwrap().apps_path;
  let mut app_name_list: Vec<String> = Vec::new();
  for entry in read_dir(&apps_path).unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();
    if path.is_dir() {
      let app_name = path.file_name().unwrap().to_str().unwrap();
      // 统一全部排除scoop自身
      if path.file_name().unwrap().to_str().unwrap() == "scoop" {
        continue;
      }
      if app_name != "scoop" {
        app_name_list.push(String::from(app_name));
      }
    }
  }
  println!("Installed Apps Count:{} \n", app_name_list.len());
  println!("{:<30}\t\t\t\t{:<30}\t\t\t{:<30}\t\t\t{:<30} ",
           "Name".dark_green().bold(), "Version".dark_green().bold(),
           "Bucket".dark_green().bold(), "UpDate".dark_green().bold());
  println!("{:<30}\t\t\t\t{:<30}\t\t\t{:<30}\t\t\t{:<30} ",
           "____".dark_green().bold(), "_______".dark_green().bold(),
           "______".dark_green().bold(), "______".dark_green().bold());


  let mut app_version: Vec<String> = Vec::with_capacity(app_name_list.len());
  let mut app_source_bucket: Vec<String> = Vec::with_capacity(app_name_list.len());
  let mut app_update_date: Vec<String> = Vec::with_capacity(app_name_list.len());
  app_version = get_apps_version(&apps_path);
  app_source_bucket = get_apps_source_bucket(&apps_path);
  app_update_date = get_apps_update_date(&apps_path);
  // println!("name{} version{} bucket{} update{}", app_name_list.len()
  //          , app_version.len(), app_source_bucket.len(), app_update_date.len());


  for i in 0..app_name_list.len() {
    println!("{:<30}\t{:<23}\t{:<20}\t{:<10} ",
             app_name_list[i], app_version[i],
             app_source_bucket[i], app_update_date[i]);
  }

  // rust 文件系统IO默认是异步非阻塞的 , 所有一定尽可能的明确判断边界条件和空值检查

}

fn get_apps_update_date(apps_path: &String) -> Vec<String> {
  let mut app_update_date = vec![];
  for apps_file in read_dir(apps_path).unwrap() {
    let apps_file = apps_file.unwrap();
    let apps_file = apps_file.path();
    if apps_file.file_name().unwrap().to_str().unwrap() == "scoop" {
      continue;
    }
    if apps_file.is_dir() && !is_directory_empty(&apps_file) {
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
    let apps_file = apps_file.path();
    if apps_file.is_dir() && apps_file.exists() {
      //检测目录安全性
      if is_directory_empty(&apps_file) {
        println!("{} is empty, removing it", apps_file.to_str().unwrap());
        remove_dir_all(&apps_file).unwrap();  // 删除空目录
        continue;
      }
      if apps_file.file_name().unwrap().to_str().unwrap() == "scoop" {
        continue;
      }
      let install_file = apps_file.join("current\\install.json");

      if install_file.is_file() && install_file.exists() {
        let install_file = install_file.to_str().unwrap().to_string();
        let reader = std::io::BufReader::new(std::fs::File::open(install_file).unwrap());
        let install_file = read_to_string(reader).expect("Unable to read install file");
        let source = serde_json::from_str::<serde_json::Value>(&install_file).expect("Unable to parse install file");
        let version = source.get("bucket").expect("获取 Source Bucket事变").to_string();
        let re = Regex::new(r#"^\"|\"$"#).unwrap(); // 匹配字符串开头和结尾的双引号
        let unquoted_str = re.replace_all(&version, "").to_string(); // 替换为 ""
        app_source_bucket.push(unquoted_str);
      }
    }
  };

  return app_source_bucket;
}

fn get_apps_version(apps_path: &String) -> Vec<String> {
  let mut app_version: Vec<String> = Vec::new();
  for entry in read_dir(apps_path).unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();

    if is_directory_empty(&path) {
      println!("{} is empty, removing it", path.to_str().unwrap());
      remove_dir_all(&path).unwrap();  // 删除空目录
      continue;
    }
    if path.file_name().unwrap().to_str().unwrap() == "scoop" {
      continue;
    }
    if path.is_dir() {
      let mut max_version = String::new();

      for version_dir in read_dir(path).unwrap() {
        let version_dir = version_dir.unwrap();
        let version_path = version_dir.path();

        if version_path.is_dir() && version_path.exists() {
          // 检查目录安全性
          if is_directory_empty(&version_path) {
            println!("{} is empty, removing it", version_path.to_str().unwrap());
            remove_dir_all(&version_path).unwrap();  // 删除空目录
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
      };

      app_version.push(String::from(max_version));
    }
  }
  return app_version;
}
