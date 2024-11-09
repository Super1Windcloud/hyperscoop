use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, Mutex};
use crossterm::style::Stylize;
use crate::buckets::Buckets;
use crate::utils::detect_encoding::{transform_to_only_version_manifest};
use crate::list::get_all_installed_apps_name;
use crate::manifest::search_manifest::SearchManifest;
use rayon::prelude::*; // 并行处理
pub fn fuzzy_search(query: String) {
  let bucket = Buckets::new();
  let buckets_path = bucket.buckets_path;
  // 存储满足条件的应用名称
  // 使用 ARC和 Mutex 实现多线程安全互斥锁会影响性能，所有这里放弃

  // let mut result = Arc::new(Mutex::new(Vec::new()));
  //
  // buckets_path.par_iter().for_each(|entry| {
  //   let path = Path::new(&entry);
  //   if path.is_dir() {
  //     let bucket_path = path.join("bucket");
  //     if bucket_path.is_dir() {
  //       let temp = get_apps_names(&bucket_path, &query).expect("Failed to get apps info");
  //       if !temp.is_empty() {
  //         let mut result = result.lock().unwrap();
  //         result.extend(temp);
  //       }
  //     }
  //   }
  // });
  // let result = Arc::try_unwrap(result).unwrap().into_inner().unwrap();

  if query.contains("/") {
    let query_args = query.split("/").collect::<Vec<&str>>();
    if query_args.len() == 2 {
      search_app_in_specific_bucket(query_args[0], query_args[1]);
    }
  }
  let all_result: Vec<Vec<(String, PathBuf)>> = buckets_path.
    par_iter().
    filter_map(|entry| {
      let path = Path::new(&entry);
      if path.is_dir() {
        let bucket_path = path.join("bucket");
        if bucket_path.is_dir() {
          let temp = get_apps_names(&bucket_path, &query).expect("Failed to get apps info");
          if !temp.is_empty() {
            Some(temp)
          } else { None }
        } else { None }
      } else { None }
    }).collect(); // 并行处理
  let result = all_result.into_iter().flatten().collect();


  let result_info = get_result_source_and_version(result).unwrap();

  let count = result_info.len();
  println!("{} {}", count.to_string().dark_green().bold(),
           "Results from local buckets...\n".dark_green().bold());
  // println!("加载完毕");
  sort_result_by_bucket_name(result_info);
}


fn get_result_source_and_version(app_names: Vec<(String, PathBuf)>) ->
Result<Vec<(String, String, String)>, anyhow::Error> {
  let result_info = app_names.par_iter().filter_map(|item| {
    let path = &item.1;
    let app_name = item.0.clone();
    let source = path.parent().unwrap().parent().unwrap().
      file_stem().unwrap().to_string_lossy().to_string();
    let version = transform_to_only_version_manifest(path.as_ref())
      .expect("Failed to transform to only version manifest");
    let version = version.get_version().unwrap().to_string();

    Some((app_name, version, source))
  }).collect();


  Ok(result_info)
}


fn search_app_in_specific_bucket(bucket: &str, app_name: &str) {
  todo!();
}

fn sort_result_by_bucket_name(mut result: Vec<(String, String, String)>) {
  //  ( name  version source )
  // 将官方维护的bucket靠前
  let official_bucket =
    ["main", "extras", "versions"];

  for i in 0..result.len() {
    for name in official_bucket {
      if result[i].2 == name {
        result.insert(0, result[i].clone());
        result.remove(i + 1);
        break;
      }
    }
  }
  println!("{:<30}\t\t\t\t{:<30}\t\t\t\t{:<30}  ",
           "Name".dark_green().bold(), "Version".dark_green().bold(),
           "Source_Bucket".dark_green().bold());
  println!("{:<30}\t\t\t\t{:<30}\t\t\t\t{:<30}  ",
           "____".dark_green().bold(), "_______".dark_green().bold(),
           "_____________".dark_green().bold());

  display_result(&result)
}

fn get_apps_names(path: &PathBuf, query: &String)
                  -> Result<Vec<(String, PathBuf)>, anyhow::Error> {
  let query_lower = query.to_lowercase();
  // for entry in path.read_dir()? {
  //   let path = entry?.path();
  //   if path.is_file() && path.extension().unwrap_or_default() == "json" {
  //     let app_name = path.file_stem().unwrap().to_str().unwrap();
  //     let app_name = app_name.to_lowercase();
  //     if app_name == query_lower ||
  //       app_name.contains(&query_lower) {
  //       let app_path = path.clone();
  //       app_names.push((app_name.to_string(), app_path));
  //     }
  //   }
  //
  let app_names = path.read_dir()?
    .into_iter()
    .par_bridge().
    filter_map(|entry| {
      let path = entry.unwrap().path();
      if path.is_file() && path.extension().unwrap_or_default() == "json" {
        let app_name = path.file_stem().unwrap().to_str().unwrap();
        let app_name = app_name.to_lowercase();
        if app_name == query_lower ||
          app_name.contains(&query_lower) {
          let app_path = path.clone();
          Some((app_name.to_string(), app_path))
        } else {
          None
        }
      } else {
        None
      }
    }).collect();
  Ok(app_names)
}

pub fn exact_search(query: String) {
  println!("Exact search  ")
}

fn display_result(result: &Vec<(String, String, String)>) {
  for i in 0..result.len() {
    println!("{:<30}\t{:<30}\t{:<30}",
             result[i].0, result[i].1,
             result[i].2, );
  }
}

fn is_installed(app_name: &str) -> bool {
  let apps_list = get_all_installed_apps_name();
  for item in apps_list {
    if item == app_name { return true; }
  }
  return false;
}
