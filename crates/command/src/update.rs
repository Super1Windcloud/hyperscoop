use crate::buckets::{get_buckets_name, get_buckets_path};
use crate::utils::git::{
    git_pull_update_repo, git_pull_update_repo_with_scoop, local_scoop_latest_commit,
    remote_latest_scoop_commit,
};
#[allow(unused_imports)]
use anyhow::bail;
use rayon::prelude::*;
use tokio::runtime::Runtime;

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

pub fn update_specific_app_without_cache_and_hash_check(
    app_name: String,
) -> Result<(), anyhow::Error> {
    log::trace!("update_specific_app_without_cache_and_hash_check");
    Ok(())
}

pub fn update_specific_app(app_name: String) -> Result<(), anyhow::Error> {
    log::trace!("update_specific_app {}", &app_name);
    Ok(())
}

async fn check_scoop_update() -> anyhow::Result<bool> {
    let remote_head = remote_latest_scoop_commit().await?;
    let local_head = local_scoop_latest_commit().expect("failed to get local_scoop_latest_commit");
    if remote_head == local_head {
        log::trace!("Scoop is up to date");
        return Ok(false);
    }
    log::trace!("Scoop is not up to date");
    Ok(true)
}

use crate::utils::progrees_bar::{
    gen_stats_callback,
    indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle},
    style, Message, ProgressOptions,
};
use crate::utils::request::get_git_repo_remote_url;
use crate::utils::utility::get_official_bucket_path;

const FINISH_MESSAGE: &'static str = "✅";

pub async fn update_scoop_bar() -> anyhow::Result<()> {
    let progress_style = style(Some(ProgressOptions::PosLen), Some(Message::suffix()));
    let buckets_name = get_buckets_name()?;

    let longest_bucket_name = buckets_name
        .iter()
        .map(|item| item.len())
        .max()
        .unwrap_or(0);
    let pb = ProgressBar::new(1)
        .with_style(progress_style)
        .with_message("Checking for updates")
        .with_prefix(format!("🐧{:<10}", "Scoop "))
        .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into()));

    let scoop_status = check_scoop_update().await?;

    if !scoop_status {
        pb.finish_with_message("✅ No updates available");
        return Ok(());
    }
    let callback = gen_stats_callback(&pb);
    git_pull_update_repo_with_scoop(callback)?;
    pb.finish_with_message(FINISH_MESSAGE);
    Ok(())
}

pub fn update_all_buckets_bar() -> anyhow::Result<()> {
    let progress_style = style(Some(ProgressOptions::Hide), Some(Message::suffix()));
    let official_buckets = get_include_buckets_name()?;
    let longest_bucket_name = official_buckets
        .iter()
        .map(|item| item.len())
        .max()
        .unwrap_or(0);
    let mp = MultiProgress::new();
    let outdated_buckets = official_buckets
        .into_iter()
        .map(|bucket| {
            let bucket_path = get_official_bucket_path(bucket.clone());

            let pb = mp.add(
                ProgressBar::new(1)
                    .with_style(progress_style.clone())
                    .with_message("Checking updates")
                    .with_prefix(format!("🐼 {:<longest_bucket_name$}", bucket))
                    .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into())),
            );
            pb.set_position(0);
            (pb, bucket_path)
        })
        .collect::<Vec<_>>();
    let _ = outdated_buckets
        .par_iter()
        .map(|(pb, bucket_path)| {
            let callback = gen_stats_callback(pb);
            let result = git_pull_update_repo(bucket_path, &callback);
            if let Err(e) = result {
                pb.finish_with_message(format!("❌ {}", e.to_string()));
                return Err(e);
            }
            pb.finish_with_message(FINISH_MESSAGE);
            Ok(())
        })
        .collect::<Vec<anyhow::Result<()>>>();
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

pub fn get_include_buckets_name() -> anyhow::Result<Vec<String>> {
    let bucket_path = get_buckets_path()?;
    let mut finial_bucket_path: Vec<String> = Vec::new();
    let large_community_bucket = [
        "https://github.com/anderlli0053/DEV-tools",
        "https://github.com/duzyn/scoop-cn",
        "https://github.com/lzwme/scoop-proxy-cn",
        "https://github.com/kkzzhizhou/scoop-apps",
        "https://github.com/cmontage/scoopbucket-third",
        "https://github.com/okibcn/ScoopMaster",
        "http://github.com/okibcn/ScoopMaster",
    ];
    for path in bucket_path.iter() {
        let url = get_git_repo_remote_url(path)?; 
        if !large_community_bucket.iter().any(|&x| x == url) {
            finial_bucket_path.push(path.into());
        }
    }
    let finial_bucket_name = finial_bucket_path
        .iter()
        .map(|item| item.split("\\").last().unwrap())
        .collect::<Vec<&str>>();
    Ok(finial_bucket_name
        .into_iter()
        .map(|item| item.to_string())
        .collect())
}
pub fn get_include_buckets_path() -> anyhow::Result<Vec<String>> {
    let bucket_path = get_buckets_path()?;
    let mut final_bucket_path: Vec<String> = Vec::new();
    let large_community_bucket = [
        "https://github.com/anderlli0053/DEV-tools",
        "https://github.com/cmontage/scoopbucket",
        "https://github.com/duzyn/scoop-cn",
        "https://github.com/lzwme/scoop-proxy-cn",
        "https://github.com/kkzzhizhou/scoop-apps",
        "https://github.com/cmontage/scoopbucket-third",
        "https://github.com/okibcn/ScoopMaster",
      "http://github.com/okibcn/ScoopMaster",
    ];
    final_bucket_path = bucket_path
        .iter()
        .filter(|path| {
            let url = get_git_repo_remote_url(path).unwrap_or_default();
            !large_community_bucket.contains(&url.as_str())
        })
        .cloned()
        .collect();

    Ok(final_bucket_path)
}

mod tests {
    use super::*;

    #[test]
    fn test_bucket_name() {
        let official_buckets = get_buckets_name().unwrap();
        println!("Official buckets: {:?}", official_buckets);
    }
    #[test]
    fn test_include_buckets_name() {
        let official_buckets = get_include_buckets_name().unwrap();
        println!(
            "Official buckets: {:? } {:?}",
            official_buckets.len(),
            official_buckets
        );
    }
    #[test]
    fn test_finail_bucket_path() {
        let official_buckets = get_include_buckets_path().unwrap();
        println!(
            "Official buckets: {:? } {:?}",
            official_buckets.len(),
            official_buckets
        );
    }
}
