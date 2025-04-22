use crate::init_env::{get_apps_path, get_apps_path_global};
use crate::utils::utility::compare_versions;
use anyhow::bail;
use crossterm::style::Stylize;
use std::cmp::Ordering;
use std::os::windows::fs::symlink_dir;

pub fn reset_latest_version(name: String, global: bool) -> Result<(), anyhow::Error> {
    let app_root_dir = if global {
        get_apps_path_global()
    } else {
        get_apps_path()
    };
    for entry in std::fs::read_dir(app_root_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            if dir_name != name {
                continue;
            }
            log::info!("Resetting app: {}", dir_name);
            let mut count = 0;
            for _ in std::fs::read_dir(&path)? {
                count += 1;
            }
            if count <= 1 {
                bail!("app 文件目录格式不正确,不存在Current目录或者至少一个版本目录")
            } else if count == 2 {
                let current_path = path.join("current");
                log::info!("Resetting app: {}", current_path.display());
                if !current_path.exists() {
                    bail!("Current directory is not exist");
                }
                std::fs::remove_dir_all(&current_path)?;
                for entry in std::fs::read_dir(&path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    symlink_dir(&path, current_path.as_path())
                        .expect("Failed to create app symlink");
                    println!(
                        "Resetting app  successfully {}=>{}",
                        current_path.display().to_string().dark_green().bold(),
                        path.display().to_string().dark_green().bold()
                    );
                }
                return Ok(());
            } else {
                let current_path = path.join("current");
                if !current_path.exists() {
                    bail!("Current directory is not exist");
                };
                std::fs::remove_dir_all(&current_path).unwrap_or_else(|e| {
                    eprintln!("Failed to remove current directory, error: {}", e);
                });
                let mut max_version = String::new();
                for entry in std::fs::read_dir(&path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let version_name = path.file_name().unwrap().to_str().unwrap();
                    log::info!("Resetting app: {}@{}", name, version_name);
                    match compare_versions(version_name.into(), max_version.clone()) {
                        Ordering::Less => {}
                        Ordering::Equal => {}
                        Ordering::Greater => max_version = version_name.to_string(),
                    }
                }

                let max_version_path = path.join(&max_version);
                log::info!("Resetting app: {}", max_version_path.display());
                symlink_dir(max_version_path, current_path.as_path())
                    .expect("Failed to create app symlink");
                println!("Reset {}@{} success", name, &max_version);
                return Ok(());
            }
        }
    }
    bail!("App not found: {}", name)
}

pub fn reset_specific_version(
    name: &str,
    version: &str,
    global: bool,
) -> Result<(), anyhow::Error> {
    log::info!("Resetting app: {}@{}", name, version);
    let app_root_dir = if global {
        get_apps_path_global()
    } else {
        get_apps_path()
    };

    for entry in std::fs::read_dir(app_root_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            if dir_name != name {
                continue;
            }
            log::info!("Resetting app: {}", dir_name);
            let mut count = 0;
            for _ in std::fs::read_dir(&path)? {
                count += 1;
            }
            if count <= 1 {
                bail!("app 文件目录格式不正确,不存在Current目录或者至少一个版本目录")
            } else {
                let current_path = path.join("current");

                let version_path = path.join(&version);
                if !version_path.exists() {
                    bail!("Version dir {} is not exist ", version);
                }
                std::fs::remove_dir_all(&current_path).unwrap_or_else(|e| {
                    eprintln!("Failed to remove current directory, error: {}", e);
                });
                symlink_dir(version_path, current_path.as_path()).expect("Create dir symlink failed");;
                println!("Reset {}@{} success", name, &version);
                return Ok(());
            }
        }
    }
    bail!("App not found: {}", name);
}
