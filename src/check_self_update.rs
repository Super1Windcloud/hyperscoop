#[allow(unused_imports)]
use crate::crypto::decrypt_gitee;
#[allow(unused_imports)]
use crate::crypto::decrypt_github;
use crate::Cli;
use anyhow::{anyhow, bail, Context};
use clap::CommandFactory;
use command_util_lib::buckets::get_hp_bucket_repo_path;
use command_util_lib::config::get_config_value_no_print;
use command_util_lib::init_env::{get_app_dir_manifest_json, get_app_dir_manifest_json_global};
use command_util_lib::install::UpdateOptions;
use command_util_lib::list::VersionJSON;
use command_util_lib::utils::git::pull_special_local_repo;
use command_util_lib::utils::utility::is_valid_url;
use crossterm::style::Stylize;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[allow(clippy::unsafe_derive_deserialize)]
#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize)]

struct GiteeRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
}

pub fn get_app_old_version(app_name: &str, options: &[UpdateOptions]) -> anyhow::Result<String> {
    let  old_install_manifest = if options.contains(&UpdateOptions::Global) {
        get_app_dir_manifest_json_global(app_name)
    } else {
        get_app_dir_manifest_json(app_name)
    };
    if !Path::new(&old_install_manifest).exists() {
        bail!("not found {} install manifest file", app_name)
    }
    let content = std::fs::read_to_string(&old_install_manifest).context(
        "failed to read old install manifest file",
    )?;
    let version: VersionJSON = serde_json::from_str(content.as_str()).
      context("failed to parse old install manifest file")?;
    let version = version.version;
    if version.is_none() {
        bail!("not found version in old install manifest file")
    }
    Ok(version.unwrap())
}

pub async fn auto_check_hp_update(old_version: Option<&str>) -> anyhow::Result<bool> {
    let cmd = Cli::command();
    let version = if old_version.is_some() {
        old_version.unwrap()
    } else {
        cmd.get_version().ok_or(anyhow!("hp version is empty"))?
    };
    let latest_github_version = get_latest_version_from_github()
        .await.map_err(|e| anyhow!("failed to get latest github version: {}", e))?;
    let latest_version = if latest_github_version.is_empty() {
        get_latest_version_from_gitee()
            .await
            .map_err(|e| anyhow!("failed to get latest gitee version: {}", e))?
    } else {
        latest_github_version
    };
  
    if version.to_string() < latest_version {
        println!("{}", format!("发现hp新版本 {latest_version}, `hp u hp` or `hp u -f -k hp`  \n请访问https://github.com/Super1Windcloud/hp/releases").dark_cyan().bold());
        let hp_repo = get_hp_bucket_repo_path("hp")?;
        if hp_repo.is_none() {
            bail!("hp bucket repository  is empty");
        }
        let hp_repo_path = hp_repo.unwrap();
        pull_special_local_repo(hp_repo_path.as_str())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
struct GithubRelease {
    tag_name: String,
}

#[cfg(token_local)]
async fn get_latest_version_from_github() -> anyhow::Result<String> {
    let token = include_str!("../.github_token").trim();
    if token.is_empty() {
        bail!("GITHUB_TOKEN environment variable is empty");
    }
  
  
    let owner = "super1windcloud";
    let repo = "hp";
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );
    const USER_AGENT: &str = "Rust-GitHub-API-Client";
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, USER_AGENT.parse()?);
    headers.insert("Accept", "application/vnd.github.v3+json".parse()?);
    headers.insert(header::AUTHORIZATION, format!("token {}", token).parse()?);

    let proxy_url = get_config_value_no_print("proxy");

    let client = if !proxy_url.is_empty() {
        let proxy_url = if proxy_url.starts_with("http://") || proxy_url.starts_with("https://") {
            proxy_url
        } else {
            format!("http://{}", proxy_url)
        };
        is_valid_url(&proxy_url);
        let proxy = reqwest::Proxy::https(proxy_url)?;
        Client::builder().proxy(proxy).build()?
    } else {
        Client::builder().build()?
    };

    let response = client.get(&url).headers(headers).send().await.context(
      format!("failed to fetch GitHub-API-REQUEST {}", url),
    )?;

    let tags: GithubRelease = response.json().await .context(
      format!("failed to parse response data from {}", url),
    )?;
    Ok(tags.tag_name)
}

#[cfg(not(token_local))]
async fn get_latest_version_from_github() -> anyhow::Result<String> {
    let owner = "super1windcloud";
    let repo = "hp";
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );
    let token = decrypt_github().expect("failed to decrypt github version");
    let proxy_url = get_config_value_no_print("proxy");

    const USER_AGENT: &str = "Rust-GitHub-API-Client";
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, USER_AGENT.parse()?);
    headers.insert("Accept", "application/vnd.github.v3+json".parse()?);
    headers.insert(header::AUTHORIZATION, format!("token {}", token).parse()?);

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

    let response = client.get(&url).headers(headers).send().await?;

    if !response.status().is_success() {
        eprintln!("请求失败: {}", response.status());
        eprintln!("Response: {}", response.text().await?);
        return Ok("".into());
    }
    let tags: GithubRelease = response.json().await?;
    Ok(tags.tag_name)
}

#[cfg(token_local)]
async fn get_latest_version_from_gitee() -> anyhow::Result<String> {
    let access_token = include_str!("../.env").trim();
    if access_token.is_empty() {
        bail!("GITEE_TOKEN environment variable is empty");
    }
    let client = Client::new();
    let response = client
        .get("https://gitee.com/api/v5/repos/superwindcloud/hyperscoop/releases/latest")
        .header("Content-Type", "application/json;charset=UTF-8")
        .query(&[("access_token", access_token)])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("请求失败: {}", response.status()));
    }
    let release = response.json::<GiteeRelease>().await?;
    let gitee_tag = release.tag_name;

    Ok(gitee_tag)
}

#[cfg(not(token_local))]
async fn get_latest_version_from_gitee() -> anyhow::Result<String> {
    let access_token = decrypt_gitee()?;
    let client = Client::new();
    let response = client
        .get("https://gitee.com/api/v5/repos/superwindcloud/hyperscoop/releases/latest")
        .header("Content-Type", "application/json;charset=UTF-8")
        .query(&[("access_token", access_token)])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("请求失败: {}", response.status()));
    }
    let release = response.json::<GiteeRelease>().await?;
    let gitee_tag = release.tag_name;

    Ok(gitee_tag)
}

mod test_auto_update {
    #[allow(unused)]
    use super::*;

    #[tokio::test]
    async fn test_auto_check_hp_update() {
        use super::auto_check_hp_update;
        auto_check_hp_update(None).await.unwrap();
    }

    #[tokio::test]
    async fn test_github_api() {
        let _token = "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";
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
            eprintln!("请求失败: {}", response.status());
        }
        let tags: GithubRelease = response.json().await.unwrap();
        println!("{}", tags.tag_name);
    }

    #[test]
    fn test_old_version() {
        let old_version = get_app_old_version("hp", &vec![]).unwrap();
        println!("{}", old_version);
    }
  
  
  #[test]
  fn test_context_throw(){  
    
    fn  read_file(path: &str) -> anyhow::Result<String> {
       let result =   std::fs::read_to_string(path).context(format!("failed to read file {}", path))?; 
       Ok(result)
    }
    
     let _result = read_file("not_exist_file").expect("not found file exception");
    
  }
}
