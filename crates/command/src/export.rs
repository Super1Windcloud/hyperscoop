use crate::buckets::get_buckets_path;
use crate::init_hyperscoop;
use chrono::{DateTime, Utc};
use git2::Repository;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{metadata, read_dir};
use std::io::Write;
use std::path::Path;

pub fn export_config_to_path(file_name: String) {
    let path = std::path::Path::new(&file_name);
    let mut file = std::fs::File::create(path).expect("路径错误无法创建文件");

    log::info!("导出配置文件到 {}", file_name);
  let bucket_config = get_all_buckets_info();
  let apps = get_all_installed_apps();
  let json_data = json!({
        "buckets": bucket_config  ,
         "apps"  : apps
    });
  let pretty_json = serde_json::to_string_pretty(&json_data).unwrap();
  file.write_all(pretty_json.as_bytes()).unwrap();
}
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BucketInfo {
    Name: String,
    Source: String,
    Updated: String,
    Manifests: u32,
}

impl BucketInfo {
    fn new(name: String, source: String, updated: String, manifests: u32) -> BucketInfo {
        BucketInfo {
            Name: name,
            Source: source,
            Updated: updated,
            Manifests: manifests,
        }
    }
}
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstalledApp {
    Updated: String,
    Version: String,
    Source: String,
    Name: String,
}
impl InstalledApp {
    fn new(name: String, source: String, updated: String, version: String) -> InstalledApp {
        InstalledApp {
            Name: name,
            Source: source,
            Updated: updated,
            Version: version,
        }
    }
}
pub fn export_config_to_current_dir(file_name: String) {
    let path = std::path::Path::new(&file_name);
    let mut file = std::fs::File::create(path).expect("路径错误无法创建文件");
    let current_dir = std::env::current_dir().unwrap();
    let path = current_dir.join(path);
    log::info!("导出配置文件到 {}", path.display());
    let bucket_config = get_all_buckets_info();
    let apps = get_all_installed_apps();
    let json_data = json!({
        "buckets": bucket_config  ,
         "apps"  : apps
    });
    let pretty_json = serde_json::to_string_pretty(&json_data).unwrap();
    file.write_all(pretty_json.as_bytes()).unwrap();
}

fn get_all_installed_apps() -> Vec<InstalledApp> {
    let app_path = init_hyperscoop().unwrap();
    let apps_path = Path::new(&app_path.apps_path);
    let mut installed_apps: Vec<InstalledApp> = Vec::new();
    for entry in read_dir(apps_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap();
        let app_path = path.join("current");
        if !app_path.is_dir() {
            continue;
        }
        let install_file = app_path.join("install.json");
        if !install_file.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(install_file).unwrap();
        let install_info = serde_json::from_str::<serde_json::Value>(&content).unwrap();
        let source = install_info["bucket"].as_str().unwrap();
        let manifest_file = app_path.join("manifest.json");
        if !manifest_file.is_file() {
            continue;
        }
        let _ =
            rust_file_encode_mode_convert::translate_all_encoded_mode_file_to_utf8(&manifest_file);
      let content = std::fs::read_to_string(&manifest_file).unwrap();
       
        let content =
            serde_json::from_str::<serde_json::Value>(&content).unwrap_or_default();
        let version = content["version"].as_str().unwrap_or("unknown"); 
        let updated = get_repo_updated(&path);
        installed_apps.push(InstalledApp::new(
            name.to_string(),
            source.to_string(),
            updated,
            version.to_string(),
        ));
    }
    return installed_apps;
}

fn get_all_buckets_info() -> Vec<BucketInfo> {
    let bucket_path = get_buckets_path().unwrap();
    let mut bucket_info_list: Vec<BucketInfo> = Vec::new();
    for bucket_dir in bucket_path {
        let path = Path::new(&bucket_dir);
        let bucket_name = path.file_name().unwrap().to_str().unwrap();
        let bucket_source = get_repo_url(path);
        let bucket_updated = get_repo_updated(path);
        log::info!("bucket_name: {}", bucket_name);
        log::info!("bucket_source: {}", bucket_source);
        log::info!("bucket_updated: {}", bucket_updated);
        let manfiests_count = get_manifests_count(path);
        log::info!("manifests_count: {}", manfiests_count);
        let bucket_info = BucketInfo::new(
            bucket_name.to_string(),
            bucket_source,
            bucket_updated,
            manfiests_count,
        );
        bucket_info_list.push(bucket_info);
    }
    return bucket_info_list;
}

fn get_manifests_count(path: &Path) -> u32 {
    let mut count = 0;
    let path = path.join("bucket");
    if path.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            if path.is_file() {
                let extension = path.extension();
                if extension.is_none() {
                    log::warn!("文件 {} 没有扩展名", path.display());
                    continue;
                }
                let extension = extension.unwrap().to_str().unwrap();
                if extension == "json" {
                    count += 1;
                }
            }
        }
    }
    return count;
}
fn get_repo_updated(path: &Path) -> String {
    let metadatas = metadata(path).unwrap();
    let modified_time = metadatas.modified().unwrap();
    let datetime: DateTime<Utc> = DateTime::from(modified_time);
    let formatted_time = datetime.format("%Y-%m-%dT%H:%M:%S%z").to_string();
    return formatted_time;
}

fn get_repo_url(path: &Path) -> String {
    let repo = Repository::open(path).unwrap();
    let remote = repo.find_remote("origin").unwrap();
    let url = remote.url().unwrap_or("No remote URL found").to_string();
    url
}

pub fn export_config_to_path_width_config(file_name: String) {
  let path = std::path::Path::new(&file_name);
  let mut file = std::fs::File::create(path).expect("路径错误无法创建文件");
  log::info!("导出配置文件到 {}", path.display());
  let bucket_config = get_all_buckets_info();
  let apps = get_all_installed_apps();
  let config = get_scoop_config_info();
  let value = serde_json::from_str::<serde_json::Value>(&config).unwrap();
  let json_data = json!({
        "buckets": bucket_config  ,
         "apps"  : apps 
      , "config" : value
    });
  let pretty_json = serde_json::to_string_pretty(&json_data).unwrap();
  file.write_all(pretty_json.as_bytes()).unwrap();
}

fn get_scoop_config_info() -> String  {
   let config_path = std::env::var("USERPROFILE").unwrap_or("".to_string());
   let config_path = Path::new(&config_path).join(".config\\scoop\\config.json"); 
  log::info!("config_path: {}", config_path.display()); 
   let content = std::fs::read_to_string(config_path).unwrap();
   return content;
}

pub fn export_config_to_current_dir_with_config(file_name: String) {
  let path = std::path::Path::new(&file_name);
  let mut file = std::fs::File::create(path).expect("路径错误无法创建文件");
  let current_dir = std::env::current_dir().unwrap();
  let path = current_dir.join(path);
  log::info!("导出配置文件到 {}", path.display());
  let bucket_config = get_all_buckets_info();
  let apps = get_all_installed_apps();
  let config = get_scoop_config_info(); 
  let value = serde_json::from_str::<serde_json::Value>(&config).unwrap(); 
  let json_data = json!({
        "buckets": bucket_config  ,
         "apps"  : apps 
      , "config" : value
    });
  let pretty_json = serde_json::to_string_pretty(&json_data).unwrap();
  file.write_all(pretty_json.as_bytes()).unwrap();
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;
    #[test]
    fn test_transform_file_to_utf8() {
        let dir = current_dir().unwrap();
        let file_path = dir.join("src\\bin\\test.rs");
        dbg!(&file_path);
        let _ = rust_file_encode_mode_convert::translate_all_encoded_mode_file_to_utf8(&file_path);
    }
}
