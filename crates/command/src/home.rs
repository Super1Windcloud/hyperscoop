use crate::utils::utility::is_valid_url;
use webbrowser;

pub fn _open_home_page(bucket_paths: Vec<String>, name: String) {
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
                let url = serde_obj["homepage"].as_str().unwrap_or_default();
                #[cfg(debug_assertions)]
                dbg!(url);
                webbrowser::open(url).expect("Failed to open page");
                return;
            }
        }
    }
}

pub fn open_home_page(bucket_paths: Vec<String>, name: String) -> anyhow::Result<()> {
    let found = bucket_paths.iter().find_map(|bucket_path| {
        let manifest_path = format!("{}\\bucket", bucket_path);
        log::info!("manifest_path: {}", manifest_path);

        std::fs::read_dir(manifest_path)
            .ok()?
            .filter_map(|entry| {
                let file = entry.ok()?.path();
                if !file.is_file() || !file.to_string_lossy().ends_with(".json") {
                    return None;
                }

                let file_name = file.file_stem()?.to_str()?;
                if file_name != name {
                    return None;
                }

                let content = std::fs::read_to_string(&file).ok()?;
                let serde_obj = serde_json::from_str::<serde_json::Value>(&content).ok()?;
                let url = serde_obj["homepage"].as_str().unwrap_or_default();

                if !url.is_empty() && is_valid_url(&url) {
                    webbrowser::open(url).ok()?;
                }
                Some(())
            })
            .next()
    });
    if found.is_some() {
        std::process::exit(0);
    }
    Ok(())
}
