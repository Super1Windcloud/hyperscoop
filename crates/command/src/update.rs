use std::path::Path;
#[allow(unused_imports)]
use anyhow::bail;
use tokio::runtime::Runtime;
use crate::buckets::{get_buckets_name, get_buckets_path};
use crate::utils::git::{local_scoop_latest_commit, remote_latest_scoop_commit};
use crate::utils::request::get_git_repo_remote_url;

pub fn update_all_apps() -> Result<(), anyhow::Error> {
  Ok(())
}

pub fn update_specific_app_without_cache(app_name: String) -> Result<(), anyhow::Error> {
  log::trace!("update_specific_app_without_cache");
  Ok(())
}

pub fn update_specific_app_without_hash_check(app_name: String) -> Result<(), anyhow::Error> {
  log::trace!("update_specific_app_without_hash_check");
  Ok(())
}

pub fn update_specific_app_without_cache_and_hash_check(app_name: String) -> Result<(), anyhow::Error> {
  log::trace!("update_specific_app_without_cache_and_hash_check");
  Ok(())
}


pub fn update_specific_app(app_name: String) -> Result<(), anyhow::Error> {
  log::trace!("update_specific_app {}", &app_name );
  Ok(())
}

async fn check_scoop_update() -> anyhow::Result<bool > {
  let  remote_head = remote_latest_scoop_commit().await?;
 let  local_head =  local_scoop_latest_commit().expect("failed to get local_scoop_latest_commit");
  if remote_head == local_head {
    log::trace!("Scoop is up to date"); 
    return Ok(false);
  }
  log::trace!("Scoop is not up to date");
 Ok(true )
}

pub async fn update_scoop_bar() -> anyhow::Result<()> {
  use crate::utils::progrees_bar::{
    indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle},
    style, Message, ProgressOptions,
  };
  use std::thread;
  use std::time::Duration;
  const FINISH_MESSAGE: &'static str = "✅";
  log::trace!("update_scoop_bar");

  let progress_style = style(Some(ProgressOptions::PosLen), Some(Message::suffix()));
  let buckets_name = get_buckets_name()?;

  let longest_bucket_name =
    buckets_name.iter().map(|item| item.len()).max().unwrap_or(0);
  let pb = ProgressBar::new(1)
    .with_style(progress_style)
    .with_message("Checking for updates")
    .with_prefix(format!("🍨 {:<longest_bucket_name$}", "Scoop"))
    .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into()));

  let scoop_status = check_scoop_update().await?;
  
  if  !scoop_status {
    pb.finish_with_message("✅ No updates available");
    return Ok(());
  }
  Ok(())
}
