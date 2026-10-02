use crate::i18n::tr;
use crate::init_env::*;
use crate::install::{InstallOptions, create_shim_or_shortcuts};
use crate::list::VersionJSON;
use crate::tr_fmt;
use crate::update::{check_bucket_update_status, update_all_buckets_bar_parallel};
use crate::utils::utility::update_scoop_config_last_update_time;
use anyhow::{Context, bail};
use crossterm::style::Stylize;
use std::path::Path;

pub fn get_app_old_version(app_name: &str, options: &[InstallOptions]) -> anyhow::Result<String> {
    let app_install_manifest = if options.contains(&InstallOptions::Global) {
        get_app_dir_manifest_json_global(app_name)
    } else {
        get_app_dir_manifest_json(app_name)
    };
    if !Path::new(&app_install_manifest).exists() {
        bail!(
            "{}",
            tr_fmt!(
                "Not found {app_name} install manifest file",
                "未找到 {app_name} 的安装 manifest 文件",
                app_name = app_name
            )
        )
    }
    let content = std::fs::read_to_string(&app_install_manifest)
        .context("Failed to read the app install manifest file")?;
    let version: VersionJSON = serde_json::from_str(content.as_str())
        .context("Failed to parse the app install manifest file")?;
    let version = version.version;
    if version.is_none() {
        bail!(
            "{}",
            tr_fmt!(
                "Not found version in install manifest file for app: {app_name}",
                "在 {app_name} 的 manifest 文件中未找到版本信息",
                app_name = app_name
            )
        )
    }
    Ok(version.unwrap())
}

pub fn check_before_install(
    name: &str,
    version: &str,
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
        std::fs::create_dir_all(app_dir_path).context("Failed to create app directory")?;
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

        if Path::new(&install_json).exists() && Path::new(&manifest_json).exists() {
            let old_version =
                get_app_old_version(name, options).unwrap_or_else(|_| "unknown".to_string());
            println!(
                "{}",
                tr_fmt!(
                    "WARN  '{name}' ({old_version}) is already installed",
                    "WARN  '{name}' ({old_version}) 已经安装",
                    name = name,
                    old_version = old_version
                )
                .dark_yellow()
                .bold(),
            );
            println!(
                "{}",
                tr_fmt!(
                    "You can use 'hp update {name}' to install another version",
                    "您可以使用 'hp update {name}' 安装其他版本",
                    name = name
                )
                .dark_cyan()
                .bold()
            );
            Ok(1)
        } else {
            if !Path::new(&install_json).exists() {
                eprintln!(
                    "{}",
                    tr_fmt!(
                        "WARN  '{name}' install.json is missing, reinstall is recommended",
                        "WARN  '{name}' install.json 文件丢失，建议覆盖安装",
                        name = name
                    )
                    .dark_yellow()
                    .bold()
                );
            }

            if !Path::new(&manifest_json).exists() {
                eprintln!(
                    "{}",
                    tr_fmt!(
                        "WARN  '{name}' manifest.json is missing, reinstall is recommended",
                        "WARN  '{name}' manifest.json 文件丢失，建议覆盖安装",
                        name = name
                    )
                    .dark_yellow()
                    .bold()
                );
            }
            println!(
                "{}",
                tr_fmt!(
                    "ERROR '{name}' is not installed correctly",
                    "ERROR '{name}' 未正确安装",
                    name = name
                )
                .dark_red()
                .bold(),
            );
            println!(
                "{}",
                tr_fmt!(
                    "WARN  '{name}' Cleaning up previously failed installation files first",
                    "WARN  '{name}' 先清除之前安装失败的文件",
                    name = name
                )
                .dark_yellow()
                .bold(),
            );
            let target =
                std::fs::read_link(&app_current_dir).context("Failed to read link target")?;

            std::fs::remove_dir_all(target).context("Failed to remove target directory")?;

            std::fs::remove_dir_all(app_current_dir)
                .context("Failed to remove app current directory")?;

            println!(
                "{}",
                tr_fmt!(
                    "'{name}' was already uninstalled successfully!",
                    "'{name}' 已成功卸载！",
                    name = name
                )
                .dark_green()
                .bold(),
            );
            Ok(0)
        }
    } else if app_version_path.exists() && std::fs::symlink_metadata(&app_current_dir).is_err() {
        let manifest_json = if options.contains(&InstallOptions::Global) {
            get_app_dir_version_dir_manifest_global(name, version)
        } else {
            get_app_dir_version_dir_manifest(name, version)
        };
        if !Path::new(&manifest_json).exists() {
            eprintln!(
                "{}",
                tr_fmt!(
                    "'{name}' manifest.json is missing, reinstall is recommended",
                    "'{name}' manifest.json 文件丢失，建议覆盖安装",
                    name = name
                )
                .dark_yellow()
                .bold()
            );
            println!(
                "{}",
                tr_fmt!(
                    "ERROR '{name}' is not installed correctly",
                    "ERROR '{name}' 未正确安装",
                    name = name
                )
                .dark_red()
                .bold(),
            );
            println!(
                "{}",
                tr_fmt!(
                    "WARN  '{name}' Cleaning up previously failed installation files first",
                    "WARN  '{name}' 先清除之前安装失败的文件",
                    name = name
                )
                .dark_yellow()
                .bold(),
            );
            check_child_directory(&app_dir)?;
            println!(
                "{}",
                tr_fmt!(
                    "'{name}' was already uninstalled successfully!",
                    "'{name}' 已成功卸载！",
                    name = name
                )
                .dark_green()
                .bold(),
            );
        }
        println!(
            "{}",
            tr(
                "WARN  Fixing missing links and shortcuts",
                "WARN  修复缺失的链接和快捷方式"
            )
            .dark_yellow()
            .bold(),
        );
        let old_version = version;
        println!(
            "{}",
            tr_fmt!(
                "Resetting '{name}' ({old_version})",
                "正在重置 '{name}' ({old_version})",
                name = name,
                old_version = old_version
            )
            .dark_cyan()
            .bold()
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
                tr_fmt!(
                    "WARN  '{name}' ({old_version}) is already installed",
                    "WARN  '{name}' ({old_version}) 已经安装",
                    name = name,
                    old_version = old_version
                )
                .dark_yellow()
                .bold(),
            );
            println!(
                "{}",
                tr_fmt!(
                    "You can use 'hp update {name}' to install another version",
                    "您可以使用 'hp update {name}' 安装其他版本",
                    name = name
                )
                .dark_cyan()
                .bold()
            );
            return Ok(1);
        } else {
            eprintln!(
                "{}",
                tr_fmt!(
                    "'{name}' install.json is missing, reinstall is recommended",
                    "'{name}' install.json 文件丢失，建议覆盖安装",
                    name = name
                )
                .dark_yellow()
                .bold()
            );
            println!(
                "{}",
                tr_fmt!(
                    "ERROR '{name}' is not installed correctly",
                    "ERROR '{name}' 未正确安装",
                    name = name
                )
                .dark_red()
                .bold(),
            );
            println!(
                "{}",
                tr_fmt!(
                    "WARN  '{name}' Cleaning up previously failed installation files first",
                    "WARN  '{name}' 先清除之前安装失败的文件",
                    name = name
                )
                .dark_yellow()
                .bold(),
            );
            check_child_directory(&app_dir)?;
            println!(
                "{}",
                tr_fmt!(
                    "'{name}' was already uninstalled successfully!",
                    "'{name}' 已成功卸载！",
                    name = name
                )
                .dark_green()
                .bold(),
            );
            Ok(0)
        }
    } else if std::fs::symlink_metadata(&app_current_dir).is_ok() && !app_current_path.exists()
    //exists默认会解析符号链接
    {
        println!(
            "{}",
            tr_fmt!(
                "ERROR  '{name}' is not installed correctly",
                "ERROR  '{name}' 未正确安装",
                name = name
            )
            .dark_red()
            .bold(),
        );

        println!(
            "{}",
            tr_fmt!(
                "WARN '{name}' Cleaning up previously failed installation files first",
                "WARN '{name}' 先清除之前安装失败的文件",
                name = name
            )
            .dark_yellow()
            .bold(),
        );
        check_child_directory(&app_dir)?;

        println!(
            "{}",
            tr_fmt!("'{name}' was uninstalled", "'{name}' 已卸载", name = name)
                .dark_green()
                .bold(),
        );
        std::fs::remove_dir_all(app_dir_path).context("Failed to remove app directory")?;
        Ok(0)
    } else if !app_version_path.exists() && std::fs::symlink_metadata(app_current_dir).is_err() {
        println!(
            "{}",
            tr_fmt!(
                "ERROR  '{name}' is not installed correctly",
                "ERROR  '{name}' 未正确安装",
                name = name
            )
            .dark_red()
            .bold(),
        );
        println!(
            "{}",
            tr_fmt!(
                "WARN  '{name}' Cleaning up previously failed installation files first",
                "WARN  '{name}' 先清除之前安装失败的文件",
                name = name
            )
            .dark_yellow()
            .bold(),
        );
        check_child_directory(&app_dir)?;

        println!(
            "{}",
            tr_fmt!("'{name}' was uninstalled", "'{name}' 已卸载", name = name)
                .dark_green()
                .bold(),
        );
        std::fs::remove_dir_all(app_dir_path).context("Failed to remove app directory")?;
        Ok(0)
    } else {
        println!(
            "{}",
            tr_fmt!(
                "ERROR  '{name}' is not installed correctly",
                "ERROR  '{name}' 未正确安装",
                name = name
            )
            .dark_red()
            .bold(),
        );
        Ok(0)
    }
}

fn check_child_directory(app_dir: &String) -> anyhow::Result<()> {
    let dirs = std::fs::read_dir(app_dir).context("Failed to read app directory")?;
    for dir in dirs {
        let dir = dir?;
        let path = dir.path();
        if Path::new(&path).exists() {
            println!(
                "{} {}",
                tr("Removing", "正在删除").dark_cyan().bold(),
                path.to_string_lossy().dark_cyan().bold()
            );
        }
    }
    Ok(())
}

pub fn create_dir_symbolic_link(version_dir: &String, current_dir: &String) -> anyhow::Result<()> {
    crate::utils::system::create_dir_link(version_dir, current_dir)
        .context("Failed to create link directory")?;
    println!(
        "{} {}",
        tr("Creating Link", "正在创建链接").dark_blue().bold(),
        format!("{current_dir}  => {version_dir}")
            .dark_green()
            .bold()
    );
    Ok(())
}
