use crate::manifest::install_manifest::InstallManifest;
use anyhow::{bail, Result};

mod   installer;
pub use installer::* ;
use crate::manifest::manifest_deserialize::* ;

pub struct  ArchStruct { }
pub async fn install_app_from_local_manifest_file (manifest_path  : &String, arch : Option<String>) -> Result<()> {

   log::info!("install from local manifest file {}", manifest_path);
    let  content = std::fs::read_to_string(manifest_path)?;
   let mut serde_obj:InstallManifest = serde_json::from_str (&content)?;
  let  name  = serde_obj.set_name(manifest_path).get_name().unwrap_or(String::new());
  if name .is_empty() {
    bail!("manifest file name is empty")
  }
  let  version = serde_obj.version.unwrap_or(String::new());
  if version.is_empty() {
    bail!("manifest file version is empty")
  }
  let suggest = serde_obj.suggest.unwrap_or(ManifestObj::Null);
 let  notes = serde_obj.notes.unwrap_or(Default::default());
  let arch_obj =  serde_obj.architecture.unwrap_or(ManifestObj::Null );
  if   ! arch_obj .is_null(){ 
    handle_arch(arch_obj); 
  }

  if    !suggest.is_null() {
    show_suggest(&suggest);
  } 
  if  notes !=  ArrayOrString::default() { 
    show_notes(&notes); 
  }
  Ok(())
}

 pub async fn install_from_specific_bucket ( bucket_name : &str, app_name : &str    ) -> Result<()> {
     log::info!("install from specific bucket from {}", bucket_name);
   Ok(())
 }

 pub async fn install_app_specific_version ( app_name : &str, app_version : &str ) -> Result<()> {

   log::info!("install from app specific version {}", app_version);
   Ok(())
 }

 pub async fn  install_app ( app_name :&str  ) -> Result<()> {
   log::info!("install from app {}", app_name);
   Ok( ())
 }



