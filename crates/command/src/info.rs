use crossterm::style::Stylize;
use std::fs;
use std::io;
use chrono::{DateTime, Local};
use serde_json::Value;

pub fn display_app_info(app_name: String, bucket_paths: Vec<String>) {
    if  app_name.contains('/')&&  !app_name.split('/').last().unwrap().is_empty(){
       #[cfg(debug_assertions) ]
        dbg!(&app_name) ;
       let  name  = app_name.split('/').collect::<Vec<&str>>()[1];
       let bucket  = app_name.split('/').collect::<Vec<&str>>()[0];
        log::info!("name: {}", &name);
        log::info!("bucket: {}", &bucket);
       display_specific_app_info(name, bucket , bucket_paths );
        return ;
    }
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
              let   mut  short   =  Vec::new();
              static EMPTY_ARRAY: Vec<Value> = Vec::new();
              let  notes = serde_obj["notes"].as_array().unwrap_or_else(|| &EMPTY_ARRAY);
              let  note = notes.iter().map(|v| v.as_str().unwrap_or_default()).
                collect::<Vec<&str>>().join("\n\t\t");
              if let Some(shortcuts) = serde_obj.get("shortcuts") {
                if let Some(array) = shortcuts.as_array() {
                  for shortcut in array {
                    if let Some(inner_array) = shortcut.as_array() {
                      short.push(inner_array);
                    }
                  }
                }
              }
              if let Some(shortcuts) = serde_obj.get("shortcuts") {
                if let Some(array) = shortcuts.as_array() {
                  for shortcut in array {
                    if let Some(inner_array) = shortcut.as_array() {
                      short.push(inner_array);
                    }
                  }
                }
              }

              let short_str = short.iter()
                .map(|v| v.iter().map(|val| val.to_string()).collect::<Vec<String>>().join(", "))
                .collect::<Vec<String>>()
                .join(" | ");
               let  info = vec![
                ("Name\t\t".to_string(), app_name),
                ("Description\t".to_string(), description.to_string()),
                ("Version\t\t".to_string(), version.to_string()),
                ("Bucket\t\t".to_string(), bucket_name.to_string()),
                ("Website\t\t".to_string(), website.to_string()),
                ("License\t\t".to_string(), license.to_string()),
                ("UpdateAt\t".to_string(),  update_at.to_string()),
                ("Binary\t\t".to_string(), binary.to_string()),
                ("Shortcuts\t".to_string(), short_str.to_string()),
                ("Notes\t\t".to_string(), note.to_string()),
              ];
              let max_key_length = info.iter().map(|(key, _)| key.len()).max().unwrap_or(0);
              for (key, value) in info {
                if value.is_empty()  {
                  continue;
                }
                println!("{:<width$} : {}", key.green().bold(), value, width = max_key_length + 1);
              }
                return  ;
            }
        }
    }
}

fn display_specific_app_info(app_name : &str, bucket_name  : &str, bucket_paths: Vec<String>) {
     if  bucket_paths.is_empty()  || bucket_name.is_empty() {
        println!("No bucket found.");
        return ;
    }
     for bucket_path in bucket_paths.iter() {
        let  bucket = bucket_path.split('\\').collect::<Vec<&str>>()[3];
       if bucket != bucket_name { continue;  }
       let bucket_path = bucket_path.clone() + "\\bucket";
       for file in std::fs::read_dir(&bucket_path).unwrap() {
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

             let  website = serde_obj["homepage"].as_str().unwrap_or_default();
             let  license = serde_obj["license"].as_str().unwrap_or_default();
             let  update_at = get_file_modified_time(&file_str).unwrap();
             let   binary    = serde_obj["bin"] .as_str().unwrap_or_default();
             let   mut  short   =  Vec::new();
             static EMPTY_ARRAY: Vec<Value> = Vec::new();
             let  notes = serde_obj["notes"].as_array().unwrap_or_else(|| &EMPTY_ARRAY);
             let  note = notes.iter().map(|v| v.as_str().unwrap_or_default()).
               collect::<Vec<&str>>().join("\n\t\t");
             if let Some(shortcuts) = serde_obj.get("shortcuts") {
               if let Some(array) = shortcuts.as_array() {
                 for shortcut in array {
                   if let Some(inner_array) = shortcut.as_array() {
                     short.push(inner_array);
                   }
                 }
               }
             }

             let short_str = short.iter()
               .map(|v| v.iter().map(|val| val.to_string())
                 .collect::<Vec<String>>().join(", "))
               .collect::<Vec<String>>()
               .join(" | ");
             let  info = vec![
               ("Name\t\t".to_string(), app_name.to_string() ),
               ("Description\t".to_string(), description.to_string()),
               ("Version\t\t".to_string(), version.to_string()),
               ("Bucket\t\t".to_string(), bucket_name.to_string()),
               ("Website\t\t".to_string(), website.to_string()),
               ("License\t\t".to_string(), license.to_string()),
               ("UpdateAt\t".to_string(),  update_at.to_string()),
               ("Binary\t\t".to_string(), binary.to_string()),
               ("Shortcuts\t".to_string(), short_str.to_string()),
               ("Notes\t\t".to_string(), note.to_string()),
             ];
             let max_key_length = info.iter().map(|(key, _)| key.len()).max().unwrap_or(0);
             for (key, value) in info {
               if value.is_empty()  {
                 continue;
               }
               println!("{:<width$} : {}", key.green().bold(), value, width = max_key_length + 1);
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


