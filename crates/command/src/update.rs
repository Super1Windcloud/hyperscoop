#[allow(unused_imports)]
use anyhow::bail;
use rayon::prelude::* ;
use tokio::runtime::Runtime;
use crate::buckets::{get_buckets_name, get_buckets_path};
use crate::utils::git::{git_pull_update_repo, git_pull_update_repo_with_scoop, local_scoop_latest_commit, remote_latest_scoop_commit};

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

use crate::utils::progrees_bar::{gen_stats_callback, indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle}, style, Message, ProgressOptions};
use crate::utils::utility::get_official_bucket_path;

const FINISH_MESSAGE: &'static str = "✅";

pub async fn update_scoop_bar() -> anyhow::Result<()> {


  let progress_style = style(Some(ProgressOptions::PosLen), Some(Message::suffix()));
  let buckets_name = get_buckets_name()?;

  let longest_bucket_name =
    buckets_name.iter().map(|item| item.len()).max().unwrap_or(0);
  let pb = ProgressBar::new(1)
    .with_style(progress_style)
    .with_message("Checking for updates")
    .with_prefix(format!("🐧{:<10}", "Scoop "))
    .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into()));

  let scoop_status = check_scoop_update().await?;

  if  !scoop_status {
    pb.finish_with_message("✅ No updates available");
    return Ok(());
  }
  let callback = gen_stats_callback(&pb);
  git_pull_update_repo_with_scoop(callback)?;
  pb.finish_with_message(FINISH_MESSAGE) ;
  Ok(())
}


pub   fn update_all_buckets_bar(official_buckets: Vec<String>) -> anyhow::Result<()> {
  let progress_style = style(Some(ProgressOptions::Hide), Some(Message::suffix()));
  let longest_bucket_name =
    official_buckets.iter().map(|item| item.len()).max().unwrap_or(0);
  let mp = MultiProgress::new();
  let outdated_buckets = official_buckets
    .into_iter()
    .map(|bucket| {
      let bucket_path =get_official_bucket_path(bucket.clone());

      let pb =  mp.add(
          ProgressBar::new(1)
            .with_style(progress_style.clone())
            .with_message("Checking updates")
            .with_prefix(format!("🐼 {:<longest_bucket_name$}", bucket))
            .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into()))
      );
      pb.set_position(0);
      ( pb,  bucket_path )
    }).collect::<Vec<_>>();
   let _  =  outdated_buckets.par_iter()
    .map(  |( pb , bucket_path )|  {
       let callback = gen_stats_callback(pb);
         let  result  = git_pull_update_repo(bucket_path  ,&callback);
        if let Err(e) = result {
          pb.finish_with_message(format!("❌ {}", e.to_string()));
          return Err(e);
        }
       pb.finish_with_message(FINISH_MESSAGE) ;
       Ok(())
      }).collect::<Vec<anyhow::Result<()>>>();
  /// replace rayon  iterator  running  with foreach  for  map  method
   // outdated_buckets.par_iter() // 来自 rayon
  //     .for_each(|(pb, bucket_path)| {
  //         let callback = gen_stats_callback(pb);
  //         let result = git_pull_update_repo(bucket_path, &callback);
  //         if let Err(e) = result {
  //             pb.finish_with_message(format!("❌ {}", e));
  //         } else {
  //             pb.finish_with_message(FINISH_MESSAGE);
  //         }
  //     });
  Ok(())
}
