use anyhow::{anyhow };
use clap::CommandFactory;
use crossterm::style::Stylize;
use reqwest::Client;
use serde::Deserialize;
use crate::Cli;

#[allow(clippy::unsafe_derive_deserialize)]
#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize )]
struct GiteeRelease {
  tag_name: String,
  name : Option <String>, 
  body : Option <String> , 
}


pub async  fn auto_check_hp_update  ( ) ->anyhow::Result<()> { 
   let  cmd =Cli::command(); 
   let version =   cmd. get_version().ok_or(anyhow!("hp version is empty"))?;
  
   let  latest_version = get_latest_version_from_gitee().await?;
   
   if version .to_string() < latest_version {
      println!("{}", format!("发现hp新版本 {latest_version},请访问https://gitee.com/SuperWindcloud/hyperscoop/releases").yellow().bold());
   }
   Ok(() )
}

async fn get_latest_version_from_gitee() -> anyhow::Result<String> {
   
   let  access_token =include_str!("../.env") .trim().to_string(); 
  let   client =  Client::new();  
  let response = client  .get("https://gitee.com/api/v5/repos/superwindcloud/hyperscoop/releases/latest")
    .header("Content-Type", "application/json;charset=UTF-8")
    .query(&[("access_token", access_token )])
    .send()
    .await?;
  if !response.status().is_success() {
    return Err(anyhow::anyhow!("请求失败: {}", response.status()));
  } 
   let release = response.json::<GiteeRelease>().await?;  
   Ok(release.tag_name)
}

mod  test_auto_update{

  #[tokio::test]
  async  fn test_auto_check_hp_update () {
      use  super::auto_check_hp_update;
      auto_check_hp_update().await.unwrap() ;

  }
}
