use crate::buckets::get_buckets_path;
use crate::init_env::{get_apps_path, get_scoop_cfg_path};
use anyhow::{bail, Context};
use chrono::{DateTime, Utc};
use git2::Repository;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{metadata, read_dir};
use std::io::Write;
use std::path::Path;

pub fn export_config_to_path(file_name: String) -> anyhow::Result<()> {
    let path = Path::new(&file_name);
    let mut file = std::fs::File::create(path)
        .with_context(|| format!("Failed to create file {} at line 15", file_name))?;

    let bucket_config = get_all_buckets_info()?;
    let apps = get_all_installed_apps()?;
    let json_data = json!({
        "buckets": bucket_config  ,
         "apps"  : apps
    });
    let pretty_json = serde_json::to_string_pretty(&json_data)
        .context("Failed to serialize JSON data into JSON at lin 25")?;
    file.write_all(pretty_json.as_bytes())
        .context("Failed to write to JSON file a line 27")?;
    println!("成功导出配置文件到 {}", path.display());

    Ok(())
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
pub fn export_config_to_current_dir(file_name: String) -> anyhow::Result<()> {
    let path = Path::new(&file_name);
    let mut file = std::fs::File::create(path).expect("路径错误无法创建文件");
    let current_dir = std::env::current_dir()?;
    let path = current_dir.join(path);
    let bucket_config = get_all_buckets_info()?;
    let apps = get_all_installed_apps()?;
    let json_data = json!({
        "buckets": bucket_config  ,
         "apps"  : apps
    });
    let pretty_json = serde_json::to_string_pretty(&json_data)
        .context("Failed to serialize JSON data into JSON at lin 81")?;
    file.write_all(pretty_json.as_bytes())
        .context("Failed to write to JSON file a line 27")?;

    println!("成功导出配置文件到 {}", path.display());

    Ok(())
}

fn get_all_installed_apps() -> anyhow::Result<Vec<InstalledApp>> {
    let apps_root_dir = get_apps_path();
    let mut installed_apps: Vec<InstalledApp> = Vec::new();

    for entry in read_dir(&apps_root_dir)
        .with_context(|| format!("Failed to read directory :{} at line 94", apps_root_dir))?
    {
        let entry = entry.context("Failed to read entry at 99")?;
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
        let content = std::fs::read_to_string(install_file)
            .context("Failed to read install file at line 113")?;
        let install_info = serde_json::from_str::<serde_json::Value>(&content)
            .context("Failed to deserialize install file at line 115")?;
        let source = install_info["bucket"].as_str().unwrap();
        let manifest_file = app_path.join("manifest.json");
        if !manifest_file.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(&manifest_file)
            .with_context(|| "Failed to read manifest file at line 122".to_string())?;

        let content = serde_json::from_str::<serde_json::Value>(&content)
            .context("Failed to deserialize manifest file at line 125")?;
        let version = content["version"].as_str().unwrap_or("unknown");
        let updated = get_repo_updated(&path)?;
        installed_apps.push(InstalledApp::new(
            name.to_string(),
            source.to_string(),
            updated,
            version.to_string(),
        ));
    }
    Ok(installed_apps)
}

fn get_all_buckets_info() -> anyhow::Result<Vec<BucketInfo>> {
    let bucket_path = get_buckets_path()?;

    let mut bucket_info_list: Vec<BucketInfo> = Vec::new();
    for bucket_dir in bucket_path {
        let path = Path::new(&bucket_dir);
        let bucket_name = path.file_name().unwrap().to_str().unwrap();
        let bucket_source = get_repo_url(path)?;
        let bucket_updated = get_repo_updated(path)?;
        let manfiests_count = get_manifests_count(path)?;

        let bucket_info = BucketInfo::new(
            bucket_name.to_string(),
            bucket_source,
            bucket_updated,
            manfiests_count,
        );
        bucket_info_list.push(bucket_info);
    }
    Ok(bucket_info_list)
}

fn get_manifests_count(path: &Path) -> anyhow::Result<u32> {
    let path = path.join("bucket");
    if path.is_dir() {
        // read_dir can't use into_iter
        let files = path
            .read_dir()
            .context("Failed to read bucket directory at line 163")?
            .par_bridge()
            .collect::<Vec<_>>();

        let count = files.iter().count();
        Ok(count as u32)
    } else {
        bail!("Failed to read bucket directory at line 168")
    }
}

fn get_repo_updated(path: &Path) -> anyhow::Result<String> {
    let metadatas = metadata(path).context("Failed to read metadata at line 194".to_string())?;
    let modified_time = metadatas
        .modified()
        .context("Failed to read metadata modified at line 196")?;
    let datetime: DateTime<Utc> = DateTime::from(modified_time);
    let formatted_time = datetime.format("%Y-%m-%dT%H:%M:%S%z").to_string();
    Ok(formatted_time)
}

fn get_repo_url(path: &Path) -> anyhow::Result<String> {
    let repo = Repository::open(path).context("Failed to open repository at line 197")?;
    let remote = repo
        .find_remote("origin")
        .context("Failed to find remote origin")?;
    let url = remote.url().unwrap_or("No remote URL found").to_string();
    Ok(url)
}

pub fn export_config_to_path_width_config(file_name: String) -> anyhow::Result<()> {
    let path = Path::new(&file_name);
    let mut file = std::fs::File::create(path).expect("路径错误无法创建文件");
    let bucket_config = get_all_buckets_info()?;
    let apps = get_all_installed_apps()?;
    let config = get_scoop_config_info()?;
    let value = serde_json::from_str::<serde_json::Value>(&config)
        .context("Failed to deserialize JSON config into JSON at lin 212")?;
    let json_data = json!({
        "buckets": bucket_config  ,
         "apps"  : apps
      , "config" : value
    });
    let pretty_json = serde_json::to_string_pretty(&json_data)
        .context("Failed to  convert JSON data to pretty JSON at lin 219")?;
    file.write_all(pretty_json.as_bytes())
        .context("Failed to write to JSON file a line 221")?;
    println!("成功导出配置文件到 {}", path.display());

    Ok(())
}

fn get_scoop_config_info() -> anyhow::Result<String> {
    let config_path = get_scoop_cfg_path();
    log::info!("config_path: {}", config_path);
    let content = std::fs::read_to_string(config_path)
        .context("Failed to read scoop config file at line 232")?;
    Ok(content)
}

pub fn export_config_to_current_dir_with_config(file_name: String) -> anyhow::Result<()> {
    let path = Path::new(&file_name);
    let mut file = std::fs::File::create(path).expect("路径错误无法创建文件");
    let current_dir = std::env::current_dir()?;
    let path = current_dir.join(path);
    let bucket_config = get_all_buckets_info()?;
    let apps = get_all_installed_apps()?;
    let config = get_scoop_config_info()?;
    let value = serde_json::from_str::<serde_json::Value>(&config)
        .context("Failed to deserialize JSON config into JSON at lin 245")?;
    let json_data = json!({
        "buckets": bucket_config  ,
         "apps"  : apps
      , "config" : value
    });
    let pretty_json = serde_json::to_string_pretty(&json_data)
        .context("Failed to  convert JSON data to pretty JSON at lin 252")?;
    file.write_all(pretty_json.as_bytes())
        .context("Failed to write to JSON file a line 254")?;

    println!("成功导出配置文件到 {}", path.display());

    Ok(())
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
