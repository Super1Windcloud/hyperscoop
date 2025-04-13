use std::io::Write;
use std::path::Path;
use crate::init_env::{get_app_current_dir, get_app_current_dir_global, get_app_version_dir, get_app_version_dir_global, get_cache_dir_path, get_cache_dir_path_global};
use crate::install::InstallOptions::{ArchOptions, Global};
use crate::install::{Aria2C, HashFormat, InstallOptions};
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::StringArrayOrString;
use crate::utils::system::get_system_default_arch;
use anyhow::bail;
use hex;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DownloadManager<'a> {
    app_name: &'a str,
    app_version: String,
    aria2c: Aria2C<'a>,
    scoop_cache_dir: String,
    cache_file_name: Vec<String>,
    app_version_dir : String ,
    app_current_dir : String ,
    options: &'a [InstallOptions],
    hash: HashFormat,
    input_file: String  ,
    download_urls: Vec<String>,
}

impl<'a> DownloadManager<'a> {
    pub fn  get_download_urls(&self ) -> Vec<&str> {
      self.download_urls.iter().map(|s| &**s).collect()
    }
   pub fn set_download_urls(&mut self, download_urls: &[String] )  {
       self.download_urls = download_urls.to_vec();
   }
  pub fn  get_aria2c ( &self) -> &Aria2C {
    &self.aria2c
  }
  pub  fn  get_input_file ( &self ) -> &str {
    &self.input_file.as_str()
  }

   pub  fn  set_input_file ( &mut self ) {
     let  aria2c_input_file =format!("{}\\{}_input_file.txt",self.get_scoop_cache_dir(), self.app_name );
     self.input_file = aria2c_input_file
   }
   pub fn create_input_file(&self ) ->anyhow::Result<()>{
     let mut file = std::fs::File::create(self.get_input_file())?;
      let  urls = self.get_download_urls();
      let result = urls.iter().zip(self.get_cache_file_name()).try_for_each(|(url,cache_name)| {
          let content = format!("{}\n\tout={}\n", url  ,cache_name );
          writeln!(file, "{}", content)?;
          Ok(())
      }) as anyhow::Result<_>; 
      if   result.is_err(){
          bail!("创建缓存输入文件{}失败", self.get_input_file() )
      }
       Ok(())
   }
  pub fn  create_aria2c_instance(&mut self) {
        let aria2c = Aria2C::new();
        self.aria2c = aria2c;
    }
   pub  fn  set_options (&mut self, options: &'a [InstallOptions]) {
        self.options = options;
    }
   pub fn get_options(&self) -> &'a [InstallOptions] {
        self.options
    }
    pub  fn set_app_version_dir(&mut self ) {
        let app_version_dir =if  self.options.contains(&Global) {
           get_app_version_dir_global(self.app_name , &self.app_version)
        }else { get_app_version_dir(self.app_name , &self.app_version) } ;

        self.app_version_dir = app_version_dir
    }
    pub  fn set_app_current_dir(&mut self ) {
       let   app_current_dir = if self .options.contains(&Global) {
         get_app_current_dir_global(self.app_name)
       } else { get_app_current_dir(self.app_name ) } ;
        self.app_current_dir = app_current_dir;
    }
    pub fn get_app_version_dir(&self) -> & str {
        self.app_version_dir.as_str()
    }
    pub fn get_app_current_dir(&self) -> &str {
        self.app_current_dir.as_str()
    }
    fn init(&mut self,  manifest_path: &'a str) -> anyhow::Result<()> {
        let download_cache_dir = if self.options.contains(&Global) {
            get_cache_dir_path_global()
        } else {
            get_cache_dir_path()
        };

        let app_name =Path::new(manifest_path).file_stem().unwrap().to_str().unwrap();
        let content = std::fs::read_to_string(manifest_path)?;
        let serde_obj = serde_json::from_str::<InstallManifest>(&content)?;
        let version = serde_obj.version.expect("version 不能为空");
        self.set_app_version(&version);
        self.set_scoop_cache_dir(&download_cache_dir);

        let url = serde_obj.url;
        if url.is_some() {
            self.set_cache_file_name(app_name, &version, &url.clone().unwrap())?;
        }
        let arch = serde_obj.architecture;
        if arch.is_some() && url.is_none() {
            let architecture = arch.unwrap();
            let final_arch = if let Some(ArchOptions(arch)) =
                self.options.iter().find(|opt| matches!(opt, ArchOptions(_)))
            {
                arch.to_string()
            } else {
                get_system_default_arch()?
            };
            if final_arch == "64bit" {
                let x64 = architecture.x64bit;
                if x64.is_some() {
                    let url = x64.unwrap().url.unwrap();
                    self.set_cache_file_name(app_name, &version, &url)?;
                }
            } else if final_arch == "32bit" {
                let x86 = architecture.x86bit;
                if x86.is_some() {
                    let url = x86.unwrap().url.unwrap();
                    self.set_cache_file_name(app_name, &version, &url)?;
                }
            } else if final_arch == "64bit" {
                let arm64 = architecture.arm64;
                if arm64.is_some() {
                    let url = arm64.unwrap().url.unwrap();
                    self.set_cache_file_name(app_name, &version, &url)?;
                }
            } else {
                bail!("架构选项错误");
            }
        }
        self.set_download_app_name(app_name);
        self.set_app_current_dir();
        self.set_app_version_dir();
        self.set_input_file();
        self.create_input_file()?;
        Ok(())
    }

    pub fn new(options: &'a [InstallOptions], manifest_path: &'a str) -> DownloadManager<'a> {
        let mut download = Self {
             app_name: "",
             app_version: "".into(),
             aria2c: Aria2C::new(),
             scoop_cache_dir: "".to_string(),
             cache_file_name: vec![],
             app_version_dir: "".into() ,
             app_current_dir: "".into(),
             options,
             hash: HashFormat::SHA256 ,
             input_file : "".into(),
             download_urls: vec![],
        };
        download.init(manifest_path).unwrap();
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
    pub fn get_app_version(&self) -> &str {
        self.app_version.as_str()
    }
    pub fn set_app_version(&mut self, app_version: &str ) {
        self.app_version = app_version.into() ;
    }
    pub fn get_cache_file_name(&self) -> &Vec<String> {
        &self.cache_file_name
    }
    pub fn set_cache_file_name(
        &mut self,
        app_name: &str,
        app_version: &str,
        url: &StringArrayOrString,
    ) -> Result<(), anyhow::Error> {
        let urls = match url {
            StringArrayOrString::StringArray(urls) => urls
                .iter()
                .map(|url| {
                     url.trim().to_string()
                })
                .collect::<Vec<String>>(),
            StringArrayOrString::String(url) => {
                vec![url .trim().to_string()]
            }
            StringArrayOrString::Null => {
                bail!("url 不能为空");
            }
        }
        .into_iter()
        .collect::<Vec<String>>();
        self.set_download_urls(urls.as_slice().as_ref());
        let file_names_prefix = format!("{}#{}", app_name, app_version);
        let hashs = urls
            .iter()
            .map(|url| {
                let mut hasher = Sha256::new();
                hasher.update(url.as_bytes());
                let result = hasher.finalize();
                let short_hash = hex::encode(&result)[..7].to_string();
                short_hash
            })
            .collect::<Vec<String>>();
        let extensions = urls
            .iter()
            .map(|url| {
                let last_item = url.split('/').last().unwrap();
                let file_extension = last_item.split('.').last().unwrap();
                file_extension
            })
            .collect::<Vec<&str>>();

        let final_suffix = hashs
            .iter()
            .zip(extensions)
            .map(|(hash, extension)| {
                let file_name = format!(
                    "{}{}",
                    hash,
                    if extension.is_empty() {
                        "".into()
                    } else {
                        format!(".{}", extension)
                    }
                );
                file_name
            })
            .collect::<Vec<String>>();
        let final_file_names = final_suffix
            .iter()
            .map(|suffix| format!("{}#{}", file_names_prefix, suffix))
            .collect::<Vec<String>>();
        self.cache_file_name = final_file_names;
        Ok(())
    }
    pub fn start_download(&self) -> anyhow::Result<()> {
        let cache_file_name = self.get_cache_file_name();
        let scoop_cache_dir = self.get_scoop_cache_dir();
        Ok(())
    }
}

mod test_download_manager {

    #[test]
    fn test_cache_file_name() {
        use crate::install::DownloadManager;
        let option = vec![];
        let d = DownloadManager::new(&option, r"A://Scoop//buckets//main//bucket//scons.json");
        let d = DownloadManager::new(&option, r"A:\Scoop\buckets\extras\bucket\sfsu.json");
        let d = DownloadManager::new(&option, r"A:\Scoop\buckets\extras\bucket\7ztm.json");
       d.start_download().unwrap();
    }
}
