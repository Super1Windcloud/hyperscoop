use webbrowser;

pub fn open_home_page(bucket_paths: Vec<String>, name: String) {
    for bucket_path in bucket_paths.iter() {
        let manifest_path = bucket_path.clone() + "\\bucket";
        log::info!("manifest_path: {}", manifest_path);
        for file in std::fs::read_dir(manifest_path).unwrap() {
            let file = file.unwrap().path();
            let file_str = file.as_path().display().to_string();
            if file.is_file() && file_str.ends_with(".json") {
                let file_name = file.file_stem().unwrap().to_str().unwrap();
                let file_name = file_name.split('/').last().unwrap();
                if file_name != name {
                    continue;
                }
                let content = std::fs::read_to_string(file).unwrap();
                let serde_obj = serde_json::from_str::<serde_json::Value>(&content).unwrap();
                let url = serde_obj["homepage"].as_str().unwrap();
                #[cfg(debug_assertions)]
                dbg!(url);
                webbrowser::open(url).expect("Failed to open page");

                return;
            }
        }
    }
}


// pub fn _open_home_page(bucket_paths: Vec<String>, name: String) {
//   use rayon::prelude::*;
// 
//   bucket_paths.par_iter().find_map_any(|bucket_path| {
//     let manifest_path = bucket_path.clone() + "\\bucket";
//     log::info!("manifest_path: {}", manifest_path);
// 
//     let dir_entries = match std::fs::read_dir(&manifest_path) {
//       Ok(entries) => entries,
//       Err(_) => return None,
//     };
// 
//     dir_entries.filter_map(|entry| { 
//       let  entry = entry.unwrap();
//       let file_type =&entry.file_type().unwrap();
//       let file  = entry.path();
//       let file_str = file.as_path().display().to_string();
//       if  file_type.is_file() && file_str.ends_with(".json") {
//         let file_name = match file.file_stem().and_then(|s| s.to_str()) {
//           Some(name) => name.split('/').last().unwrap(),
//           None => return None,
//         };
// 
//         if file_name != name {
//           return None;
//         }
// 
//         match std::fs::read_to_string(&file) {
//           Ok(content) => {
//             match serde_json::from_str::<serde_json::Value>(&content) {
//               Ok(serde_obj) => serde_obj["homepage"].as_str().map(|s| s.to_string()),
//               Err(_) => None,
//             }
//           }
//           Err(_) => None,
//         }
//       } else {
//         None
//       }
//     }).find_any (|url| {
//       if let Some(url) = url {
//         #[cfg(debug_assertions)]
//         dbg!(url);
//         webbrowser::open(url).is_ok()
//       } else {
//         false
//       }
//     })
//   });
// }
