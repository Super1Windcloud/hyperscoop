use crate::config::get_config_value_no_print;
use crate::i18n::tr;
use crate::init_env::{
    get_app_dir, get_app_dir_global, get_app_dir_manifest_json, get_app_dir_manifest_json_global,
};
use crate::install::UpdateOptions;
use crate::install::UpdateOptions::Global;
use crate::list::VersionJSON;
use crate::manifest::manifest::{
    get_latest_app_version_from_local_bucket, get_latest_app_version_from_local_bucket_global,
};
use crate::tr_fmt;
use crate::utils::utility::{get_official_bucket_path, get_official_buckets_name};
use anyhow::{Context, bail};
use crossterm::style::Stylize;
use git2::{FetchOptions, ProxyOptions, Repository};
use rayon::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub fn check_bucket_update_status<'a>() -> anyhow::Result<bool> {
    let official_buckets = get_official_buckets_name();
    let official_buckets_path = official_buckets
        .iter()
        .map(|b| get_official_bucket_path(b.clone()))
        .collect::<Vec<_>>();

    let status_flag = Arc::new(Mutex::new(false));
    let result: anyhow::Result<()> = official_buckets_path.par_iter().try_for_each(|path| {
        let mut proxy_options = ProxyOptions::new();

        let config_proxy = get_config_value_no_print("proxy");
        if !config_proxy.is_empty() {
            let proxy_url =
                if config_proxy.starts_with("http://") || config_proxy.starts_with("https://") {
                    config_proxy.clone()
                } else {
                    format!("http://{}", config_proxy)
                };

            proxy_options.url(&proxy_url);
            // log::info!("Using proxy: {}", proxy_url);
        }

        let mut fetch_options = FetchOptions::new();
        fetch_options
            .download_tags(git2::AutotagOption::All)
            .proxy_options(proxy_options); // 应用代理配置

        let repo = Repository::open(&path)
            .with_context(|| format!("Failed to open repository at {}", path))?;

        let mut remote = repo
            .find_remote("origin")
            .with_context(|| format!("Failed to find remote 'origin' in {}", path))?;

        remote
            .fetch::<&str>(&[], Some(&mut fetch_options), None)
            .with_context(|| format!("Failed to fetch from remote for {}", path))?;

        let local_head = repo
            .head()?
            .target()
            .with_context(|| format!("Failed to get local HEAD for {}", path))?;

        let remote_head = repo
            .refname_to_id("refs/remotes/origin/HEAD")
            .with_context(|| format!("Failed to get remote HEAD for {}", path))?;

        if local_head != remote_head {
            *status_flag.lock().unwrap() = true;
        }
        Ok(())
    });

    if result.is_err() {
        bail!(result.unwrap_err())
    }
    let flag = *status_flag.lock().unwrap();
    if !flag {
        println!(
            "{}",
            tr("All buckets are up to date", "所有 buckets 均为最新版本")
                .dark_green()
                .bold()
        );
    } else {
        println!(
            "{}",
            tr(
                "Some buckets have updates available",
                "部分 buckets 有可用更新"
            )
            .dark_green()
            .bold()
        );
    }
    Ok(flag)
}

pub fn check_app_version_latest(
    app_name: &str,
    options: &[UpdateOptions],
) -> anyhow::Result<Option<String>> {
    if options.contains(&UpdateOptions::ForceUpdateOverride) {
        return Ok(None);
    }
    let app_dir = if options.contains(&Global) {
        get_app_dir_global(app_name)
    } else {
        get_app_dir(app_name)
    };
    if !Path::new(&app_dir).exists() {
        bail!(
            "{}",
            tr_fmt!(
                "App '{app_name}' is not installed",
                "应用 '{app_name}' 未安装",
                app_name = app_name
            )
        );
    }
    let manifest_path = if options.contains(&Global) {
        get_app_dir_manifest_json_global(app_name)
    } else {
        get_app_dir_manifest_json(app_name)
    };
    if !Path::new(&manifest_path).exists() {
        bail!(
            "{}",
            tr_fmt!(
                "Manifest path {manifest_path} does not exist",
                "Manifest 路径 {manifest_path} 不存在",
                manifest_path = manifest_path
            )
        );
    }
    let content =
        std::fs::read_to_string(&manifest_path).context("Failed to read manifest.json")?;
    let version: VersionJSON =
        serde_json::from_str(&content).context("Failed to parse manifest.json")?;
    let old_version = version.version.ok_or(0);
    match old_version {
        Err(_) => {
            bail!(
                "{}",
                tr(
                    "Version info not found for this app, manifest.json format error",
                    "该App没有找到版本信息,manifest.json格式错误"
                )
            );
        }
        Ok(old_version) => {
            let latest_version = if options.contains(&Global) {
                get_latest_app_version_from_local_bucket_global(app_name)?
            } else {
                get_latest_app_version_from_local_bucket(app_name)?
            };
            if old_version == latest_version {
                Ok(Some(old_version))
            } else {
                Ok(None)
            }
        }
    }
}

pub fn should_auto_update_buckets() -> bool {
    if std::env::var("HYPERSCOOP_NO_AUTO_UPDATE").is_ok()
        || std::env::var("HOMEBREW_NO_AUTO_UPDATE").is_ok()
    {
        return false;
    }

    let last_update_str = get_config_value_no_print("last_update");
    if last_update_str.is_empty() {
        return true;
    }

    let threshold_secs: i64 = std::env::var("HYPERSCOOP_AUTO_UPDATE_SECS")
        .or_else(|_| std::env::var("HOMEBREW_AUTO_UPDATE_SECS"))
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(86400);

    if let Ok(last_time) = chrono::DateTime::parse_from_rfc3339(&last_update_str) {
        let now = chrono::Local::now();
        let elapsed = now.signed_duration_since(last_time.with_timezone(&chrono::Local));
        elapsed.num_seconds() >= threshold_secs
    } else {
        true
    }
}

mod test {
    #[allow(unused)]
    use super::*;

    #[test]
    fn check_update() {
        let _ = check_bucket_update_status();
    }
}
