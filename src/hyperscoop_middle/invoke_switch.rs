use crate::command_args::switch::SwitchArgs;
use crate::i18n::{tr, tr_fmt};
use anyhow::{Context, bail};
use command_util_lib::init_env::{get_app_dir, get_app_dir_global};
use command_util_lib::reset::reset_specific_version;
use crossterm::style::Stylize;
use std::path::Path;

pub fn execute_switch_command(args: SwitchArgs) -> anyhow::Result<()> {
    let app_dir = if args.global {
        get_app_dir_global(&args.app_name)
    } else {
        get_app_dir(&args.app_name)
    };
    let app_path = Path::new(&app_dir);

    if !app_path.exists() {
        bail!(
            "{}",
            tr_fmt!(
                "App '{name}' is not installed.",
                "应用 '{name}' 尚未安装。",
                name = args.app_name
            )
        );
    }

    let mut versions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(app_path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name != "current" {
                        versions.push(file_name);
                    }
                }
            }
        }
    }

    if versions.is_empty() {
        bail!(
            "{}",
            tr_fmt!(
                "No version directories found for '{name}'.",
                "未在 '{name}' 目录下找到任何已安装版本。",
                name = args.app_name
            )
        );
    }

    versions
        .sort_by(|a, b| command_util_lib::utils::utility::compare_versions(a.clone(), b.clone()));

    // Detect current active version
    let current_path = app_path.join("current");
    let current_version = if current_path.exists() {
        if let Ok(target) = std::fs::read_link(&current_path) {
            target.file_name().map(|n| n.to_string_lossy().to_string())
        } else {
            // Read manifest inside current
            let manifest_path = current_path.join("manifest.json");
            if let Ok(content) = std::fs::read_to_string(manifest_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    val.get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }
    } else {
        None
    };

    match args.version {
        None => {
            println!(
                "{}",
                tr_fmt!(
                    "Installed versions for '{name}':",
                    "应用 '{name}' 已安装的可用版本：",
                    name = args.app_name
                )
                .dark_cyan()
                .bold()
            );

            for ver in &versions {
                let is_active = current_version.as_deref() == Some(ver.as_str());
                if is_active {
                    println!(
                        "  * {} {}",
                        ver.as_str().green().bold(),
                        tr("(active)", "(当前使用中)").green()
                    );
                } else {
                    println!("    {}", ver);
                }
            }

            println!(
                "\n{}",
                tr_fmt!(
                    "Run 'hp switch {name} <version>' to switch.",
                    "运行 'hp switch {name} <版本号>' 即可切换版本。",
                    name = args.app_name
                )
                .dark_grey()
            );
        }
        Some(target_ver) => {
            let matched = versions
                .iter()
                .find(|v| v.eq_ignore_ascii_case(&target_ver));

            match matched {
                Some(ver) => {
                    reset_specific_version(&args.app_name, ver, args.global, true).context(
                        format!("Failed to switch '{}' to version {}", args.app_name, ver),
                    )?;

                    println!(
                        "{}",
                        tr_fmt!(
                            "Successfully switched '{name}' to version {version}!",
                            "成功将 '{name}' 切换至版本 {version}！",
                            name = args.app_name,
                            version = ver
                        )
                        .green()
                        .bold()
                    );
                }
                None => {
                    bail!(
                        "{}",
                        tr_fmt!(
                            "Version '{version}' is not installed for '{name}'. Available: {available}",
                            "版本 '{version}' 未在 '{name}' 中安装。可用版本: {available}",
                            version = target_ver,
                            name = args.app_name,
                            available = versions.join(", ")
                        )
                    );
                }
            }
        }
    }

    Ok(())
}
