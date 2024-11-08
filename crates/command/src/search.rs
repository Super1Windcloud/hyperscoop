use std::path::{Path, PathBuf};
use std::process::exit;
use crossterm::style::Stylize;
use crate::buckets::Buckets;
use crate::utils::detect_encoding::{transform_to_only_version_manifest};
use crate::list::get_all_installed_apps_name;
use crate::manifest::search_manifest::SearchManifest;

pub fn fuzzy_search(query: String) {
  let bucket = Buckets::new();
  let buckets_path = bucket.buckets_path;
  let mut result = Vec::new();
  for entry in buckets_path {
    let path = Path::new(&entry);
    if path.is_dir() && path.exists() {
      let bucket_path = path.join("bucket");
      if bucket_path.exists() && bucket_path.is_dir() {
        let temp = get_apps_info(&bucket_path, &query).expect("Failed to get apps info");
        result.push(temp);
      }
    }
  }
  let mut result_count = 0;
  for i in 0..result.len() {
    result_count += result[i].0.len();
  }

  println!("{} {}", result_count.to_string().dark_green().bold(),
           "Results from local buckets...\n".dark_green().bold());

  // println!("加载完毕");
  sort_result_by_bucket_name(result);
}
fn sort_result_by_bucket_name(mut result: Vec<(Vec<String>, Vec<String>, String)>) {

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

  for i in 0..result.len() {
    display_result(&result[i])
  }
}

fn get_apps_info(path: &PathBuf, query: &String)
                 -> Result<(Vec<String>, Vec<String>, String), anyhow::Error> {
  let mut app_names = Vec::new();
  let mut app_versions = Vec::new();
  let mut app_sources = String::new();
  for entry in path.read_dir()? {
    let path = entry?.path();
    if path.is_file() && path.extension().unwrap_or_default() == "json" {
      let app_name = path.file_stem().unwrap().to_str().unwrap();
      let source = path.parent().unwrap().parent().unwrap().
        file_stem().unwrap().to_string_lossy().to_string();

      if app_name.to_lowercase() == query.to_lowercase() ||
        app_name.to_lowercase().contains(&query.to_lowercase()) {
        let app_info: SearchManifest = transform_to_only_version_manifest(&path)?;
        let app_version = app_info.version.unwrap_or_default();


        app_versions.push(app_version.to_string());
        app_names.push(app_name.to_string());
        app_sources = source;
      }
    };
  }

  let result = (app_names, app_versions, app_sources);
  Ok(result)
}


pub fn exact_search(query: String) {
  println!("Exact search  ")
}


fn display_result(result: &(Vec<String>, Vec<String>, String)) {
  let app_names = &result.0;
  let app_versions = &result.1;
  let app_sources = &result.2;


  for i in 0..app_names.len() {
    println!("{:<30}\t{:<30}\t{:<30}",
             app_names[i], app_versions[i],
             app_sources);
  }
}

fn is_installed(app_name: &str) -> bool {
  let apps_list = get_all_installed_apps_name();
  for item in apps_list {
    if item == app_name { return true; }
  }
  return false;
}
