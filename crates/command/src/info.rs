use dashmap::{  DashSet};
use crossterm::style::Stylize;
use std::fs;
use std::io;
use chrono::{DateTime, Local};
use color_eyre::owo_colors::OwoColorize;
use serde_json::Value;
use rayon::prelude::*;

pub fn display_app_info(app_name: String, bucket_paths: Vec<String>) {
  let app_name = app_name.trim().to_lowercase();
  if  let Some((bucket, name)) = app_name.split_once('/') {
    log::info!("name: {}", name);
    log::info!("bucket: {}", bucket);
    display_specific_app_info(name, bucket, bucket_paths);
    return;
  }
  let      infos_set    = DashSet::new(); 
 
  bucket_paths.par_iter().for_each(|bucket_path| {
    let manifest_path = format!("{}\\bucket", bucket_path);
    // log::info!("manifest_path: {}", manifest_path);

    if let Ok(entries) = fs::read_dir(&manifest_path) {
      entries.par_bridge().for_each(|entry| {
        if let Ok(file) = entry { 
          let  file_type = file.file_type().unwrap();
          let file_path = file.path(); 
          if file_type.is_file() && file_path.extension().map_or(false, |ext| ext == "json") {
            if let Some(file_name) = file_path.file_stem().and_then(|s| s.to_str()) {
              if file_name == app_name {
                let  result   = process_manifest_file(&file_path, &bucket_path, &app_name );
                match result {
                  Ok(info) => {
                    infos_set.insert(info);
                  },
                  Err(e) => {
                    log::error!("Failed to process file {}: {}", file_path.display(), e);
                  }
                }
              }
            }
          }
        }
      });
    }
  });
  print_pretty_info( infos_set );
  
}

fn display_specific_app_info(app_name: &str, bucket_name: &str, bucket_paths: Vec<String>) {
  if bucket_paths.is_empty() || bucket_name.is_empty() {
    println!("No bucket found.");
    return;
  }
  let  info_set = DashSet::new();
  bucket_paths.par_iter().for_each(|bucket_path| {
    if let Some(bucket) = bucket_path.split('\\').nth(3) {
      if bucket == bucket_name {
        let bucket_path = format!("{}\\bucket", bucket_path);
        if let Ok(entries) = fs::read_dir(&bucket_path) {
          entries.par_bridge().for_each(|entry| {
            if let Ok(file) = entry { 
              let file_type = file.file_type().unwrap();
              let file_path = file.path();
              if file_type.is_file() && file_path.extension().map_or(false, |ext| ext == "json") {
                if let Some(file_name) = file_path.file_stem().and_then(|s| s.to_str()) {
                  if file_name == app_name {
                    let  result   = process_manifest_file(&file_path, &bucket_path, app_name );
                    match result {
                      Ok(info) => {
                        info_set.insert(info);
                        return ; 
                      },
                      Err(e) => {
                        log::error!("Failed to process file {}: {}", file_path.display(), e);
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
  print_pretty_info( info_set);
  
}

fn process_manifest_file(file_path: &std::path::Path, manifest_path: &str, app_name: &str ) -> anyhow::Result<Vec< (String , String ) >> {
  let content = fs::read_to_string(file_path)?;
  let serde_obj: Value = serde_json::from_str(&content)?;

  let description = serde_obj["description"].as_str().unwrap_or_default();
  let version = serde_obj["version"].as_str().unwrap_or_default();
  let bucket_name = manifest_path.split('\\').nth(3).unwrap_or("");
  let website = serde_obj["homepage"].as_str().unwrap_or_default();
  let license = serde_obj["license"].as_str().unwrap_or_default();
  let update_at = get_file_modified_time(file_path.to_str().unwrap_or(""))?;
  let binary = serde_obj["bin"].as_str().unwrap_or_default();

  let short_str = serde_obj["shortcuts"].as_array()
    .map(|arr| arr.iter()
      .filter_map(|shortcut| shortcut.as_array())
      .map(|inner| inner.iter().map(|val| val.to_string()).collect::<Vec<String>>().join(", "))
      .collect::<Vec<String>>()
      .join(" | "))
    .unwrap_or_default();

  let notes = serde_obj["notes"].as_array()
    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>().join("\n\t\t"))
    .unwrap_or_default();

  let info = vec![
    ("Name\t\t".to_string(), app_name.to_string()),
    ("Description\t".to_string(), description.to_string()),
    ("Version\t\t".to_string(), version.to_string()),
    ("Bucket\t\t".to_string(), bucket_name.to_string()),
    ("Website\t\t".to_string(), website.to_string()),
    ("License\t\t".to_string(), license.to_string()),
    ("UpdateAt\t".to_string(), update_at.to_string()),
    ("Binary\t\t".to_string(), binary.to_string()),
    ("Shortcuts\t".to_string(), short_str.to_string()),
    ("Notes\t\t".to_string(), notes.to_string()),
  ];
  
   
  Ok(info)
}


fn   print_pretty_info( info    : DashSet<Vec<(String, String)>>)  {

  let max_key_length = info.iter().flat_map(|vec
  | vec.iter().map(|(key, _)| key.len()).collect::<Vec<_>>()).max().unwrap_or(0);
 let  max_value_length = info.iter().flat_map(|vec
  | vec.iter().map(|(_, value)| value.len()).collect::<Vec<_>>()).max().unwrap_or(0);
  let max_width = max_key_length.max(max_value_length);

  for vec in info {
    for (key, value) in vec {
      if !value.is_empty() {
        println!("{:<width$} : {}", key.green().bold(), value, width = max_key_length + 1);
      }
    }
    println!("{:<width$}" ,  "-".repeat(max_value_length+max_key_length+6).black() ,width=max_value_length + 1 );
  }

}
fn get_file_modified_time(file_path: &str) -> io::Result<String> {
  let metadata = fs::metadata(file_path)?;
  let time = metadata.modified()?;
  let datetime: DateTime<Local> = time.into();
  Ok(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
}
