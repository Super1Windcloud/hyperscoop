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
