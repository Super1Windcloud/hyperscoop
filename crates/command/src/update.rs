#[allow(unused_imports)] 
use anyhow::bail; 
pub fn  update_all_apps()  -> Result<(), anyhow::Error> { 
  
  Ok(()) 
}

pub fn update_specific_app_without_cache(app_name: String) -> Result<(), anyhow::Error> {
  log::trace!("update_specific_app_without_cache");
    Ok(())
}

pub fn  update_specific_app_without_hash_check(app_name: String) -> Result<(), anyhow::Error> {
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