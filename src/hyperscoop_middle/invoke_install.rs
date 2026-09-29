use crate::check_self_update::auto_check_hp_update;
use crate::command_args::install::InstallArgs;
use crate::hyperscoop_middle::invoke_update::update_buckets_parallel;
use crate::i18n::{tr, tr_fmt};
use anyhow::bail;
use command_util_lib::install::*;
use command_util_lib::utils::system::{get_system_default_arch, is_admin, request_admin};
use command_util_lib::utils::utility::is_valid_url;
use crossterm::style::Stylize;
use std::env;
use std::path::Path;

pub async fn execute_install_command(args: InstallArgs) -> Result<(), anyhow::Error> {
    if args.app_names.is_empty() {
        return Ok(());
    }

    if args.global && !is_admin()? {
        let args_env = env::args().skip(1).collect::<Vec<String>>();
        let args_str = args_env.join(" ");
        log::warn!(
            "Global command arguments: {}",
            args_str.clone().dark_yellow()
        );
        request_admin(args_str.as_str())?;
        return Ok(());
    }

    let options = inject_user_options(&args)?;
    if options.contains(&InstallOptions::CheckCurrentVersionIsLatest) {
        auto_check_hp_update(None).await?;
    }
    if options.contains(&InstallOptions::UpdateHpAndBuckets) {
        println!(
            "{}",
            tr("Starting bucket updates", "开始更新 buckets")
                .dark_cyan()
                .bold()
        );
        update_buckets_parallel()?;
    }

    let total = args.app_names.len();
    let mut errors = Vec::new();

    for (i, raw_name) in args.app_names.iter().enumerate() {
        if total > 1 {
            println!(
                "{}",
                tr_fmt!(
                    "\n[{i}/{total}] Installing '{name}'...",
                    "\n[{i}/{total}] 正在安装 '{name}'...",
                    i = i + 1,
                    total = total,
                    name = raw_name
                )
                .dark_cyan()
                .bold()
            );
        }
        if let Err(e) = install_single_app(raw_name, &options, &args).await {
            eprintln!(
                "{}",
                tr_fmt!(
                    "Failed to install '{name}': {e}",
                    "安装 '{name}' 失败: {e}",
                    name = raw_name,
                    e = e
                )
                .dark_red()
                .bold()
            );
            errors.push((raw_name.clone(), e));
        }
    }

    if !errors.is_empty() {
        if total > 1 {
            eprintln!(
                "{}",
                tr_fmt!(
                    "\n{failed} of {total} apps failed to install.",
                    "\n共有 {failed} 个应用（共 {total} 个）安装失败。",
                    failed = errors.len(),
                    total = total
                )
                .dark_red()
                .bold()
            );
            bail!(
                "{}",
                tr_fmt!(
                    "{failed} app(s) failed to install",
                    "{failed} 个应用安装失败",
                    failed = errors.len()
                )
            );
        } else {
            let (_, err) = errors.remove(0);
            return Err(err);
        }
    }

    Ok(())
}

async fn install_single_app(
    raw_name: &str,
    options: &[InstallOptions<'_>],
    args: &InstallArgs,
) -> Result<(), anyhow::Error> {
    let app_name = convert_path(raw_name.trim()).to_lowercase();
    if app_name == "hp" || app_name.ends_with("/hp") || app_name.starts_with("hp@") {
        bail!(
            "{}",
            tr(
                "hp cannot be installed with `hp install`; use `hp self-update` to update hp itself",
                "不能通过 `hp install` 安装 hp 自身；请使用 `hp self-update` 更新 hp"
            )
        );
    }
    let app_path = Path::new(&app_name);
    if app_path.exists() {
        if app_path.is_file() {
            log::debug!("manifest file {}", app_name);
            let manifest_path = app_name.as_str();
            if app_path.extension().unwrap_or_default() == "json" {
                install_app_from_local_manifest_file(manifest_path, options.to_vec(), None)?;
                return Ok(());
            } else {
                bail!(
                    "{}",
                    tr_fmt!(
                        "{path} is not a json file",
                        "{path} 不是 json 文件",
                        path = app_path.display()
                    )
                );
            }
        } else {
            if app_path.is_dir() {
                bail!(
                    "{}",
                    tr_fmt!(
                        "{path} is not a json file",
                        "{path} 不是 json 文件",
                        path = app_path.display()
                    )
                );
            } else {
                bail!(
                    "{}",
                    tr_fmt!(
                        "{path} is an incorrect file",
                        "{path} 是不正确的文件",
                        path = app_path.display()
                    )
                );
            }
        }
    }

    if is_valid_url(app_name.as_str()) {
        install_app_from_url(app_path, options, args.app_alias_from_url_install.clone())?;
        return Ok(());
    }

    if contains_special_char(app_name.as_str()) {
        bail!(
            "{}",
            tr(
                "Invalid app name format (contains invalid characters)",
                "指定的 APP 格式错误，包含非法字符"
            )
        );
    }

    if app_name.contains("/") {
        if app_name.contains('@') {
            bail!(
                "{}",
                tr(
                    "Invalid app format: cannot combine '/' and '@'",
                    "指定的 App 格式不正确：不能同时包含 '/' 和 '@'"
                )
            );
        }
        let split_arg = app_name.split('/').collect::<Vec<&str>>();
        if split_arg.iter().count() == 2 {
            let bucket = split_arg[0].trim().to_lowercase();
            let app_name = split_arg[1].trim().to_lowercase();
            if bucket.is_empty() || app_name.is_empty() {
                bail!(
                    "{}",
                    tr(
                        "Invalid app format: bucket or app name is empty",
                        "指定的 App 格式不正确：bucket 或应用名为空"
                    )
                );
            }
            install_from_specific_bucket(&bucket, &app_name, options)?;
            return Ok(());
        } else if split_arg.iter().count() > 2 || split_arg.len() == 1 {
            bail!(
                "{}",
                tr(
                    "Invalid app format: expected 'bucket/app'",
                    "指定的 APP 格式错误：期望 'bucket/app'"
                )
            );
        }
    }
    if app_name.contains('@') {
        let split_version = app_name.split('@').collect::<Vec<&str>>();
        if split_version.iter().count() == 2 {
            let app_name = split_version[0].trim().to_lowercase();
            let app_version = split_version[1].trim().to_lowercase();
            log::info!("install {} specific version {}", app_name, app_version);
            if app_name.is_empty() || app_version.is_empty() {
                bail!(
                    "{}",
                    tr(
                        "Invalid app format: app name or version is empty",
                        "指定的 APP 格式错误：应用名或版本为空"
                    )
                );
            }
            install_app_specific_version(&app_name, &app_version, &options.to_vec()).await?;
            return Ok(());
        } else if split_version.len() == 1 || split_version.len() > 2 {
            bail!(
                "{}",
                tr(
                    "Invalid app format: expected 'app@version'",
                    "指定的 APP 格式错误：期望 'app@version'"
                )
            );
        }
    }
    if contains_special_char(app_name.as_str()) {
        bail!(
            "{}",
            tr(
                "Invalid app name format (contains invalid characters)",
                "指定的 APP 格式错误，包含非法字符"
            )
        );
    }
    install_app(app_name.as_str(), options)?;
    Ok(())
}

pub fn inject_user_options(install_args: &InstallArgs) -> anyhow::Result<Vec<InstallOptions<'_>>> {
    let mut install_options = vec![];
    if let Some(arch) = install_args.arch.as_ref() {
        // as_ref 引用原始数据
        let arch = arch.trim();
        if arch != "64bit" && arch != "32bit" && arch != "arm64" {
            bail!(
                "{}",
                tr(
                    "Invalid arch option, please use 64bit, 32bit, or arm64",
                    "arch 格式错误, 请使用 64bit, 32bit, arm64"
                )
            );
        }
        if arch.is_empty() {
            let arch = get_system_default_arch()?;
            let arch = match arch.as_ref() {
                "64bit" => "64bit",
                "32bit" => "32bit",
                "arm64" => "arm64",
                _ => {
                    bail!(
                        "{}",
                        tr(
                            "Failed to get system default architecture",
                            "获取系统默认架构失败"
                        )
                    );
                }
            };
            install_options.push(InstallOptions::ArchOptions(arch));
        } else {
            install_options.push(InstallOptions::ArchOptions(arch));
        }
    }
    if install_args.only_download_no_install_with_override_cache {
        install_options.push(InstallOptions::ForceDownloadNoInstallOverrideCache)
    }
    if install_args.skip_download_hash_check {
        install_options.push(InstallOptions::SkipDownloadHashCheck)
    }
    if install_args.update_hp_and_buckets {
        install_options.push(InstallOptions::UpdateHpAndBuckets)
    }
    if install_args.no_use_download_cache {
        install_options.push(InstallOptions::NoUseDownloadCache)
    }
    if install_args.no_auto_download_dependencies {
        install_options.push(InstallOptions::NoAutoDownloadDepends)
    }
    if install_args.global {
        install_options.push(InstallOptions::Global)
    }
    if install_args.only_download_no_install {
        install_options.push(InstallOptions::OnlyDownloadNoInstall)
    }
    if install_args.check_version_up_to_date {
        install_options.push(InstallOptions::CheckCurrentVersionIsLatest)
    }

    if install_args.force_install_override {
        install_options.push(InstallOptions::ForceInstallOverride)
    }
    if install_args.interactive {
        install_options.push(InstallOptions::InteractiveInstall)
    }

    Ok(install_options)
}

fn contains_special_char(s: &str) -> bool {
    let special_chars = r#"!#$%^&*()+=\[]\{}|;':",<>?~`"#;
    s.chars().any(|c| special_chars.contains(c))
}

fn convert_path(path: &str) -> String {
    let path = path.replace("\\", "/");
    path
}
