use anyhow::{Context, bail};
use command_util_lib::buckets::get_hp_bucket_repo_path;
use command_util_lib::config::get_config_value_no_print;
use command_util_lib::init_env::{
    get_app_current_dir, get_app_dir_manifest_json, get_app_dir_manifest_json_global,
};
use command_util_lib::install::UpdateOptions;
use command_util_lib::list::VersionJSON;
use command_util_lib::utils::git::pull_special_local_repo;
use command_util_lib::utils::utility::is_valid_url;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub fn get_app_old_version(app_name: &str, options: &[UpdateOptions]) -> anyhow::Result<String> {
    let old_install_manifest = if options.contains(&UpdateOptions::Global) {
        get_app_dir_manifest_json_global(app_name)
    } else {
        get_app_dir_manifest_json(app_name)
    };
    if !Path::new(&old_install_manifest).exists() {
        bail!("not found {} install manifest file", app_name)
    }
    let content = std::fs::read_to_string(&old_install_manifest)
        .context("failed to read old install manifest file")?;
    let version: VersionJSON = serde_json::from_str(content.as_str())
        .context("failed to parse old install manifest file")?;
    let version = version.version;
    if version.is_none() {
        bail!("not found version in old install manifest file")
    }
    Ok(version.unwrap())
}

pub fn is_version_newer(current: &str, latest: &str) -> bool {
    let parse_nums = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v')
            .split(['.', '-', '_'])
            .filter_map(|p| p.parse::<u64>().ok())
            .collect()
    };
    let cur_nums = parse_nums(current);
    let lat_nums = parse_nums(latest);
    if !cur_nums.is_empty() && !lat_nums.is_empty() {
        for (c, l) in cur_nums.iter().zip(lat_nums.iter()) {
            if l > c {
                return true;
            } else if l < c {
                return false;
            }
        }
        return lat_nums.len() > cur_nums.len();
    }
    latest > current
}

pub async fn auto_check_hp_update(old_version: Option<&str>) -> anyhow::Result<bool> {
    let mut version = match old_version {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => {
            get_app_old_version("hp", &[]).unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string())
        }
    };
    let current_pkg_ver = env!("CARGO_PKG_VERSION");
    if is_version_newer(&version, current_pkg_ver) {
        version = current_pkg_ver.to_string();
    }

    let github_version = get_latest_version_from_github().await.unwrap_or_default();
    let bucket_version = get_latest_app_version_from_local_bucket("hp").unwrap_or_default();

    let latest_version = if !github_version.is_empty() && !bucket_version.is_empty() {
        if is_version_newer(&bucket_version, &github_version) {
            github_version
        } else {
            bucket_version
        }
    } else if !github_version.is_empty() {
        github_version
    } else {
        bucket_version
    };

    log::debug!(
        "latest version: {} , current version: {}",
        latest_version,
        version
    );
    let is_newer = if !latest_version.is_empty() && !version.is_empty() {
        is_version_newer(&version, &latest_version)
    } else {
        false
    };
    if is_newer {
        if let Ok(Some(hp_repo_path)) = get_hp_bucket_repo_path("hp") {
            let _ = pull_special_local_repo(hp_repo_path.as_str());
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

use command_util_lib::manifest::manifest::{
    get_latest_app_version_from_local_bucket, get_latest_manifest_from_local_bucket,
};
use sha2::{Digest, Sha256};

#[allow(dead_code)]
pub fn hash_changed() -> bool {
    let hp_manifest = get_latest_manifest_from_local_bucket("hp").unwrap();
    let hp_current = get_app_current_dir("hp");
    let exe_path = Path::new(&hp_current).join("hp.exe");
    if !exe_path.exists() {
        return true;
    }

    let content = std::fs::read_to_string(&hp_manifest);
    if content.is_err() {
        return true;
    };
    let manifest: serde_json::Value = serde_json::from_str(&content.unwrap_or_default()).unwrap();
    let hash = manifest.get("hash").unwrap().as_str().unwrap();

    let mut hasher = Sha256::new();
    let buffer = std::fs::read(&exe_path).unwrap();
    hasher.update(&buffer);
    let hash_result = hasher.finalize();
    let old_hash = hex::encode(hash_result);
    if hash.to_lowercase() != old_hash.to_lowercase() {
        true
    } else {
        false
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
struct GithubRelease {
    tag_name: String,
}

async fn get_latest_version_from_github() -> anyhow::Result<String> {
    let owner = "Super1Windcloud";
    let repo = "hyperscoop";
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );
    let proxy_url = get_config_value_no_print("proxy");

    const USER_AGENT: &str = "Rust-GitHub-API-Client";
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, USER_AGENT.parse()?);
    headers.insert("Accept", "application/vnd.github.v3+json".parse()?);

    let client = if !proxy_url.is_empty() {
        // log::info!("Using proxy: {}", proxy_url);
        let proxy_url = if proxy_url.starts_with("http://") || proxy_url.starts_with("https://") {
            proxy_url
        } else {
            format!("http://{}", proxy_url)
        };
        if is_valid_url(&proxy_url) {
            let proxy = reqwest::Proxy::https(proxy_url)?;
            Client::builder().proxy(proxy).build()?
        } else {
            Client::builder().build()?
        }
    } else {
        Client::builder().build()?
    };

    let response = client.get(&url).headers(headers.clone()).send().await;

    if let Ok(resp) = response {
        if resp.status().is_success() {
            if let Ok(tags) = resp.json::<GithubRelease>().await {
                return Ok(tags.tag_name);
            }
        }
    }

    // Fallback: check redirect location of https://github.com/{owner}/{repo}/releases/latest
    // This endpoint is not subject to GitHub API 60 req/hour rate limits.
    let web_url = format!("https://github.com/{}/{}/releases/latest", owner, repo);
    let no_redirect_client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    if let Ok(resp) = no_redirect_client
        .get(&web_url)
        .headers(headers)
        .send()
        .await
    {
        if resp.status().is_redirection() {
            if let Some(loc) = resp.headers().get(header::LOCATION) {
                if let Ok(loc_str) = loc.to_str() {
                    if let Some(tag) = loc_str.split("/tag/").last() {
                        return Ok(tag.trim().to_string());
                    }
                }
            }
        }
    }

    Ok("".into())
}

mod test_auto_update {
    #[allow(unused)]
    use super::*;
    #[allow(unused_imports)]
    use rust_i18n::t;

    #[test]
    fn test_is_version_newer() {
        assert!(!is_version_newer("4.2.5", "4.2.5"));
        assert!(!is_version_newer("v4.2.5", "4.2.5"));
        assert!(!is_version_newer("4.2.5", "v4.2.5"));
        assert!(!is_version_newer("4.2.6", "4.2.5"));
        assert!(is_version_newer("4.2.4", "4.2.5"));
        assert!(is_version_newer("4.2.5", "4.2.6"));
        assert!(is_version_newer("4.2.5", "4.3.0"));
        assert!(is_version_newer("4.2.5", "5.0.0"));
        assert!(is_version_newer("4.2.5", "4.2.5.1"));
        assert!(!is_version_newer("4.2.5.1", "4.2.5"));
        assert!(!is_version_newer("4.2.5", ""));
    }

    #[tokio::test]
    async fn test_auto_check_hp_update() {
        use super::auto_check_hp_update;
        let _ = auto_check_hp_update(None).await;
    }

    #[tokio::test]
    async fn test_github_api() {
        let owner = "super1windcloud";
        let repo = "hp";
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        );
        let client = Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "Rust-GitHub-API-Client")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .unwrap();

        if !response.status().is_success() {
            log::debug!(
                "{}",
                t!("network.request_failed", status = response.status())
            );
        }
        let tags: GithubRelease = response.json().await.unwrap();
        println!("{}", tags.tag_name);
    }

    #[test]
    fn test_old_version() {
        let old_version = get_app_old_version("hp", &vec![]);
        println!("{:?}", old_version);
    }

    #[test]
    #[should_panic(expected = "not found file exception")]
    fn test_context_throw() {
        fn read_file(path: &str) -> anyhow::Result<String> {
            let result =
                std::fs::read_to_string(path).context(format!("failed to read file {}", path))?;
            Ok(result)
        }

        let _result = read_file("not_exist_file").expect("not found file exception");
    }
}
