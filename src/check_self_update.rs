use crate::Cli;
use anyhow::{anyhow, bail};
use clap::CommandFactory;
use crossterm::style::Stylize;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
#[allow(clippy::unsafe_derive_deserialize)]
#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize)]

struct GiteeRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
}

pub async fn auto_check_hp_update() -> anyhow::Result<bool> {
    let cmd = Cli::command();
    let version = cmd.get_version().ok_or(anyhow!("hp version is empty"))?;
    let latest_github_version = get_latest_version_from_github().await?;

    if version.to_string() < latest_github_version {
        println!("{}", format!("发现hp新版本 {latest_github_version},请访问https://github.com/Super1Windcloud/hp/releases").yellow().bold());
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
    log::info!("Getting latest version from github");
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
    const USER_AGENT: &str = "super1windcloud";
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, USER_AGENT.parse()?);
    headers.insert(header::AUTHORIZATION, format!("token {}", token).parse()?);
    let client = Client::builder().default_headers(headers).build()?;

    let response = client.get(&url).send().await?;
    let tags: GithubRelease = response.json().await?;
    Ok(tags.tag_name)
}


#[cfg(not(token_local))]
async fn get_latest_version_from_github() -> anyhow::Result<String> {
    // 该token只具备读权限  
    let token =  "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";
    let owner = "super1windcloud";
    let repo = "hp";
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );
   const USER_AGENT: &str = "super1windcloud";
   let mut headers = header::HeaderMap::new();
   headers.insert(header::USER_AGENT, USER_AGENT.parse()?);
   headers.insert(header::AUTHORIZATION, format!("token {}", token).parse()?);
   let client = Client::builder().default_headers(headers).build()?;
    let response = client.get(&url).send().await?;
    let tags: GithubRelease = response.json().await?;
    Ok(tags.tag_name)
}

#[cfg(token_local)]
async fn _get_latest_version_from_gitee() -> anyhow::Result<String> {
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
    let access_token = std::env::var("GITEE_TOKEN");
    if access_token.is_err() {
        bail!("GITEE_TOKEN environment variable is empty");
    }
    let access_token = access_token?.trim().to_string();
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

    #[tokio::test]
    async fn test_auto_check_hp_update() {
        use super::auto_check_hp_update;
        auto_check_hp_update().await.unwrap();
    }
    #[tokio::test]
    async fn test_github_api() {
        use super::*;
        let github_version = get_latest_version_from_github().await.unwrap();
        println!("Latest  github version: {}", github_version);
    }
}
