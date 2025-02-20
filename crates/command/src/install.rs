use crate::manifest::install_manifest::InstallManifest;


pub struct  ArchStruct { }
pub async fn install_app_from_local_manifest_file (manifest_path  : &String, arch : Option<String>) -> anyhow::Result<()> { 
    let  content = std::fs::read_to_string(manifest_path)?; 
   let   serde_obj :InstallManifest = serde_json::from_str (&content)?;  
  let arch_obj =  serde_obj.architecture.unwrap(); 
   let arch = if arch.is_some() { arch.unwrap() } else {  
        String::from(arch.as_ref().unwrap())
   };
  Ok(())
}
 
 pub async fn install_from_specific_bucket ( bucket_name : &str, app_name : &str    ) -> anyhow::Result<()> { 
   
   Ok(())
 }
 
 pub async fn install_app_specific_version ( app_name : &str, app_version : &str ) -> anyhow::Result<()> {

   Ok(())
 }
 
 pub async fn  install_app ( app_name :&str  ) -> anyhow::Result<()> { 
   
   Ok( ())
 }



