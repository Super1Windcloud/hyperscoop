use std::path::{Path, PathBuf};
use crossterm::style::Stylize;
use crate::buckets::Buckets;
use crate::utils::detect_encoding::transform_to_serde_value_object;

pub fn fuzzy_search(query: String) {
  let bucket = Buckets::new();
  let buckets_path = bucket.buckets_path;
  for entry in buckets_path {
    let path = Path::new(&entry);
    if path.is_dir() && path.exists() {
      let bucket_path = path.join("bucket");
      if bucket_path.exists() && bucket_path.is_dir() {
        let result = get_apps_info(&bucket_path, &query).expect("Failed to get apps info");
        display_result(&result);
      }
    }
  }
}

fn get_apps_info(path: &PathBuf, query: &String)
                 -> Result<(Vec<String>, Vec<String>, Vec<String>), anyhow::Error> {
  let mut app_names = Vec::new();
  let mut app_versions = Vec::new();
  let mut app_sources = Vec::new();
  for entry in path.read_dir()? {
    let path = entry?.path();
    if path.is_file() && path.exists() && path.extension().unwrap_or_default() == "json" {
      let app_info = transform_to_serde_value_object(&path).expect("Failed to parse json file");

      let app_version = app_info["version"].as_str().unwrap_or("null");
      let app_name = path.file_stem().unwrap().to_str().unwrap();
      let source = path.parent().unwrap().parent().unwrap().
        file_stem().unwrap().to_string_lossy().to_string();
      if app_name == query || app_name.contains(query) {
        app_versions.push(app_version.to_string());
        app_names.push(app_name.to_string());
        app_sources.push(source);
      }
    }
  }

  let result = (app_names, app_versions, app_sources);
  Ok(result)
}


pub fn exact_search(query: String) {
  println!("Exact search  ")
}


fn display_result(result: &(Vec<String>, Vec<String>, Vec<String>)) {
  let app_names = &result.0;
  let app_versions = &result.1;
  let app_sources = &result.2;

  println!("{}", "Results from local buckets...\n".dark_green().bold());
  println!("{:<30}\t\t\t\t{:<30}\t\t\t\t{:<30}  ",
           "Name".dark_green().bold(), "Version".dark_green().bold(),
           "Source_Bucket".dark_green().bold());
  println!("{:<30}\t\t\t\t{:<30}\t\t\t\t{:<30}  ",
           "____".dark_green().bold(), "_______".dark_green().bold(),
           "_____________".dark_green().bold());

  for i in 0..app_names.len() {
    println!("{:<30}\t\t\t\t{:<30}\t\t\t\t{:<30}  ",
             app_names[i], app_versions[i],
             app_sources[i]);
  }
}
