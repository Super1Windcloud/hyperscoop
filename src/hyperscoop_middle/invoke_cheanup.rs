use crate::command_args::cleanup::CleanupArgs;
use anyhow::anyhow;
use anyhow::bail;
use command_util_lib::init_env::{get_app_dir, get_app_dir_global};
use command_util_lib::init_hyperscoop;
use command_util_lib::utils::utility::compare_versions;
use crossterm::style::Stylize;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub fn execute_cleanup_command(args: CleanupArgs) -> Result<(), anyhow::Error> {
    if let Some(name) = args.app_names {
        if args.all {
            clean_all_old_versions(args.global)?
        } else {
            clean_specific_old_version(name, args.global)?
        }
    }
    if args.all {
        clean_all_old_versions(args.global)?
    }
    Ok(())
}

fn clean_specific_old_version(app_name: Vec<String>, is_global: bool) -> anyhow::Result<()> {
    log::info!("Run cleanup command '{:?}'", app_name);
    let app_dirs = app_name
        .iter()
        .map(|name| {
            if is_global {
                get_app_dir_global(name)
            } else {
                get_app_dir(name)
            }
        })
        .collect::<Vec<_>>();
    let result = app_dirs.iter().try_for_each(|dir| {
        let dir = Path::new(dir);
        let child_dirs = dir
            .read_dir()?
            .filter_map(|dir| {
                let dir = dir.unwrap();
                let binding = dir.file_name();
                let name = binding.to_str().unwrap();
                if name == "current" {
                    return None;
                } else {
                    Some(dir.path())
                }
            })
            .collect::<Vec<_>>();
        let highest_version_dir = child_dirs
            .par_iter()
            .max_by(|dir1, dir2| {
                let file_name = dir1.file_name().unwrap().to_str().unwrap();
                let file_name2 = dir2.file_name().unwrap().to_str().unwrap();
                compare_versions(file_name.to_string(), file_name2.to_string())
            })
            .ok_or(anyhow!("No version directory found"))?;

        let retain_dir = highest_version_dir; 
        let      flag = Arc::new(Mutex::new(false )) ; 
        let result = child_dirs.par_iter().try_for_each(|dir| {
            if dir != retain_dir {
                log::info!("Removing old version: {}", dir.display());
                std::fs::remove_dir_all(dir)?;
                *flag.lock().unwrap() = true; 
            }
            Ok(())
        });
       if  !* flag.lock().unwrap()  { 
          println!("No old version for '{}'", dir.display());
       }
        result
    });
    if result.is_err() {
        let err: anyhow::Error = result.unwrap_err();
        bail!("Run cleanup Err '{:?}'", err);
    }
    Ok(())
}

fn clean_all_old_versions(is_global: bool) -> anyhow::Result<()> {
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
        let exclude_path = Path::new(&exclude_path);
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
                    std::fs::remove_dir_all(entry.path()).expect("Failed to remove old version");
                }
            }
        }
    }

    Ok(())
}
