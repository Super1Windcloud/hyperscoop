use crate::init_env::{get_cache_dir_path, get_cache_dir_path_global};
use crate::install::InstallOptions::Global;
use crate::install::{Aria2C, InstallOptions};
use crate::manifest::install_manifest::InstallManifest;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DownloadManager<'a> {
    app_name: &'a str,
    app_version: String ,
    aria2c: Aria2C<'a>,
    scoop_cache_dir: String,
    cache_file_name: String,
}

impl<'a> DownloadManager<'a> {
    fn init(&mut self, options: &[InstallOptions], manifest_path: &'a str) -> anyhow::Result<()> {
        let download_cache_dir = if options.contains(&Global) {
            get_cache_dir_path_global()
         } else {
            get_cache_dir_path()
         };  
         let  app_name = manifest_path.split('/').last().unwrap();  
         let  app_name  = app_name.split('.').next().unwrap();   
         let  content  = std::fs::read_to_string(manifest_path)?; 
         let   serde_obj = serde_json::from_str::<InstallManifest>(&content)?  ;  
         let version = serde_obj.version.expect("version 不能为空"); 
         self.set_download_app_name(app_name);
        // self.set_cache_file_name(&version, app_name,url );
      
         self.set_app_version(version); 
         self.set_scoop_cache_dir(&download_cache_dir);  
        Ok(())
    }
  
    pub fn new(options: &'a [InstallOptions], manifest_path :&'a str ) -> DownloadManager<'a> {  
       let mut download =  Self {
            app_name: "",
            app_version: "".into(),
            aria2c: Aria2C::new(options),
            scoop_cache_dir: "".to_string(),
            cache_file_name: "".to_string(),
        }; 
      download.init(options, manifest_path ).unwrap(); 
      download
    }
    pub fn get_scoop_cache_dir(&self) -> &str {
        &self.scoop_cache_dir
    }
    pub fn set_scoop_cache_dir(&mut self, path: &str) {
        self.scoop_cache_dir = path.to_string();
    }
    pub fn set_download_app_name(&mut self, app_name: &'a str) {
        self.app_name = app_name;
    }
    pub fn get_download_app_name(&self) -> &'a str {
        self.app_name
    }
    pub fn get_app_version(&self) -> &str   {
        self.app_version.as_str() 
    }
    pub fn set_app_version(&mut self, app_version: String ) {
        self.app_version = app_version;
    }
  pub  fn get_cache_file_name(&self) -> &str {
        &self.cache_file_name
  }
   pub fn  set_cache_file_name(&mut self, file_name: &str) {
      
   }
}



mod   test_download_manager {
  use crate::install::DownloadManager;

  #[test ] 
    fn   test_cache_file_name() { 
        let  option = vec![crate::install::InstallOptions::Global];
       let  d= DownloadManager::new(&option, r"A:\Scoop\buckets\main\bucket\scons.json");
    }
}