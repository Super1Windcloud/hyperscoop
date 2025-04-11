use crate::Cli;
use anyhow::{anyhow, bail};
use clap::CommandFactory;
use crossterm::style::Stylize;
use reqwest::Client;
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

    let latest_version = get_latest_version_from_gitee().await?;
    let   latest_github_version = get_latest_version_from_github().await?;
     println!("Latest version: {}", latest_github_version);
    if version.to_string() < latest_version {
        println!("{}", format!("发现hp新版本 {latest_version},请访问https://gitee.com/SuperWindcloud/hyperscoop/releases").yellow().bold());
        Ok(true)
    } else {
        Ok(false)
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
struct GithubRelease {
  tag_name : String,
}
async fn  get_latest_version_from_github () -> anyhow::Result<String> {

  let  token = std::env::var("GITHUB_TOKEN").unwrap_or_default();
   let  local_token =  std::fs::read_to_string(".github_token").unwrap_or_default();
   if  token.is_empty() && local_token.is_empty() {
       bail!("GITHUB_TOKEN environment variable is empty") ;
   }
  let owner = "super1windcloud";
  let repo = "hp";
  let url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);

   let  response = reqwest::get(&url).await?;
  
  let tags: GithubRelease  = response.json().await.unwrap();
   
  Ok(tags.tag_name)
}
async fn get_latest_version_from_gitee() -> anyhow::Result<String> {
    let access_token = std::fs::read_to_string(".env")?.trim().to_string();
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
  use crate::check_self_update::{ get_latest_version_from_github};

  #[tokio::test]
    async fn test_auto_check_hp_update() {
        use super::auto_check_hp_update;
        auto_check_hp_update().await.unwrap();
    }

    #[tokio::test]
    async  fn  test_github_api (){
       println!("{}", get_latest_version_from_github().await.unwrap());
    }
}
