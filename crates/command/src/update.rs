use crate::buckets::{get_buckets_name, get_buckets_path};
use crate::utils::git::{
    git_pull_update_repo, git_pull_update_repo_with_scoop, local_scoop_latest_commit,
    remote_latest_scoop_commit,
};
use rayon::prelude::*;
use std::fs;
use std::path::Path;
pub(crate) mod update;
use crate::init_env::{
    get_app_current_dir, get_app_current_dir_global, get_app_dir, get_app_dir_global,
};
use crate::install::InstallOptions::UpdateTransaction;
use crate::install::UpdateOptions::{ForceUpdateOverride, Global, RemoveOldVersionApp};
use crate::install::{install_app, InstallOptions, UpdateOptions};
use crate::list::get_all_installed_apps_name;
use crate::utils::progrees_bar::{gen_stats_callback, ProgressOptions};
use crate::utils::progrees_bar::{
    indicatif::{MultiProgress, ProgressBar, ProgressFinish},
    style, Message,
};
use crate::utils::request::get_git_repo_remote_url;
use crate::utils::system::kill_processes_using_app;
use crate::utils::utility::get_official_bucket_path;
#[allow(unused_imports)]
use anyhow::bail;
use anyhow::Context;
use crossterm::style::Stylize;
use indicatif::ProgressDrawTarget;
pub use update::*;

const FINISH_MESSAGE: &str = "✅";

pub fn update_all_apps(options: &[UpdateOptions]) -> Result<(), anyhow::Error> {
    let all_apps_name = get_all_installed_apps_name();
    for app in all_apps_name {
        let _ = match check_app_version_latest(&app, &options) {
            Ok(version) => {
                if version.is_some() {
                    continue;
                }
            }
            Err(err) => {
                eprintln!("{}", err.to_string().dark_red().bold());
                let app_current = if options.contains(&Global) {
                    get_app_current_dir_global(&app)
                } else {
                    get_app_current_dir(&app)
                };
                fs::remove_dir_all(&app_current).context(format!(
                    "Failed to remove broken link current {app_current} at 50"
                ))?;
            }
        };
        update_specific_app(&app, options)?;
    }
    Ok(())
}

pub fn remove_old_version(app_name: &str, options: &[UpdateOptions]) -> anyhow::Result<()> {
    let app_current_dir = if options.contains(&Global) {
        get_app_current_dir_global(app_name)
    } else {
        get_app_current_dir(app_name)
    };
    let target_version_path = fs::read_link(app_current_dir)
        .context(format!("failed to read link of {} at line 67", app_name))?;
    log::debug!("target_version_path: {:?}", target_version_path);
    let result = fs::remove_dir_all(&target_version_path).context(format!(
        "failed to remove target version of {} at line 70",
        app_name
    ));
    if result.is_err() {
        eprintln!(
            "Remove Failed : {}",
            result.err().unwrap().to_string().dark_red().bold()
        );
        kill_processes_using_app(app_name);
        fs::remove_dir_all(&target_version_path).context(format!(
            "failed to remove target version of {app_name} at line 80"
        ))?
    }
    Ok(())
}
pub fn transform_update_options_to_install(
    update_options: &[UpdateOptions],
) -> Vec<InstallOptions> {
    let mut options = vec![];
    options.push(UpdateTransaction);
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
    if update_options.contains(&ForceUpdateOverride) {
        options.push(InstallOptions::ForceInstallOverride)
    }
    if update_options.contains(&UpdateOptions::InteractiveInstall) {
        options.push(InstallOptions::InteractiveInstall)
    }
    options
}

pub fn update_specific_app(app_name: &str, options: &[UpdateOptions]) -> Result<(), anyhow::Error> {
    log::debug!("update_specific_app {}", &app_name);
    let origin_options = options.to_vec();
    let options = transform_update_options_to_install(options);

    if origin_options.contains(&ForceUpdateOverride) {
        let special_app_dir = if origin_options.contains(&Global) {
            get_app_dir_global(&app_name)
        } else {
            get_app_dir(&app_name)
        };
        if Path::new(&special_app_dir).exists() {
            fs::remove_dir_all(special_app_dir)
                .context("Failed to remove old version of app at 102")?;
        }
    };
    let _ = match check_app_version_latest(&app_name, &origin_options) {
        Ok(version) => {
            if version.is_some() {
                let version = version.unwrap();
                println!(
                    "{}",
                    format!("🐉🎉🍾🐦‍🔥 {app_name}({version}) 已是最新版本,无需更新")
                        .dark_cyan()
                        .bold()
                        .to_string()
                );
                return Ok(());
            }
        }
        Err(err) => {
            eprintln!("{}", err.to_string().dark_red().bold());
            let app_current = if origin_options.contains(&Global) {
                get_app_current_dir_global(&app_name)
            } else {
                get_app_current_dir(&app_name)
            };
            fs::remove_dir_all(&app_current).context(format!(
                "Failed to remove broken link current {app_current} at 126"
            ))?;
        }
    };

    if origin_options.contains(&RemoveOldVersionApp)
        && app_name != "hp"
        && !origin_options.contains(&ForceUpdateOverride)
    {
        remove_old_version(&app_name, &origin_options)?;
    }
    install_app(&app_name, options.as_ref())?;
    Ok(())
}

fn check_scoop_update() -> anyhow::Result<bool> {
    let remote_head = remote_latest_scoop_commit()?;
    let local_head = local_scoop_latest_commit().expect("failed to get local_scoop_latest_commit");
    if remote_head == local_head {
        log::debug!("Scoop is up to date");
        return Ok(false);
    }
    log::debug!("Scoop is not up to date");
    Ok(true)
}

pub fn update_scoop_bar() -> anyhow::Result<()> {
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

    pb.set_draw_target(ProgressDrawTarget::stdout());

    let scoop_status = check_scoop_update()?;

    if !scoop_status {
        pb.finish_with_message("✅ No updates available");
        return Ok(());
    }
    let callback = gen_stats_callback(&pb);
    git_pull_update_repo_with_scoop(callback)?;
    pb.finish_with_message(FINISH_MESSAGE);
    Ok(())
}

pub fn update_all_buckets_bar_serial() -> anyhow::Result<()> {
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
            pb.set_draw_target(ProgressDrawTarget::stdout());
            (pb, bucket_path)
        })
        .collect::<Vec<_>>();

    let _ = outdated_buckets.iter().try_for_each(|(pb, bucket_path)| {
        let callback = gen_stats_callback(pb);
        let result = git_pull_update_repo(bucket_path, &callback);
        if let Err(e) = result {
            pb.finish_with_message(format!("❌ {}", e.to_string()));
        }
        pb.finish_with_message(FINISH_MESSAGE);
        Ok(())
    }) as anyhow::Result<()>;
    Ok(())
}

pub fn update_all_buckets_bar_parallel() -> anyhow::Result<()> {
    let progress_style = style(Some(ProgressOptions::Hide), Some(Message::suffix()));
    let official_buckets = get_include_buckets_name()?;
    let longest_bucket_name = official_buckets
        .iter()
        .map(|item| item.len())
        .max()
        .unwrap_or(0);

    let mp = MultiProgress::new();
    mp.set_draw_target(ProgressDrawTarget::stdout());
    mp.set_move_cursor(true);
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
            pb.set_draw_target(ProgressDrawTarget::stdout());
            (pb, bucket_path)
        })
        .collect::<Vec<_>>();

    let _ = outdated_buckets
        .par_iter()
        .try_for_each(|(pb, bucket_path)| {
            let callback = gen_stats_callback(pb);
            let result = git_pull_update_repo(bucket_path, &callback);
            if let Err(e) = result {
                pb.finish_with_message(format!("❌ {}", e.to_string()));
                return Err(e);
            }
            pb.finish_with_message(FINISH_MESSAGE);
            Ok(())
        });

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
