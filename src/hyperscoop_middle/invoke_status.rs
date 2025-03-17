use command_util_lib::init_hyperscoop;
use crossterm::style::Stylize;
use rayon::prelude::*;
use std::collections::HashMap;
pub fn execute_status_command() -> Result<(), anyhow::Error> {
    let hyperscoop = init_hyperscoop()?;
    let apps_path = hyperscoop.apps_path.clone();
    let bucket_path = hyperscoop.bucket_path.clone();
    let mut current_versions = Vec::new();
    let mut installed_apps = Vec::new();
    let version_map = build_version_map(bucket_path)?;
    for app_path in std::fs::read_dir(apps_path)? {
        let app_path = app_path?.path();
        let app_name = app_path
            .file_name()
            .expect("Invalid app path")
            .to_str()
            .unwrap();
        let current = app_path.join("current");
        let manifest_path = current.join("manifest.json");

        if !manifest_path.exists() {
            continue;
        }
        let manifest = std::fs::read_to_string(manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest).unwrap_or_default();
        let current_version = manifest["version"].as_str().unwrap_or_default();
        current_versions.push(current_version.to_string());
        installed_apps.push(app_name.to_string());
    }
    let latest_versions: Vec<String> = installed_apps
        .iter()
        .map(|app_name| {
            version_map
                .get(app_name.to_lowercase().as_str())
                .cloned()
                .unwrap_or_else(|| "Not Found".to_string())
        })
        .collect();
    let max_len = current_versions.iter().map(|s| s.len()).max().unwrap_or(0);
    let width = max_len + 10;

    let mut executed = false;
    for ((current_version, app_name), latest_version) in current_versions
        .iter()
        .zip(installed_apps.iter())
        .zip(latest_versions.iter())
    {
        if executed == false {
            println!(
                "{:<width$} {:<width$}{:<width$} {:<width$}",
                "Name\t\t\t\t".green().bold(),
                "Installed Version\t\t\t".green().bold(),
                "Latest Version\t\t\t".green().bold(),
                "Need Update\t\t\t".green().bold(),
                width = width
            );

            println!(
                "{:<width$} {:<width$}{:<width$} {:<width$}",
                "____\t\t\t\t".green().bold(),
                "_________________\t\t\t".green().bold(),
                "_______________\t\t\t".green().bold(),
                "_____________\t\t".green().bold(),
                width = width
            );
        }
        executed = true;
        if latest_version > current_version {
            println!(
                "{:<width$} {:<width$} {:<width$} {:<width$}",
                app_name,
                current_version,
                latest_version,
                "Yes",
                width = width
            );
        } else {
            println!(
                "{:<width$} {:<width$} {:<width$} {:<width$}",
                app_name,
                current_version,
                latest_version,
                "No",
                width = width
            );
        }
    }
    Ok(())
}

fn build_version_map(bucket_path: String) -> Result<HashMap<String, String>, anyhow::Error> {
  let version_map: HashMap<String, String> = std::fs::read_dir(bucket_path)?
    .par_bridge()
    .filter_map(|bucket| {
      let bucket = bucket.ok()?.path().join("bucket");
      let entries: Vec<_> = std::fs::read_dir(bucket).ok()?.collect();
      Some(entries)
    })
    .flatten()
    .filter_map(|entry| {
      let entry = entry.ok()?;
      let path = entry.path();
      if     path.extension().unwrap_or_default() != "json" {
        return None;
      }

      let file_name = path.file_stem()?.to_str()?.to_lowercase();
      let manifest = std::fs::read_to_string(&path).ok()?;
      let manifest: serde_json::Value = serde_json::from_str(&manifest).ok()?;
      let version = manifest["version"].as_str()?.to_string();

      Some((file_name, version))
    })
    .collect();

  Ok(version_map)
}
