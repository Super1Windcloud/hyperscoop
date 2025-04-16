use crate::init_env::{
    get_app_current_dir, get_app_current_dir_global, get_app_version_dir,
    get_app_version_dir_global, get_cache_dir_path, get_cache_dir_path_global,
    get_persist_app_data_dir, get_persist_app_data_dir_global,
};
use crate::install::InstallOptions::{
    ArchOptions, ForceDownloadNoInstallOverrideCache, Global, NoUseDownloadCache,
};
use crate::install::{Aria2C, HashFormat, InstallOptions};
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::{ArchitectureObject, StringArrayOrString};
use crate::utils::system::get_system_default_arch;
use anyhow::bail;
use crossterm::style::Stylize;
use hex;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::vec;
use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DownloadManager<'a> {
    app_name: &'a str,
    app_version: String,
    pub aria2c: Aria2C<'a>,
    scoop_cache_dir: String,
    cache_file_name: Vec<String>,
    app_version_dir: String,
    app_current_dir: String,
    options: &'a [InstallOptions<'a>],
    hash_format: Box<[HashFormat]>,
    hash_value: Box<[String]>,
    input_file: String,
    download_urls: Box<[String]>,
    target_rename_alias: Box<[String]>,
    persist_data_dir: String,
    final_cache_file_path: Box<[String]>,
}

impl<'a> DownloadManager<'a> {
    pub fn get_final_cache_file_path(&self) -> &[String] {
        &self.final_cache_file_path
    }
    pub fn set_final_cache_file_path(&mut self) -> anyhow::Result<()> {
        let cache_file = self.get_cache_file_name();
        let mut files = vec![];
        let result = cache_file.iter().try_for_each(|name| {
            let file = format!("{}\\{}", self.get_scoop_cache_dir(), name);
            if !Path::new(file.as_str()).exists() {
                bail!("下载缓存文件{}不存在, 是否下载失败?", &file)
            } else {
                files.push(file);
                Ok(())
            }
        });
        if result.is_err() {
            bail!(result.unwrap_err())
        }
        self.final_cache_file_path = files.into_boxed_slice();
        Ok(())
    }
    pub fn get_persist_data_dir(&self) -> &str {
        &self.persist_data_dir
    }
    pub fn set_persist_data_dir(&mut self) {
        let persist_data_dir = if self.options.contains(&Global) {
            get_persist_app_data_dir_global(self.app_name)
        } else {
            get_persist_app_data_dir(self.app_name)
        };
        self.persist_data_dir = persist_data_dir;
    }
    pub fn get_hash_value(&self) -> &Box<[String]> {
        &self.hash_value
    }
    pub fn set_hash_value(&mut self, hash_value: Box<[String]>) {
        self.hash_value = hash_value;
    }

    pub fn get_hash_format(&self) -> &Box<[HashFormat]> {
        &self.hash_format
    }
    pub fn set_hash_format(
        &mut self,
        hash: Option<StringArrayOrString>,
        architecture: Option<ArchitectureObject>,
    ) -> anyhow::Result<()> {
        let hash_format = if hash.is_some() {
            hash.unwrap()
        } else if architecture.is_some() {
            let arch = architecture.unwrap();
            let options_arch = self.get_user_options_arch()?;
            if options_arch == "64bit" {
                let x64 = arch.x64bit.unwrap();
                let hash = x64.hash.unwrap();
                hash
            } else if options_arch == "32bit" {
                let x86 = arch.x86bit.unwrap();
                let hash = x86.hash.unwrap();
                hash
            } else if options_arch == "arm64" {
                let arm64 = arch.arm64.unwrap();
                let hash = arm64.hash.unwrap();
                hash
            } else {
                bail!("Unsupported architecture");
            }
        } else {
            bail!("hash str is empty")
        };
        let (format, values) = match hash_format {
            StringArrayOrString::String(s) => {
                if !s.contains(':') {
                    (vec![HashFormat::SHA256], vec![s])
                } else {
                    let format = s.split(":").next().unwrap();
                    let hash_value = s.split(":").last().unwrap();
                    let format = if format == "sha1" {
                        HashFormat::SHA1
                    } else if format == "sha512" {
                        HashFormat::SHA512
                    } else if format == "md5" {
                        HashFormat::MD5
                    } else {
                        HashFormat::SHA256
                    };
                    (vec![format], vec![hash_value.to_string()])
                }
            }
            StringArrayOrString::StringArray(arr) => {
                let (format, hash_value) = arr.into_iter().fold(
                    (vec![], vec![]),
                    |(mut formats, mut values), hash_str| {
                        if !hash_str.contains(':') {
                            formats.push(HashFormat::SHA256);
                            values.push(hash_str);
                        } else {
                            let format = hash_str.split(":").next().unwrap();
                            let hash_value = hash_str.split(":").last().unwrap();
                            let format = match format {
                                "sha1" => HashFormat::SHA1,
                                "sha512" => HashFormat::SHA512,
                                "md5" => HashFormat::MD5,
                                _ => HashFormat::SHA256,
                            };
                            formats.push(format);
                            values.push(hash_value.to_string());
                        }

                        (formats, values)
                    },
                );
                (format, hash_value)
            }
            StringArrayOrString::Null => {
                bail!("hash str is empty")
            }
        };
        self.hash_format = format.into_boxed_slice();
        self.set_hash_value(values.into_boxed_slice());

        Ok(())
    }
    pub fn get_target_rename_alias(&self) -> Vec<&str> {
        self.target_rename_alias.iter().map(|s| &**s).collect()
    }

    pub fn set_target_rename_alias(&mut self, new_alias: Vec<&str>) {
        let a = new_alias.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        self.target_rename_alias = a.into_boxed_slice();
    }
    pub fn get_download_urls(&self) -> Vec<&str> {
        self.download_urls.iter().map(|s| &**s).collect()
    }
    pub fn set_download_urls(&mut self, download_urls: &Vec<String>) {
        let mut alias = vec![];
        let download_urls = download_urls
            .iter()
            .map(|s| {
                if s.contains("#/") {
                    let suffix = s.split("#/").last().unwrap();
                    alias.push(suffix);
                    let url = s.split("#/").next().unwrap();
                    url.to_string()
                } else {
                    s.to_string()
                }
            })
            .collect::<Vec<_>>();
        self.set_target_rename_alias(alias);
        self.download_urls = download_urls.to_vec().into_boxed_slice();
    }

    pub fn get_input_file(&self) -> &str {
        &self.input_file.as_str()
    }

    pub fn set_input_file(&mut self) {
        let aria2c_input_file = format!(
            "{}\\{}_input_file.txt",
            self.get_scoop_cache_dir(),
            self.app_name
        );
        self.input_file = aria2c_input_file
    }
    pub fn create_input_file(&self) -> anyhow::Result<()> {
        let mut file = std::fs::File::create(self.get_input_file())?;
        let urls = self.get_download_urls();
        let result =
            urls.iter()
                .zip(self.get_cache_file_name())
                .try_for_each(|(url, cache_name)| {
                    let content = format!("{}\n\tout={}", url, cache_name);
                    writeln!(file, "{}", content)?;
                    Ok(())
                }) as anyhow::Result<_>;
        if result.is_err() {
            bail!("创建缓存输入文件{}失败", self.get_input_file())
        }
        Ok(())
    }
    pub fn create_aria2c_instance(&self) -> Aria2C {
        let aria2c = Aria2C::new();
        aria2c
    }
    pub fn set_options(&mut self, options: &'a [InstallOptions]) {
        self.options = options;
    }
    pub fn get_options(&self) -> &'a [InstallOptions] {
        self.options
    }
    // ensure_install_dir_not_in_path   检查并清理系统或用户的 PATH 环境变量，确保某个目录（或其子目录）不会出现在 PATH 中，
    pub fn ensure_install_dir_not_in_env_path(&self) -> anyhow::Result<()> {
        let env_path: String = if self.options.contains(&Global) {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let environment_key = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
            let env_key = hklm.open_subkey(environment_key)?;
            env_key.get_value("PATH")?
        } else {
            let hklm = RegKey::predef(HKEY_CURRENT_USER);
            let env = hklm.open_subkey("Environment")?;
            let user_path = env.get_value("PATH")?;
            user_path
        };
        let dir = self.app_current_dir.as_str();
        if env_path.contains(dir) {
            bail!("{} 已经存在于 PATH 环境变量中, 请检查", dir)
        }
        Ok(())
    }

    pub fn ensure_version_dir_exist(&self) -> anyhow::Result<PathBuf> {
        let path = Path::new(self.app_version_dir.as_str());

        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        let absolute_path = if path.is_relative() {
            std::env::current_dir()?.join(path)
        } else {
            path.to_path_buf()
        };
        // 规范化路径（移除多余的 ./ 或 ../）
        let canonical_path = std::fs::canonicalize(absolute_path)?;
        Ok(canonical_path)
    }
    pub fn set_app_version_dir(&mut self) {
        let app_version_dir = if self.options.contains(&Global) {
            get_app_version_dir_global(self.app_name, &self.app_version)
        } else {
            get_app_version_dir(self.app_name, &self.app_version)
        };

        self.app_version_dir = app_version_dir
    }
    pub fn set_app_current_dir(&mut self) {
        let app_current_dir = if self.options.contains(&Global) {
            get_app_current_dir_global(self.app_name)
        } else {
            get_app_current_dir(self.app_name)
        };
        self.app_current_dir = app_current_dir;
    }
    pub fn get_app_version_dir(&self) -> &str {
        self.app_version_dir.as_str()
    }
    pub fn get_app_current_dir(&self) -> &str {
        self.app_current_dir.as_str()
    }
    pub fn get_user_options_arch(&self) -> anyhow::Result<String> {
        if let Some(ArchOptions(arch)) = self
            .options
            .iter()
            .find(|opt| matches!(opt, ArchOptions(_)))
        {
            Ok(arch.to_string())
        } else {
            Ok(get_system_default_arch()?)
        }
    }

    fn init(&mut self, manifest_path: &'a str) -> anyhow::Result<()> {
        let download_cache_dir = if self.options.contains(&Global) {
            get_cache_dir_path_global()
        } else {
            get_cache_dir_path()
        };
        self.ensure_version_dir_exist()?;
        let app_name = Path::new(manifest_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        let content = std::fs::read_to_string(manifest_path)?;
        let serde_obj = serde_json::from_str::<InstallManifest>(&content)?;
        let version = serde_obj.version.expect("version 不能为空");
        self.set_app_version(&version);
        self.set_scoop_cache_dir(&download_cache_dir);
        let architecture = serde_obj.architecture;

        let hash = serde_obj.hash;

        self.set_hash_format(hash, architecture.clone())?;

        let url = serde_obj.url;
        if url.is_some() {
            self.set_cache_file_name(app_name, &version, &url.clone().unwrap())?;
        }
        if architecture.is_some() && url.is_none() {
            let architecture = architecture.unwrap();
            let final_arch = self.get_user_options_arch()?;
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
            } else if final_arch == "arm64" {
                let arm64 = architecture.arm64;
                if arm64.is_some() {
                    let url = arm64.unwrap().url.unwrap();
                    self.set_cache_file_name(app_name, &version, &url)?;
                }
            } else {
                bail!("架构选项错误");
            }
        } 
        self.set_final_cache_file_path()?; 
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
            scoop_cache_dir: "".into(),
            cache_file_name: vec![],
            app_version_dir: "".into(),
            app_current_dir: "".into(),
            options,
            hash_format: Box::from(vec![HashFormat::SHA256]),
            hash_value: vec![].into(),
            input_file: "".into(),
            download_urls: vec![].into_boxed_slice(),
            target_rename_alias: vec![].into_boxed_slice(),
             persist_data_dir: "".to_string(),
           final_cache_file_path: Box::new([]),
        };
        match download.init(manifest_path) {
            Ok(_) => download,
            Err(e) => {
                println!("{}", e.to_string().dark_red().bold());
                download
            }
        }
    }
    pub fn get_scoop_cache_dir(&self) -> &str {
        &self.scoop_cache_dir
    }
    pub fn set_scoop_cache_dir(&mut self, path: &str) {
        self.scoop_cache_dir = path.into()
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
    pub fn set_app_version(&mut self, app_version: &str) {
        self.app_version = app_version.into();
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
                .map(|url| url.trim().to_string())
                .collect::<Vec<String>>(),
            StringArrayOrString::String(url) => {
                vec![url.trim().to_string()]
            }
            StringArrayOrString::Null => {
                bail!("url 不能为空");
            }
        }
        .into_iter()
        .collect::<Vec<String>>();
        self.set_download_urls(&urls);

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
            .map(|suffix| {
                let file_name = format!("{}#{}", file_names_prefix, suffix);
                file_name
            })
            .collect::<Vec<String>>();
        self.cache_file_name = final_file_names;
        Ok(())
    }
    pub fn start_download(&self) -> anyhow::Result<()> {
        self.ensure_install_dir_not_in_env_path()?;
        let scoop_cache_dir = self.get_scoop_cache_dir();
        let input_file = self.get_input_file();
        let mut aria2c = self.create_aria2c_instance();
        aria2c.set_input_file(input_file);
        aria2c.set_scoop_cache_dir(scoop_cache_dir); // 设置aria2c的缓存目录

        if self.options.contains(&ForceDownloadNoInstallOverrideCache)
            || self.options.contains(&NoUseDownloadCache)
        {
            let cache_file_path = self
                .get_cache_file_name()
                .iter()
                .map(|name| format!("{}\\{}", self.get_scoop_cache_dir(), name))
                .collect::<Vec<String>>();
            cache_file_path.iter().try_for_each(|path| {
                println!(
                    "{}",
                    format!("Override Cache File '{path}'").dark_grey().bold()
                );
                std::fs::remove_file(path)
            })?;
        }
        let output = aria2c.invoke_aria2c_download();
        match output {
            Ok(output) => {
                println!("{}", output);
                std::fs::remove_file(input_file)?;
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", e.to_string().dark_red().bold());
                std::fs::remove_file(input_file)?;
                Ok(())
            }
        }
    }
    pub fn check_cache_file_hash(&self) -> anyhow::Result<()> { 
      
        let  cache_files = self.get_final_cache_file_path(); 
        let  hash_formats =self.get_hash_format(); 
        let  hash_values = self.get_hash_value(); 
        // let result  =  cache_files.iter().zip(hash_formats).zip(hash_values)
        //   .try_for_each(|((file,format),hash_value)| {
        //     let file = file.to_string();
        //    
        //   Ok(())
        // }); 
        Ok(())
    }
}

mod test_download_manager {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    fn test_cache_file_name() {
        let option = vec![ForceDownloadNoInstallOverrideCache];
        // let d = DownloadManager::new(&option, r"A:\Scoop\buckets\extras\bucket\sfsu.json");
        // let d = DownloadManager::new(&option, r"A:\Scoop\buckets\extras\bucket\7ztm.json");
        // let d = DownloadManager::new(option.as_slice(), r"A:\Scoop\buckets\main\bucket\yazi.json");
        let d = DownloadManager::new(option.as_slice(), r"A:\Scoop\buckets\main\bucket\bun.json");
        d.get_final_cache_file_path().iter().for_each(|name| {
            println!("{}", name);
        })
    }
  
   #[test] 
    fn test_check_hash(){ 
      let options  = vec![];
     let d = DownloadManager::new(options.as_slice(), r"A:\Scoop\buckets\main\bucket\bun.json");
      d.check_cache_file_hash().unwrap();

   }
}
