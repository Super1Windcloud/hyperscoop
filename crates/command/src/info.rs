use crossterm::style::Stylize;
use std::fs;
use std::io;
use std::time::SystemTime;
use chrono::{DateTime, Local};

pub fn display_app_info(app_name: String, bucket_paths: Vec<String>) {
    for bucket_path in bucket_paths.iter() {
        let manifest_path = bucket_path.clone() + "\\bucket";
        log::info!("manifest_path: {}", &manifest_path);
        for file in std::fs::read_dir(&manifest_path).unwrap() {
            let file = file.unwrap().path();
            let file_str = file.as_path().display().to_string();
            if file.is_file() && file_str.ends_with(".json") {
                let file_name = file.file_stem().unwrap().to_str().unwrap();
                let file_name = file_name.split('/').last().unwrap();
                if file_name != app_name {
                    continue;
                }
                let content = std::fs::read_to_string(file).unwrap();
                let serde_obj = serde_json::from_str::<serde_json::Value>(&content).unwrap();
                let description = serde_obj["description"].as_str().unwrap_or_default();
                let version = serde_obj["version"].as_str().unwrap_or_default();
                let path  : Vec<&str>  = manifest_path.split('\\').collect();
                let bucket_name = path[3] ;
                log::info!("bucket_name: {}   ", &bucket_name  );
              let  website = serde_obj["homepage"].as_str().unwrap_or_default();
              let  license = serde_obj["license"].as_str().unwrap_or_default();
              let  update_at = get_file_modified_time(&file_str).unwrap();
              let   binary    = serde_obj["bin"] .as_str().unwrap_or_default();
              let   shortcut  = serde_obj["shortcuts"] . as_str().unwrap_or_default();
              let  note = serde_obj["note"].as_str().unwrap_or_default();
 
               let  info = vec![
                ("Name".to_string(), app_name),
                ("Description".to_string(), description.to_string()),
                ("Version".to_string(), version.to_string()),
                ("Bucket".to_string(), bucket_name.to_string()),
                ("Website".to_string(), website.to_string()),
                ("License".to_string(), license.to_string()),
                ("UpdateAt".to_string(),  update_at.to_string()),
                ("Binary".to_string(), binary.to_string()),
                ("Shortcuts".to_string(), shortcut.to_string()),
                ("Notes".to_string(), note.to_string()),
              ];
              let max_key_length = info.iter().map(|(key, _)| key.len()).max().unwrap_or(0);
              for (key, value) in info {
                println!("{:<width$} : {}", key.green().bold(), value, width = max_key_length);
              }
                return  ;
            }
        }
    }
}

fn get_file_modified_time(file_path: &str) -> io::Result<String > {
  let metadata = fs::metadata(file_path)?;
  let time  =  metadata.modified() .expect("Unable to get modified time");
  let datetime: DateTime<Local> = time.into();
  Ok(datetime .format("%Y-%m-%d %H:%M:%S").to_string())

}


