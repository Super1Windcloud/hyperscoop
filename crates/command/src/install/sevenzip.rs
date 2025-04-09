use std::env;
use std::path::Path;
use crate::install::{ArchiveFormat, InstallOptions};

pub struct  SevenZipStruct<'a> {
     archive_format: ArchiveFormat,
     archive_cache_dir  : &'a str  , 
     archive_name : &'a str,
     app_name : &'a str,
     app_version : &'a str, 
     target_dir : String  , 
     install_options: &'a [InstallOptions],
}

impl  <'a> SevenZipStruct<'a >  {
  pub fn new(options:  &'a [InstallOptions]) -> Self {
    Self {
      archive_format : ArchiveFormat::SevenZip,
      archive_cache_dir  :  "", 
      archive_name: "",
      app_name: "",
      app_version: "",
      target_dir: String::new() ,
       install_options: options,
    }
  }
  pub fn  get_app_name(&self) -> &'a str {
    self.app_name
  }
  pub fn  get_app_version(&self) -> &'a str {
    self.app_version
  }
  pub fn  set_app_name(&mut self, name: &'a str) {
    
    self.app_name = name;
  }
  pub fn  set_app_version(&mut self, version: &'a str) {
    self.app_version = version;
  }
  pub fn  get_archive_name(&self) -> &'a str {
    self.archive_name
  }
  pub fn  set_archive_name(&mut self, name: &'a str) {
    self.archive_name = name;
  }
  
  pub fn  get_archive_format(&self) -> &ArchiveFormat {
    &self.archive_format
  } 
  pub fn  get_archive_dir(&self) -> &str  {
    &self.archive_cache_dir
  } 
  pub fn  set_archive_dir(&mut self, path: &'a str ) {
    self.archive_cache_dir = path;
  }
  pub fn  set_archive_format(&mut self, format: ArchiveFormat) {
    self.archive_format = format;
  }
  pub  fn get_temp_7z_path(&self) -> String {
    let temp_dir = env::temp_dir();
    let exe_path = temp_dir.join("7z.exe");
    exe_path.to_str().unwrap().to_string()
  }
}






mod test_7z {
  #[allow(unused_imports)]
    use super::*;
    #[test]  
     fn test_invoke_7z () { 
        // let  zip  =SevenZipStruct::new() ; 
        //  let  temp_dir = zip.get_temp_7z_path() ; 
     }
  
}