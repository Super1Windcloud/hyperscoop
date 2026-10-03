use crate::command_args::status::StatusArgs;
use crate::i18n::tr;
use anyhow::Context;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};
use command_util_lib::init_env::{
    get_apps_path, get_apps_path_global, get_buckets_root_dir_path,
    get_buckets_root_dir_path_global, is_app_held_by_path,
};
use command_util_lib::list::VersionJSON;
use command_util_lib::utils::utility::compare_versions;
use crossterm::style::Stylize;
use rayon::prelude::*;
use std::cmp::Ordering;
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
    let mut held_status = Vec::new();
    for app_path in std::fs::read_dir(apps_path).context("Failed to read apps directory")? {
        let app_path = app_path.context("Failed to read app directory")?.path();
        let app_name = app_path
            .file_name()
            .expect("Invalid app path")
            .to_str()
            .unwrap();
        if app_name == "scoop" {
            continue;
        }
        if !status_args.app_names.is_empty()
            && !status_args
                .app_names
                .iter()
                .any(|q| q.eq_ignore_ascii_case(app_name))
        {
            continue;
        }
        let current = app_path.join("current");
        let manifest_path = current.join("manifest.json");
        let install_json_path = current.join("install.json");

        let is_held = install_json_path.exists()
            && is_app_held_by_path(install_json_path.to_str().unwrap_or_default());
        held_status.push(is_held);

        if !manifest_path.exists() {
            current_versions.push(tr("Not Installed Correctly", "未正确安装").to_string());
            installed_apps.push(app_name.to_string());
            continue;
        }
        let manifest =
            std::fs::read_to_string(manifest_path).context("Failed to read manifest.json")?;
        let manifest: VersionJSON = serde_json::from_str(&manifest)
            .context("Failed to parse manifest.json to VersionJSON")?;
        let current_version = manifest
            .version
            .unwrap_or_else(|| tr("Not Found", "未找到").to_string());
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
                .unwrap_or_else(|| tr("Not Found", "未找到").to_string())
        })
        .collect();

    let mut final_installed_apps = Vec::new();
    for (i, (app_name, (current_version, latest_version))) in install_apps
        .iter()
        .zip(current_versions.iter().zip(latest_versions.iter()))
        .enumerate()
    {
        if latest_version != &tr("Not Found", "未找到").to_string()
            && current_version != &tr("Not Installed Correctly", "未正确安装").to_string()
            && compare_versions(latest_version.clone(), current_version.clone())
                == Ordering::Greater
        {
            let is_held = held_status.get(i).copied().unwrap_or(false);
            let status_text = if is_held {
                tr("🔒 Held (Skip)", "🔒 已锁定 (跳过)").to_string()
            } else {
                tr("Yes", "是").to_string()
            };
            final_installed_apps.push(vec![
                app_name.to_string(),
                current_version.to_string(),
                latest_version.to_string(),
                status_text,
            ]);
        }
    }

    if status_args.quiet {
        for row in &final_installed_apps {
            println!("{}", row[0]);
        }
        return Ok(());
    }

    if status_args.json {
        let json_list: Vec<serde_json::Value> = final_installed_apps
            .iter()
            .map(|row| {
                serde_json::json!({
                    "name": &row[0],
                    "installed_version": &row[1],
                    "latest_version": &row[2],
                    "held": row[3].contains("Held") || row[3].contains("锁定"),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_list)?);
        return Ok(());
    }

    display_status_information(final_installed_apps.as_slice());

    Ok(())
}

fn display_status_information(install_apps: &[Vec<String>]) {
    if install_apps.is_empty() {
        println!(
            "{}",
            tr(
                "🎉 All installed apps are up-to-date!",
                "🎉 所有已安装的应用均已是最新版本！"
            )
            .dark_green()
            .bold()
        );
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new(tr("AppName", "应用名称"))
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
            Cell::new(tr("InstalledVersion", "已安装版本"))
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
            Cell::new(tr("LatestVersion", "最新版本"))
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
            Cell::new(tr("Status", "状态"))
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
        ])
        .add_rows(install_apps.as_ref());
    let column = table.column_mut(3).expect("Our table has four columns");
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
    let Ok(bucket_entries) = std::fs::read_dir(&bucket_path) else {
        return Ok(version_map);
    };
    bucket_entries
        .par_bridge()
        .filter_map(|bucket| {
            let p = bucket.ok()?.path();
            let child = p.join("bucket");
            let target = if child.is_dir() { child } else { p };
            let entries: Vec<_> = std::fs::read_dir(target).ok()?.collect();
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
