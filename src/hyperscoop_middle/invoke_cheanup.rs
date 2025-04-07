use std::cmp::Ordering;
use crate::command_args::cleanup::CleanupArgs;
use command_util_lib::init_hyperscoop;
use crossterm::style::Stylize;
use std::collections::HashMap;
use command_util_lib::utils::utility::compare_versions;

pub fn execute_cleanup_command(args: CleanupArgs) -> Result<(), anyhow::Error> {
    if let Some(name) = args.app_names {
        if args.all {
            clean_all_old_versions()
        } else {
            clean_specific_old_version(name)
        }
    }
    if args.all {
        log::info!("Run cleanup all command");
        clean_all_old_versions()
    }
    Ok(())
}

fn clean_specific_old_version(app_name: Vec<String>) {
    log::info!("Run cleanup command '{:?}'", app_name);
}

fn clean_all_old_versions() {
    let hyperscoop = init_hyperscoop().expect("Failed to initialize hyperscoop");
    let apps_path = hyperscoop.apps_path;
    let apps_dir = std::fs::read_dir(&apps_path).expect("Failed to read apps directory");
    let mut versions_with_name = HashMap::new();

    for app_dir in apps_dir {
        let mut dir_count = 0;
        let mut versions_max = String::new();

        let path = app_dir.expect("Failed to read app directory").path();
        let app_name = path
            .clone()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        if path.is_dir() {
            for entry in path.read_dir().expect("Failed to read app directory") {
                let entry = entry.expect("Failed to read app directory entry");
                if entry.path().is_dir() {
                    dir_count += 1;
                }
            }
            if dir_count <= 2 {
                continue;
            }
            for version_dir in path.read_dir().expect("Failed to read version directory") {
                let version_path = version_dir
                    .expect("Failed to read version directory")
                    .path();
                if version_path.is_dir() {
                    let version_name = version_path
                        .file_name()
                        .expect("Failed to get version name")
                        .to_str()
                        .expect("Failed to convert version name to string");
                    if version_name == "current" {
                        continue;
                    }
                    match compare_versions(version_name.to_string(), versions_max.clone()) {
                        Ordering::Less => {}
                        Ordering::Equal => {}
                        Ordering::Greater => versions_max = version_name.to_string(),
                    }
                }
            }
        }
        versions_with_name.insert(app_name, versions_max.clone());
    }
    if versions_with_name.len() == 0 || versions_with_name.is_empty() {
        println!("{}", "No old versions found".green().bold());
    }
    log::info!("{:?}", versions_with_name);
    for (app_name, version) in versions_with_name {
        let exclude_path = format!("{}\\{}\\{}", apps_path.clone(), app_name, version);
        let exclude_path = std::path::Path::new(&exclude_path);
        let dir = format!("{}\\{}", apps_path.clone(), app_name);
        log::info!("{:?}", dir);
        if exclude_path.exists() {
            for entry in std::fs::read_dir(dir).expect("Failed to read app directory") {
                let entry = entry.expect("Failed to read app directory entry");
                if entry.path().is_dir() {
                    if entry.path() == exclude_path {
                        continue;
                    }
                    let name = entry
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    if name == "current" {
                        continue;
                    }
                    log::info!("Removing old version: {}", entry.path().display());
                    // std::fs::remove_dir_all(entry.path()).expect("Failed to remove old version");
                }
            }
        }
    }
}

