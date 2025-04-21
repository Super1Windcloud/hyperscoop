use crate::config::get_config_value_no_print;
use crate::init_env::{
    get_app_dir, get_app_dir_global, get_app_dir_manifest_json, get_app_dir_manifest_json_global,
};
use crate::install::UpdateOptions;
use crate::install::UpdateOptions::Global;
use crate::list::VersionJSON;
use crate::manifest::manifest::{
    get_latest_app_version_from_local_bucket, get_latest_app_version_from_local_bucket_global,
};
use crate::utils::utility::{get_official_bucket_path, get_official_buckets_name};
use anyhow::{bail, Context};
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
            "All Buckets are up to date".to_string().dark_green().bold()
        );
    } else {
        println!(
            "{}",
            "Some Buckets are outData and has updates available"
                .to_string()
                .dark_green()
                .bold()
        );
    }
    Ok(flag)
}

pub fn check_app_version_latest(app_name: &str, options: &[UpdateOptions]) -> anyhow::Result<bool> {
    let app_dir = if options.contains(&Global) {
        get_app_dir_global(app_name)
    } else {
        get_app_dir(app_name)
    };
    if !Path::new(&app_dir).exists() {
        bail!("Not found for '{}',App并未安装", app_name);
    }
    let manifest_path = if options.contains(&Global) {
        get_app_dir_manifest_json_global(app_name)
    } else {
        get_app_dir_manifest_json(app_name)
    };
    if !Path::new(&manifest_path).exists() {
        bail!("Manifest path {} does not exist", manifest_path);
    }
    let content = std::fs::read_to_string(&manifest_path)?;
    let version: VersionJSON = serde_json::from_str(&content)?;
    let old_version = version.version.ok_or(0);
    match old_version {
        Err(_) => {
            bail!("该App没有找到版本信息,manifest.json格式错误")
        }
        Ok(old_version) => {
            let latest_version = if options.contains(&Global) {
                get_latest_app_version_from_local_bucket_global(app_name)?
            } else {
                get_latest_app_version_from_local_bucket(app_name)?
            };
            if old_version == latest_version {
                Ok(true)
            } else {
                Ok(false)
            }
        }
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
