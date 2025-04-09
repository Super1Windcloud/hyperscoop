use crate::manifest::install_manifest::{InstallManifest  };
use anyhow::{bail, Result};

pub mod installer;
use crate::init_env::get_app_current_dir;
use crate::manifest::manifest_deserialize::*;
pub use installer::*;
pub mod check;
pub mod shim_and_shortcuts;
pub use check::*;
pub use shim_and_shortcuts::*;
pub mod cli_options_store;
pub use cli_options_store::*;
pub mod aria2;
pub use aria2::*;
pub  mod  sevenzip; 
pub use sevenzip::*;
pub struct ArchStruct {}



pub async fn install_app_from_local_manifest_file(
  manifest_path: String,
  options: Vec<InstallOptions>,
) -> Result<()> {
  let options  :Box< [InstallOptions] >= options.into_boxed_slice(); 
  let   install_arch =   handle_arch( &options )?; 
  log::info!("install arch: {}", install_arch); 
  let content = std::fs::read_to_string(&manifest_path)?;
  let mut serde_obj: InstallManifest = serde_json::from_str(&content)?; 
   let name =  serde_obj
    .set_name(&manifest_path).get_name().unwrap_or(String::new());  
   let  obj_copy = serde_obj.clone(); 
  if name.is_empty() {
    bail!("manifest file name is empty")
  }
  let version = &serde_obj.version.unwrap_or(String::new());
  if version.is_empty() {
    bail!("manifest file version is empty")
  }
  let result = check_before_install(&name, &version, &options )?;
  if result != 0 {
    return Ok(());
  };
  let depends = serde_obj.depends ;
  let suggest = serde_obj.suggest; 
  let notes = serde_obj.notes;
  let env_set = serde_obj.env_set ; 
  let env_add_path =serde_obj.env_add_path ; 
  let  url = serde_obj.url;
  let hash = serde_obj.hash; 
  if !depends.is_none()  {
    handle_depends(depends.unwrap().as_str() , &options ).await?;
  }
  if !env_set.is_none()  {
    handle_env_set( env_set.unwrap() ,    obj_copy   )?;
  };
  if env_add_path.is_some()  { 
    let env_add_path = env_add_path.unwrap();
    if  env_add_path  !=  StringArrayOrString::Null {
      let app_current_dir = get_app_current_dir(& name);
      handle_env_add_path(env_add_path, app_current_dir)?;
    }
  }
  if !suggest.is_none()   {
    show_suggest( &suggest .unwrap())?;
  }
  if  notes .is_some() {
    let  notes = notes.unwrap(); 
    if  notes !=  StringArrayOrString::Null {
         show_notes( notes)?;
    }
  }
  Ok(())
}


pub async fn install_from_specific_bucket(bucket_name: &str, app_name: &str, options : &[InstallOptions]) -> Result<()> {
  log::info!("install from specific bucket from {}", bucket_name);
  Ok(())
}

pub async fn install_app_specific_version(app_name: &str, app_version: &str, options : &Vec<InstallOptions>) -> Result<()> {
  log::info!("install from app specific version {}", app_version);
  Ok(())
}

pub async fn install_app(app_name: &str, options : &[InstallOptions]) -> Result<()> {
  log::info!("install from app {}", app_name);
  
  let   install_arch =   handle_arch( &options )?;
  log::info!("install arch: {}", install_arch);
  Ok(())
}
