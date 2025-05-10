use crate::init_env::*;
use crate::install::{create_shim_or_shortcuts, InstallOptions};
use crate::list::VersionJSON;
use crate::update::{check_bucket_update_status, update_all_buckets_bar_parallel};
use crate::utils::utility::update_scoop_config_last_update_time;
use anyhow::bail;
use crossterm::style::Stylize;
use std::os::windows::fs::symlink_dir;
use std::path::Path;

pub fn get_app_old_version(app_name: &str, options: &[InstallOptions]) -> anyhow::Result<String> {
    let  app_install_manifest = if options.contains(&InstallOptions::Global) {
        get_app_dir_manifest_json_global(app_name)
    } else {
        get_app_dir_manifest_json(app_name)
    };
    if !Path::new(&app_install_manifest).exists() {
        bail!("Not found {app_name} install manifest file")
    }
    let content = std::fs::read_to_string(&app_install_manifest)?;
    let version: VersionJSON = serde_json::from_str(content.as_str())?;
    let version = version.version;
    if version.is_none() {
        bail!("Not found version in  install manifest file for app: {app_name}")
    }
    Ok(version.unwrap())
}

pub fn check_before_install(
    name: &str,
    version: &str ,
    options: &Box<[InstallOptions<'_>]>,
) -> anyhow::Result<u8> {
    if options.contains(&InstallOptions::UpdateHpAndBuckets) {
        let status = check_bucket_update_status()?;
        if status {
            update_all_buckets_bar_parallel()?;
            update_scoop_config_last_update_time();
        }
    }
    let app_dir = if options.contains(&InstallOptions::Global) {
        get_app_dir_global(name)
    } else {
        get_app_dir(name)
    };
    let app_dir_path = Path::new(&app_dir);
    if !app_dir_path.exists() {
        std::fs::create_dir_all(app_dir_path)?;
        return Ok(0);
    }
    let app_version_dir = if options.contains(&InstallOptions::Global) {
        get_app_version_dir_global(name, &version)
    } else {
        get_app_version_dir(name, &version)
    };
    let app_current_dir = if options.contains(&InstallOptions::Global) {
        get_app_current_dir_global(name)
    } else {
        get_app_current_dir(name)
    };
    let app_version_path = Path::new(&app_version_dir);
    let app_current_path = Path::new(&app_current_dir);
    let old_version = get_app_old_version(name, options)?;
    if app_current_path.exists() {
        let install_json = if options.contains(&InstallOptions::Global) {
            get_app_dir_install_json_global(name)
        } else {
            get_app_dir_install_json(name)
        };
        let manifest_json = if options.contains(&InstallOptions::Global) {
            get_app_dir_manifest_json_global(name)
        } else {
            get_app_dir_manifest_json(name)
        };

        if Path::new(&install_json).exists()
            && Path::new(&manifest_json).exists()
        {
            println!(
                "{}",
                format!("WARN  '{name }' ({old_version}) is already installed")
                    .to_string()
                    .dark_yellow()
                    .bold(),
            );
            println!(
                "{}",
                format!("You can use 'hp update {name}' to  install another version")
                    .to_string()
                    .dark_cyan()
                    .bold()
            );
            Ok(1)
        } else {
            if !Path::new(&install_json).exists() {
                eprintln!(
                    "{}",
                    format!("WARN  '{name}'  install.json文件丢失, 建议覆盖安装")
                        .dark_yellow()
                        .bold()
                );
            }
          
            if !Path::new(&manifest_json).exists() {
                eprintln!(
                    "{}",
                    format!("WARN  '{name}'  manifest.json文件丢失, 建议覆盖安装")
                        .dark_yellow()
                        .bold()
                );
            }
            println!(
                "{}",
                format!("ERROR '{name}'  isn't installed correctly")
                    .dark_red()
                    .bold(),
            );
            println!(
                "{}",
                format!("WARN  '{name}'  先清除之前安装失败的文件")
                    .dark_yellow()
                    .bold(),
            );
            check_child_directory(&app_dir)?;
            println!(
                "{}",
                format!("'{name}' was already uninstalled successfully")
                    .dark_green()
                    .bold(),
            );
            std::fs::remove_dir_all(app_dir_path)?;
            Ok(0)
        }
    }
    else if app_version_path.exists() && std::fs::symlink_metadata(&app_current_dir).is_err() {
        let manifest_json = if options.contains(&InstallOptions::Global) {
            get_app_dir_version_dir_manifest_global(name, version)
        } else {
            get_app_dir_version_dir_manifest(name, version)
        };
        if !Path::new(&manifest_json).exists() {
            eprintln!(
                "{}",
                format!("'{name}'  manifest.json文件丢失, 建议覆盖安装")
                    .dark_yellow()
                    .bold()
            );
            println!(
                "{}",
                format!("ERROR '{name}'  isn't installed correctly")
                    .dark_red()
                    .bold(),
            );
            println!(
                "{}",
                format!("WARN  '{name}'  先清除之前安装失败的文件")
                    .dark_yellow()
                    .bold(),
            );
            check_child_directory(&app_dir)?;
            println!(
                "{}",
                format!("'{name}' was already uninstalled successfully")
                    .dark_green()
                    .bold(),
            );
        }
        println!(
            "{}",
            "WARN  修复缺失的链接和快捷方式"
                .to_string()
                .dark_yellow()
                .bold()
        );
        println!(
            "{}",
            format!("Resetting '{name}' ({version})").dark_cyan().bold()
        );
        create_dir_symbolic_link(&app_version_dir, &app_current_dir)?;
        create_shim_or_shortcuts(&manifest_json, name, options)
            .expect("Could not create shim or shortcuts");
        let install_json = if options.contains(&InstallOptions::Global) {
            get_app_dir_install_json_global(name)
        } else {
            get_app_dir_install_json(name)
        };
        if Path::new(&install_json).exists() {
            println!(
                "{}",
                format!("WARN  '{name}' ({version}) is already installed")
                    .to_string()
                    .dark_yellow()
                    .bold(),
            );
            println!(
                "{}",
                format!("You can use 'hp update {name}' to  install another version")
                    .to_string()
                    .dark_cyan()
                    .bold()
            );
            return Ok(1);
        } else {
            eprintln!(
                "{}",
                format!("'{name}' install.json文件丢失, 建议覆盖安装")
                    .dark_yellow()
                    .bold()
            );
            println!(
                "{}",
                format!("ERROR '{name}'  isn't installed correctly")
                    .dark_red()
                    .bold(),
            );
            println!(
                "{}",
                format!("WARN  '{name}'  先清除之前安装失败的文件")
                    .dark_yellow()
                    .bold(),
            );
            check_child_directory(&app_dir)?;
            println!(
                "{}",
                format!("'{name}' was already uninstalled successfully")
                    .dark_green()
                    .bold(),
            );
            Ok(0)
        }
    }
    else if std::fs::symlink_metadata(&app_current_dir).is_ok() && !app_current_path.exists()
    //exists默认会解析符号链接
    {
        println!(
            "{}",
            format!("ERROR  '{name}' isn't installed correctly")
                .dark_red()
                .bold(),
        );

        println!(
            "{}",
            format!("WARN '{name}' 先清除之前安装失败的文件")
                .dark_yellow()
                .bold(),
        );
        check_child_directory(&app_dir)?;

        println!(
            "{}",
            format!("'{name}' was uninstalled ").dark_green().bold(),
        );
        std::fs::remove_dir_all(app_dir_path)?;
        Ok(0)
    } else if !app_version_path.exists() && std::fs::symlink_metadata(app_current_dir).is_err() {
        println!(
            "{}",
            format!("ERROR  '{name}' isn't installed correctly")
                .dark_red()
                .bold(),
        );
        println!(
            "{}",
            format!("WARN  '{name}' 先清除之前安装失败的文件")
                .dark_yellow()
                .bold(),
        );
        check_child_directory(&app_dir)?;

        println!(
            "{}",
            format!("'{name}' was uninstalled ").dark_green().bold(),
        );
        std::fs::remove_dir_all(app_dir_path)?;
        Ok(0)
    } else {
        println!(
            "{}",
            format!("ERROR  '{name}' isn't installed correctly, WTF?")
                .dark_red()
                .bold(),
        );
        return Ok(0);
    }
}

fn check_child_directory(app_dir: &String) -> anyhow::Result<()> {
    let dirs = std::fs::read_dir(app_dir)?;
    for dir in dirs {
        let dir = dir?;
        let path = dir.path();
        if Path::new(&path).exists() {
            println!("Removing {}", path.to_string_lossy().dark_cyan().bold());
        }
    }
    Ok(())
}

pub fn create_dir_symbolic_link(version_dir: &String, current_dir: &String) -> anyhow::Result<()> {
    symlink_dir(version_dir, current_dir).expect("Create dir symlink failed");
    println!(
        "Creating  Link  {}",
        format!("{current_dir}  => {version_dir}")
            .dark_green()
            .bold()
    );
    Ok(())
}
