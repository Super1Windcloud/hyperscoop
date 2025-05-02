use crate::command_args::status::StatusArgs;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};
use command_util_lib::init_env::{
    get_apps_path, get_apps_path_global, get_buckets_root_dir_path,
    get_buckets_root_dir_path_global,
};
use command_util_lib::list::VersionJSON;
use rayon::prelude::*;
use std::collections::HashMap;

pub fn execute_status_command(status_args: StatusArgs) -> Result<(), anyhow::Error> {
    let apps_path = if status_args.global {
        get_apps_path_global()
    } else {
        get_apps_path()
    };
    let bucket_path = if status_args.global {
        get_buckets_root_dir_path_global()
    } else {
        get_buckets_root_dir_path()
    };

    let mut current_versions = Vec::new();
    let mut installed_apps = Vec::new();
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
            current_versions.push("Not Install Correctly".to_string());
            installed_apps.push(app_name.to_string());
            continue;
        }
        let manifest = std::fs::read_to_string(manifest_path)?;
        let manifest: VersionJSON = serde_json::from_str(&manifest)?;
        let current_version = manifest.version.unwrap_or("Not Found".to_string());
        current_versions.push(current_version.to_string());
        installed_apps.push(app_name.to_string());
    }
    let install_apps = installed_apps.as_slice();
    let str_slices: Vec<&str> = install_apps.iter().map(|s| s.as_str()).collect();
    let version_map = build_version_map(bucket_path, str_slices.as_slice())?;
    let latest_versions: Vec<String> = install_apps
        .iter()
        .map(|app_name| {
            version_map
                .get(app_name.to_lowercase().as_str())
                .cloned()
                .unwrap_or_else(|| "Not Found".to_string())
        })
        .collect();
    // let max_installed_len = current_versions.iter().map(|s| s.len()).max().unwrap_or(0) + 4;
    // let max_latest_len = latest_versions.iter().map(|s| s.len()).max().unwrap_or(0) + 4;
    // let max_name_len = install_apps.iter().map(|s| s.len()).max().unwrap_or(0) + 4;
    //
    // let name_interval = max_name_len - 4;
    // let installed_interval = max_installed_len - 9;
    // let latest_interval = max_latest_len - 6;
    let mut final_installed_apps = Vec::new();
    for (app_name, (current_version, latest_version)) in install_apps
        .iter()
        .zip(current_versions.iter().zip(latest_versions.iter()))
    {
        if latest_version > current_version {
            final_installed_apps.push(vec![
                app_name.to_string(),
                current_version.to_string(),
                latest_version.to_string(),
                "Yes".to_string(),
            ]);
        }
    }
    display_status_information(final_installed_apps.as_slice());

    Ok(())
}

fn display_status_information(install_apps: &[Vec<String>]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("AppName")
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
            Cell::new("InstalledVersion")
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
            Cell::new("LatestVersion")
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
            Cell::new("NeedUpdate")
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
        ])
        .add_rows(install_apps.as_ref());
    let column = table.column_mut(3).expect("Our table has three columns");
    column.set_cell_alignment(CellAlignment::Center);
    println!("{}", table);
}

fn build_version_map(
    bucket_path: String,
    installed_app_name: &[&str],
) -> Result<HashMap<String, String>, anyhow::Error> {
    let mut version_map = HashMap::<String, String>::new();
    let installed_app_name = installed_app_name
        .iter()
        .map(|app| app.to_lowercase())
        .collect::<Vec<_>>();
    let _ = std::fs::read_dir(bucket_path)?
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
            if path.extension().unwrap_or_default() != "json" {
                return None;
            }
            let file_name = path.file_stem()?.to_str().unwrap().to_lowercase();
            if !installed_app_name.contains(&file_name) {
                return None;
            }
            let manifest_str = std::fs::read_to_string(&path).ok()?;
            let manifest: VersionJSON = serde_json::from_str(&manifest_str).ok()?;

            let version_str = manifest.version?;
            if version_str == "nightly" || version_str == "latest" {
                return None;
            }
            Some((file_name, version_str))
        })
        .collect::<Vec<_>>() // 并行收集后统一处理，避免并发写 HashMap
        .into_iter()
        .for_each(|(file_name, version)| {
            version_map
                .entry(file_name)
                .and_modify(|v| {
                    if &version > v {
                        *v = version.clone();
                    }
                })
                .or_insert(version);
        });
    let version_map = version_map
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect();
    Ok(version_map)
}

mod test_version_map {

    #[test]
    fn test_build_version_map() {
        use crate::hyperscoop_middle::invoke_status::build_version_map;
        use command_util_lib::init_env::get_buckets_root_dir_path;
        let bucket_name = get_buckets_root_dir_path();
        let installed_app_name = vec!["vcpkg", "7zip", "xshell", "scrcpy", "shotcut"];
        let version_map = build_version_map(bucket_name, installed_app_name.as_slice()).unwrap();
        dbg!(&version_map);
    }
}
