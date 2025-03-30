use anyhow::Context;
use crossterm::style::Stylize;
use git2::{FetchOptions, Repository};
use crate::utils::utility::{get_official_bucket_path, get_official_buckets_name};

pub fn check_bucket_update_status ()  -> anyhow::Result<()> {  
    let official_buckets = get_official_buckets_name(); 
  let   official_buckets_path  =  official_buckets.iter().map( |b|
      get_official_bucket_path(b.clone()) ).collect::<Vec<_>>() ;
  let mut  status_flag  = false; 
  for path in official_buckets_path {
    let repo = Repository::open(&path)
      .with_context(|| format!("Failed to open repository at {}", path))?;

    let mut remote = repo.find_remote("origin")
      .with_context(|| format!("Failed to find remote 'origin' in {}", path))?;

    let mut fetch_options = FetchOptions::new();
    fetch_options.download_tags(git2::AutotagOption::All);

    remote.fetch::<&str>(&[] , Some(&mut fetch_options), None)
      .with_context(|| format!("Failed to fetch from remote for {}", path))?;

    let local_head = repo.head()?.target()
      .with_context(|| format!("Failed to get local HEAD for {}", path))?;

    let remote_head = repo.refname_to_id("refs/remotes/origin/HEAD")
      .with_context(|| format!("Failed to get remote HEAD for {}", path))?;

    if local_head != remote_head { 
      status_flag = true; 
    }  
  }
  if !status_flag {
    println!("{}" , "Bucket  is up to date".to_string().dark_yellow().bold());
  }else  {
    println!("{}" , "Bucket is outData and has updates available".to_string().dark_yellow().bold());
  }
  Ok(()) 
  
} 

mod test{
  use crate::update::update::check_bucket_update_status;

  #[test]
  fn  check_update(){ 
      check_bucket_update_status().unwrap()
  }
  
}
