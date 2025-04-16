use std::fs;
use crate::buckets::{get_buckets_name, get_buckets_path};
use crate::utils::git::{
    git_pull_update_repo, git_pull_update_repo_with_scoop, local_scoop_latest_commit,
    remote_latest_scoop_commit,
};
pub(crate) mod update;
use crate::init_env::{
    get_app_current_dir, get_app_current_dir_global, get_apps_path, get_apps_path_global,
};
use crate::install::UpdateOptions::{Global, RemoveOldVersionApp};
use crate::install::{install_app, InstallOptions, UpdateOptions};
use crate::list::get_all_installed_apps_name;
use crate::utils::progrees_bar::{gen_stats_callback, ProgressOptions};
use crate::utils::progrees_bar::{
    indicatif::{MultiProgress, ProgressBar, ProgressFinish},
    style, Message,
};
use crate::utils::request::get_git_repo_remote_url;
use crate::utils::utility::get_official_bucket_path;
#[allow(unused_imports)]
use anyhow::bail;
use rayon::prelude::*;
pub use update::*;

const FINISH_MESSAGE: &str = "✅";

pub async fn update_all_apps(options: &[UpdateOptions]) -> Result<(), anyhow::Error> {
    let all_apps_name = get_all_installed_apps_name();
    let origin_options = options.to_vec();
    let options = transform_update_options_to_install(options);
    for app in all_apps_name {
        if origin_options.contains(&RemoveOldVersionApp) {
            remove_old_version(&app, &origin_options)?;
        }
        install_app(&app, options.as_ref()).await?;
    }
    Ok(())
}

pub fn remove_old_version(app_name: &str, options: &[UpdateOptions]) -> anyhow::Result<()> {
    let app_current_dir = if options.contains(&Global) {
        get_app_current_dir_global(app_name)
    } else {
        get_app_current_dir(app_name)
    };
    let  target_version_path = fs::read_link(app_current_dir)?; 
    log::debug!("target_version_path: {:?}", target_version_path);
    fs::remove_dir_all(target_version_path)?;
    Ok(())
}
fn transform_update_options_to_install(update_options: &[UpdateOptions]) -> Vec<InstallOptions> {
    let mut options = vec![];
    if update_options.contains(&Global) {
        options.push(InstallOptions::Global);
    }
    if update_options.contains(&UpdateOptions::UpdateHpAndBuckets) {
        options.push(InstallOptions::UpdateHpAndBuckets);
    }
    if update_options.contains(&UpdateOptions::NoAutoDownloadDepends) {
        options.push(InstallOptions::NoAutoDownloadDepends)
    }
    if update_options.contains(&UpdateOptions::NoUseDownloadCache) {
        options.push(InstallOptions::NoUseDownloadCache)
    }
    if update_options.contains(&UpdateOptions::SkipDownloadHashCheck) {
        options.push(InstallOptions::SkipDownloadHashCheck)
    }
    options
}

pub async fn update_specific_app(
    app_name: &str,
    options: &[UpdateOptions],
) -> Result<(), anyhow::Error> {
    log::debug!("update_specific_app {}", &app_name);
    let apps_dir = if options.contains(&Global) {
        get_apps_path_global()
    } else {
        get_apps_path()
    };
 
    Ok(())
}

async fn check_scoop_update() -> anyhow::Result<bool> {
    let remote_head = remote_latest_scoop_commit()?;
    let local_head = local_scoop_latest_commit().expect("failed to get local_scoop_latest_commit");
    if remote_head == local_head {
        log::debug!("Scoop is up to date");
        return Ok(false);
    }
    log::debug!("Scoop is not up to date");
    Ok(true)
}

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
        .with_prefix(format!("🐧{:<longest_bucket_name$}", "Scoop "))
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
    #[allow(unused_doc_comments)]
    /// replace rayon  iterator  running  with foreach  for  map  method
    /// outdated_buckets.par_iter() // 来自 rayon
    ///  .for_each(|(pb, bucket_path)| {
    ///     let callback = gen_stats_callback(pb);
    ///     let result = git_pull_update_repo(bucket_path, &callback);
    ///    if let Err(e) = result {
    ///        pb.finish_with_message(format!("❌ {}", e));
    ///    } else {
    ///       pb.finish_with_message(FINISH_MESSAGE);
    ///   }
    ///    });
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
    let final_bucket_path = bucket_path
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
    #[allow(unused_imports)]
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
    fn test_final_bucket_path() {
        let official_buckets = get_include_buckets_path().unwrap();
        println!(
            "Official buckets: {:? } {:?}",
            official_buckets.len(),
            official_buckets
        );
    }
}
