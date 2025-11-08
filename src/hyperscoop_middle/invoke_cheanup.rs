use crate::command_args::cleanup::CleanupArgs;
use crate::i18n::tr;
use anyhow::bail;
use anyhow::{anyhow, Context};
use command_util_lib::init_env::{
    get_app_dir, get_app_dir_global, get_app_version_dir, get_app_version_dir_global,
    get_apps_path, get_apps_path_global,
};
use command_util_lib::utils::system::kill_processes_using_app;
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
    } else {
        if args.all {
            clean_all_old_versions(args.global)?
        }
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
        let app_name = dir.file_stem().unwrap().to_str().unwrap();
        let child_dirs = dir
            .read_dir()
            .context(format!("Failed to read directory: {}", dir.display()))?
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
            .ok_or(anyhow!("No version directory found at line 64"))?;

        let retain_dir = highest_version_dir;
        let flag = Arc::new(Mutex::new(false));
        let result = child_dirs.par_iter().try_for_each(|dir| {
            if dir != retain_dir {
                log::info!("Removing old version: {}", dir.display());
                let result = std::fs::remove_dir_all(dir).context("Failed to remove old version");
                *flag.lock().unwrap() = true;
                if result.is_err() {
                    kill_processes_using_app(app_name);
                    std::fs::remove_dir_all(dir)
                        .context("Failed to remove old version at line 80")?;
                }
            }
            Ok(())
        });
        if !*flag.lock().unwrap() {
            println!(
                "{}",
                format!(
                    tr("No old versions for '{}'.", "未找到 '{}' 的旧版本。"),
                    dir.display()
                )
            );
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
    let apps_dir = if is_global {
        get_apps_path_global()
    } else {
        get_apps_path()
    };

    let apps_dir =
        std::fs::read_dir(apps_dir).context("Failed to read apps  root directory at line 96")?;
    let mut versions_with_name = HashMap::new();

    for app_dir in apps_dir {
        let mut dir_count = 0;
        let mut versions_max = String::new();

        let path = app_dir
            .context("Failed to read app directory at line 105")?
            .path();
        let app_name = path
            .clone()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        if path.is_dir() {
            for entry in path
                .read_dir()
                .context("Failed to read app directory at line 118")?
            {
                let entry = entry?;
                if entry.path().is_dir() {
                    dir_count += 1;
                }
            }
            if dir_count <= 2 {
                continue;
            }
            for version_dir in path
                .read_dir()
                .context("Failed to read version directory at line 130")?
            {
                let version_path = version_dir
                    .context("Failed to read version directory at line 133")?
                    .path();
                if version_path.is_dir() {
                    let version_name = version_path.file_name().unwrap().to_str().unwrap();
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
        println!(
            "{}",
            tr("No old versions found", "没有发现旧版本").green().bold()
        );
    }
    log::info!("{:?}", versions_with_name);
    for (app_name, version) in versions_with_name {
        let exclude_path = if is_global {
            get_app_version_dir_global(&app_name, &version)
        } else {
            get_app_version_dir(&app_name, &version)
        };
        let exclude_path = Path::new(&exclude_path);
        let dir = if is_global {
            get_app_dir_global(&app_name)
        } else {
            get_app_dir(&app_name)
        };
        log::info!("{:?}", dir);
        if exclude_path.exists() {
            for entry in
                std::fs::read_dir(dir).context("Failed to read app directory at line 169")?
            {
                let entry = entry?;
                if entry.path().is_dir() {
                    if entry.path() == exclude_path {
                        continue;
                    }
                    let path = entry.path();
                    let version = path.file_name().unwrap().to_str().unwrap();
                    if version == "current" {
                        continue;
                    }
                    log::info!("Removing old version: {}", path.display());
                    if std::fs::remove_dir_all(path.as_path())
                        .context("Failed to remove old version at line 188")
                        .is_err()
                    {
                        kill_processes_using_app(&app_name);
                        std::fs::remove_dir_all(path.as_path())
                            .context("Failed to remove old version at line 195")?;
                    }
                }
            }
        }
    }

    Ok(())
}

mod tests {

    #[test]
    fn test_file_stem() {
        use std::path::Path;
        let path =
            Path::new("C:\\Users\\Administrator\\scoop\\apps\\rustup\\current\\bin\\rustup.exe");
        let app_name = path.file_stem().unwrap().to_str().unwrap();
        let bin_name = path.file_name().unwrap().to_str().unwrap();
        assert_eq!(bin_name, "rustup.exe");
        assert_eq!(app_name, "rustup");
        let dir = Path::new("C:\\Users\\Administrator\\scoop\\apps\\rustup\\current");
        let app_name = dir.file_stem().unwrap().to_str().unwrap();
        let file_name = dir.file_name().unwrap().to_str().unwrap();
        assert_eq!(file_name, "current");
        assert_eq!(app_name, "current");
    }
}
