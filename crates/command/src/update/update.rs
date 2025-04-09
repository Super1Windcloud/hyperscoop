use crate::utils::utility::{get_official_bucket_path, get_official_buckets_name};
use anyhow::{bail, Context};
use crossterm::style::Stylize;
use git2::{FetchOptions, Repository};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub fn check_bucket_update_status<'a>() -> anyhow::Result<bool> {
    let official_buckets = get_official_buckets_name();
    let official_buckets_path = official_buckets
        .iter()
        .map(|b| get_official_bucket_path(b.clone()))
        .collect::<Vec<_>>();
    let status_flag = Arc::new(Mutex::new(false));
    let result: anyhow::Result<()> = official_buckets_path.par_iter().try_for_each(|path| {
        let repo = Repository::open(&path)
            .with_context(|| format!("Failed to open repository at {}", path))?;

        let mut remote = repo
            .find_remote("origin")
            .with_context(|| format!("Failed to find remote 'origin' in {}", path))?;

        let mut fetch_options = FetchOptions::new();
        fetch_options.download_tags(git2::AutotagOption::All);

        remote
            .fetch::<&str>(&[], Some(&mut fetch_options), None)
            .with_context(|| format!("Failed to fetch from remote for {}", path))?;

        let local_head = repo
            .head()?
            .target()
            .with_context(|| format!("Failed to get local HEAD for {}", path))?;

        let remote_head = repo
            .refname_to_id("refs/remotes/origin/HEAD")
            .with_context(|| format!("Failed to get remote HEAD for {}", path))?;

        if local_head != remote_head {
            *status_flag.lock().unwrap() = true;
        }
        Ok(())
    });
    if result.is_err() {
        bail!(result.unwrap_err())
    }
    let flag = *status_flag.lock().unwrap();
    if !flag {
        println!(
            "{}",
            "All Buckets are up to date".to_string().dark_green().bold()
        );
    } else {
        println!(
            "{}",
            "Some Buckets are outData and has updates available"
                .to_string()
                .dark_green()
                .bold()
        );
    }
    Ok(flag)
}

mod test {
    #[allow(unused)]
    use super::*;

    #[test]
    fn check_update() {
        let _ = check_bucket_update_status();
    }
}
