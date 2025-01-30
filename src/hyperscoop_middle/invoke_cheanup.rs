use std::collections::HashMap;
use crossterm::style::Stylize;
use command_util_lib::init_hyperscoop;
use crate::command_args::cleanup::CleanupArgs;

pub  fn  execute_cleanup_command(args: CleanupArgs) -> Result<(), anyhow::Error> {
  if let Some(name) = args.name { 
    if name == "*" {   clean_all_old_versions()   } 
    else {
      clean_specific_old_version(name)
    }
  }  
  if args.all { 
    log::info!("Run cleanup all command"); 
      clean_all_old_versions()  
  }  
  Ok(()) 
}

fn clean_specific_old_version(   app_name  : String) {
   log::info!("Run cleanup command '{}'", app_name);
}

fn clean_all_old_versions() {
  let hyperscoop = init_hyperscoop().expect("Failed to initialize hyperscoop");
  let apps_path = hyperscoop.apps_path;
  let apps_dir = std::fs::read_dir(&apps_path).expect("Failed to read apps directory");
  let mut versions_with_name = HashMap::new();

  for app_dir in apps_dir {
    let mut dir_count = 0;
    let mut versions_max = String::new();

    let path = app_dir.expect("Failed to read app directory").path();
    let app_name = path.clone().file_name().unwrap().to_str().unwrap().to_string();
    if path.is_dir() {
      for entry in path.read_dir().expect("Failed to read app directory") {
        let entry = entry.expect("Failed to read app directory entry");
        if entry.path().is_dir() {
          dir_count += 1;
        }
      }
      if dir_count <= 2 { continue; }
      for version_dir in path.read_dir().expect("Failed to read version directory") {
        let version_path = version_dir.expect("Failed to read version directory").path();
        if version_path.is_dir() {
          let version_name = version_path.file_name().expect("Failed to get version name").to_str().expect("Failed to convert version name to string");
          if version_name == "current" { continue; }
          match compare_versions(version_name.to_string(), versions_max.clone()) {
            Ordering::Less => {}
            Ordering::Equal => {}
            Ordering::Greater => { versions_max = version_name.to_string() }
          }
        }
      }
    }
    versions_with_name.insert(app_name, versions_max.clone());
  }
  if versions_with_name.len() == 0 || versions_with_name.is_empty() {
    println!("{}", "No old versions found".green().bold());
  }
  log::info!("{:?}", versions_with_name);
  for (app_name, version) in versions_with_name {
    let exclude_path = format!("{}\\{}\\{}", apps_path.clone(), app_name, version);
    let exclude_path = std::path::Path::new(&exclude_path); 
    let   dir = format!("{}\\{}", apps_path.clone(), app_name)  ;
    log::info!("{:?}", dir);
    if exclude_path.exists() {
      for entry in std::fs::read_dir(dir ).expect("Failed to read app directory") { 
        let entry = entry.expect("Failed to read app directory entry"); 
        if entry.path().is_dir() { 
          if entry.path() == exclude_path { continue; }
          let  name = entry.path().file_name().unwrap().to_str().unwrap().to_string(); 
          if name == "current"  { continue; }  
          log :: info!("Removing old version: {}", entry.path().display());
          std::fs::remove_dir_all(entry.path()).expect("Failed to remove old version");
        }
      }
    }
  }
}

use std::cmp::Ordering;

fn compare_versions(ver1: String, ver2: String) -> Ordering {
  // 分割版本号并转换为数字数组
  let v1: Vec<i32> = ver1.split('.').flat_map(|s| s.parse()).collect();
  let v2: Vec<i32> = ver2.split('.').flat_map(|s| s.parse()).collect();

  // 动态比较每一级（自动补零）
  let max_len = v1.len().max(v2.len());
  (0..max_len)
    .map(|i| (v1.get(i).copied().unwrap_or(0), v2.get(i).copied().unwrap_or(0)))
    .find_map(|(a, b)| match a.cmp(&b) {
      Ordering::Equal => None,
      diff => Some(diff),
    })
    .unwrap_or(Ordering::Equal)
}
