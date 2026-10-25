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
    check_installation(name, version, options, app_dir)
}

fn check_installation(
    name: &str,
    version: &str,
    options: &Box<[InstallOptions<'_>]>,
    app_dir: String,
) -> anyhow::Result<u8> {
    let app_dir_path = Path::new(&app_dir);
    if !app_dir_path.exists() {
        std::fs::create_dir_all(app_dir_path).context("Failed to create app directory")?;
        return Ok(0);
    }
    let app_version_dir = app_dir_path.join(version).to_string_lossy().into_owned();
    let app_current_dir = app_dir_path.join("current").to_string_lossy().into_owned();
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
            if old_version == version {
                println!(
                    "{}",
                    tr_fmt!(
                        "Warning: '{name}' ({version}) is already installed and up-to-date.",
                        "警告: '{name}' ({version}) 已经安装且为最新版本。",
                        name = name,
                        version = version
                    )
                    .dark_yellow()
                    .bold(),
                );
                println!(
                    "{}",
                    tr_fmt!(
                        "To reinstall {version}, run:\n  hp reinstall {name}",
                        "如需重新安装 {version}，请运行:\n  hp reinstall {name}",
                        name = name,
                        version = version
                    )
                    .dark_cyan()
                    .bold(),
                );
                Ok(1)
            } else {
                let no_install_upgrade = std::env::var("HYPERSCOOP_NO_INSTALL_UPGRADE").is_ok()
                    || std::env::var("HOMEBREW_NO_INSTALL_UPGRADE").is_ok()
                    || options.contains(&InstallOptions::NoInstallUpgrade);

                if no_install_upgrade {
                    println!(
                        "{}",
                        tr_fmt!(
                            "Warning: '{name}' ({old_version}) is already installed, but {version} is available.",
                            "警告: '{name}' ({old_version}) 已安装，但有新版本 {version} 可用。",
                            name = name,
                            old_version = old_version,
                            version = version
                        )
                        .dark_yellow()
                        .bold(),
                    );
                    println!(
                        "{}",
                        tr_fmt!(
                            "To upgrade to {version}, run:\n  hp upgrade {name}",
                            "如需升级到 {version}，请运行:\n  hp upgrade {name}",
                            name = name,
                            version = version
                        )
                        .dark_cyan()
                        .bold()
                    );
                    Ok(1)
                } else {
                    println!(
                        "{} {}",
                        tr("==> Upgrading", "==> 正在自动升级").dark_cyan().bold(),
                        format!("'{name}' ({old_version} -> {version})").bold()
                    );
                    Ok(0)
                }
            }
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
        let manifest_json = app_version_path.join("manifest.json");
        let install_json = app_version_path.join("install.json");
        if !manifest_json.is_file() || !install_json.is_file() {
            let missing_file = if !manifest_json.is_file() {
                "manifest.json"
            } else {
                "install.json"
            };
            eprintln!(
                "{}",
                tr_fmt!(
                    "'{name}' {file} is missing, reinstall is recommended",
                    "'{name}' {file} 文件丢失，建议覆盖安装",
                    name = name,
                    file = missing_file
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
            // Only remove the incomplete version; other installed versions may be valid.
            println!(
                "{} {}",
                tr("Removing", "正在删除").dark_cyan().bold(),
                app_version_path.to_string_lossy().dark_cyan().bold()
            );
            std::fs::remove_dir_all(app_version_path)
                .context("Failed to remove incomplete app version directory")?;
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
            return Ok(0);
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
        create_shim_or_shortcuts(&manifest_json.to_string_lossy(), name, options)
            .context("Could not create shim or shortcuts")?;
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
        // Remove only the dangling link, then inspect the requested version again.
        std::fs::remove_dir_all(app_current_path)
            .context("Failed to remove broken app current link")?;
        check_installation(name, version, options, app_dir)
    } else if !app_version_path.exists() && std::fs::symlink_metadata(app_current_dir).is_err() {
        // A different version may still be installed without a current link.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_manifest_cleans_incomplete_version_and_continues_installation() -> anyhow::Result<()>
    {
        for options in [vec![], vec![InstallOptions::Global]] {
            let temp = tempfile::tempdir()?;
            let app_dir = temp.path().join("hiddify");
            let version_dir = app_dir.join("4.1.1");
            let previous_version_dir = app_dir.join("4.1.0");
            std::fs::create_dir_all(&version_dir)?;
            std::fs::write(version_dir.join("install.json"), "{}")?;
            std::fs::write(version_dir.join("partial-download"), "incomplete")?;
            std::fs::create_dir_all(&previous_version_dir)?;
            std::fs::write(previous_version_dir.join("manifest.json"), "{}")?;

            let result = check_installation(
                "hiddify",
                "4.1.1",
                &options.into_boxed_slice(),
                app_dir.to_string_lossy().into_owned(),
            )?;

            assert_eq!(result, 0, "the caller must resume installation");
            assert!(!version_dir.exists());
            assert!(std::fs::symlink_metadata(app_dir.join("current")).is_err());
            assert!(previous_version_dir.join("manifest.json").is_file());
        }
        Ok(())
    }

    #[test]
    fn invalid_manifest_returns_error_instead_of_panicking_during_reset() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let app_dir = temp.path().join("hiddify");
        let version_dir = app_dir.join("4.1.1");
        std::fs::create_dir_all(&version_dir)?;
        std::fs::write(version_dir.join("manifest.json"), "invalid json")?;
        std::fs::write(version_dir.join("install.json"), "{}")?;

        let error = check_installation(
            "hiddify",
            "4.1.1",
            &vec![].into_boxed_slice(),
            app_dir.to_string_lossy().into_owned(),
        )
        .expect_err("invalid manifest should return an error");

        let message = format!("{error:#}");
        assert!(message.contains("Could not create shim or shortcuts"));
        assert!(message.contains("Failed to parse manifest file"));
        Ok(())
    }

    #[test]
    fn missing_install_metadata_does_not_create_links_or_shims() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let app_dir = temp.path().join("hiddify");
        let version_dir = app_dir.join("4.1.1");
        std::fs::create_dir_all(&version_dir)?;
        std::fs::write(version_dir.join("manifest.json"), "{}")?;

        assert_eq!(
            check_installation(
                "hiddify",
                "4.1.1",
                &vec![].into_boxed_slice(),
                app_dir.to_string_lossy().into_owned(),
            )?,
            0
        );
        assert!(!version_dir.exists());
        assert!(std::fs::symlink_metadata(app_dir.join("current")).is_err());
        Ok(())
    }

    #[test]
    fn absent_requested_version_preserves_other_installed_versions() -> anyhow::Result<()> {
        for broken_link in [false, true] {
            let temp = tempfile::tempdir()?;
            let app_dir = temp.path().join("hiddify");
            let previous_version = app_dir.join("4.1.0");
            std::fs::create_dir_all(&previous_version)?;
            std::fs::write(previous_version.join("manifest.json"), "{}")?;
            if broken_link {
                let missing_version = app_dir.join("missing");
                std::fs::create_dir(&missing_version)?;
                crate::utils::system::create_dir_link(&missing_version, app_dir.join("current"))?;
                std::fs::remove_dir(&missing_version)?;
            }

            assert_eq!(
                check_installation(
                    "hiddify",
                    "4.1.1",
                    &vec![].into_boxed_slice(),
                    app_dir.to_string_lossy().into_owned(),
                )?,
                0
            );
            assert!(previous_version.join("manifest.json").is_file());
            assert!(std::fs::symlink_metadata(app_dir.join("current")).is_err());
        }
        Ok(())
    }

    #[test]
    fn broken_current_link_is_repaired_from_complete_requested_version() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let app_dir = temp.path();
        let version_dir = app_dir.join("4.1.1");
        std::fs::create_dir(&version_dir)?;
        std::fs::write(version_dir.join("manifest.json"), "{}")?;
        std::fs::write(version_dir.join("install.json"), "{}")?;
        let missing_version = app_dir.join("missing");
        std::fs::create_dir(&missing_version)?;
        crate::utils::system::create_dir_link(&missing_version, app_dir.join("current"))?;
        std::fs::remove_dir(&missing_version)?;

        assert_eq!(
            check_installation(
                "hiddify",
                "4.1.1",
                &vec![].into_boxed_slice(),
                app_dir.to_string_lossy().into_owned(),
            )?,
            1
        );
        assert!(app_dir.join("current").join("manifest.json").is_file());
        assert!(version_dir.join("install.json").is_file());
        Ok(())
    }
}
