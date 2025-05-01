use crate::init_env::{
    get_app_current_dir, get_app_current_dir_global, get_app_version_dir,
    get_app_version_dir_global, get_apps_path, get_apps_path_global, get_cache_dir_path,
    get_cache_dir_path_global, get_persist_app_data_dir, get_persist_app_data_dir_global,
};
use crate::install::InstallOptions::{
    ArchOptions, ForceDownloadNoInstallOverrideCache, Global, NoUseDownloadCache,
};
use crate::install::{ArchiveFormat, Aria2C, HashFormat, InstallOptions, SevenZipStruct};
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::{ArchitectureObject, StringArrayOrString};
use crate::utils::system::{compute_hash_by_powershell, get_system_default_arch};
use crate::utils::utility::get_parse_url_query;
use anyhow::bail;
use crossterm::style::Stylize;
use digest::Digest;
use hex;
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use std::borrow::Cow;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::vec;
use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DownloadManager<'a> {
    app_name: &'a str,
    manifest_path: &'a str,
    app_version: String,
    pub aria2c: Aria2C<'a>,
    scoop_cache_dir: String,
    cache_file_name: Vec<String>,
    app_version_dir: String,
    app_current_dir: String,
    bucket_source: Option<&'a str>,
    options: &'a [InstallOptions<'a>],
    hash_format: Box<[HashFormat]>,
    hash_value: Box<[String]>,
    input_file: String,
    download_urls: Box<[String]>,
    target_rename_alias: Box<[String]>,
    persist_data_dir: String,
    final_cache_file_path: Box<[String]>,
    install_arch: Cow<'a, str>,
    origin_cache_file_names: Box<[String]>,
    archive_files_format: Box<[ArchiveFormat]>,
    exe_setup: bool,
}

impl<'a> DownloadManager<'a> {
    pub fn save_install_info(&self) -> anyhow::Result<()> {
        let current_dir = self.get_app_current_dir();
        let arch = self.get_install_arch().as_ref();
        let install_json_path = format!("{}\\install.json", current_dir);
        let manifest = if self.bucket_source.is_some() {
            self.bucket_source.clone().unwrap()
        } else {
            self.manifest_path
        };
        let install_json = serde_json::json!({
            "architecture": arch ,
            "bucket": manifest
        });
        let write_manifest_path = format!("{}\\manifest.json", current_dir);
        if Path::new(&install_json_path).exists() {
            std::fs::remove_file(&install_json_path)?;
        }

        let file =
            std::fs::File::create(&install_json_path).expect("Failed to create install.json");

        serde_json::to_writer_pretty(&file, &install_json).expect("Failed to write install.json");
        if Path::new(&write_manifest_path).exists() {
            std::fs::remove_file(&write_manifest_path)?;
        }
        std::fs::copy(self.manifest_path, write_manifest_path).expect("copy manifest failed");
        Ok(())
    }
    pub fn get_archive_files_format(&self) -> &[ArchiveFormat] {
        &self.archive_files_format
    }

    pub fn set_archive_files_format(&mut self, format: Vec<ArchiveFormat>) {
        self.archive_files_format = format.into_boxed_slice()
    }

    pub fn get_install_arch(&self) -> &Cow<'a, str> {
        &self.install_arch
    }

    pub fn get_final_cache_file_path(&self) -> &[String] {
        &self.final_cache_file_path
    }

    pub fn get_origin_cache_file_names(&self) -> &[String] {
        &self.origin_cache_file_names
    }

    pub fn set_origin_cache_file_names(&mut self, names: &[String]) {
        self.origin_cache_file_names = names.to_vec().into_boxed_slice();
    }

    pub fn set_final_cache_file_path(&mut self) -> anyhow::Result<()> {
        let cache_file = self.get_cache_file_name();
        let mut files = vec![];
        let _ = cache_file.iter().for_each(|name| {
            let file = format!("{}\\{}", self.get_scoop_cache_dir(), name);
            files.push(file);
        });

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
        if self.app_name == "hp" {
            log::debug!("start updating self process");
            // self.target_rename_alias = Box::from(vec!["hp_updater.exe".to_string()]); 
           self.target_rename_alias = a.into_boxed_slice();
        } else {
            self.target_rename_alias = a.into_boxed_slice();
        }
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
                    alias.push("");
                    s.to_string()
                }
            })
            .collect::<Vec<_>>();
        self.download_urls = download_urls.to_vec().into_boxed_slice();
        let origin_files = download_urls
            .iter()
            .map(|url| {
                // log::debug!("{}", format!("Downloading '{}'", url).dark_blue().bold());
                if !self.check_is_no_special_char_url(url) {
                    let file_name =
                        get_parse_url_query(url).expect(&format!("Could not parse url {}", url));
                    file_name
                } else {
                    let file_name = url.split('/').last().unwrap();
                    file_name.to_string()
                }
            })
            .collect::<Vec<_>>();
        self.set_origin_cache_file_names(&origin_files);
        self.set_target_rename_alias(alias);
    }

    pub fn check_is_no_special_char_url(&self, url: &str) -> bool {
        let special_chars = ['?', '&', '=', '#', '%'];
        for char in special_chars {
            if url.contains(char) {
                return false;
            }
        }
        true
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
        if env_path.contains(dir) && !dir.is_empty() {
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

    pub fn set_app_download_architecture(&mut self, arch: &str) {
        self.install_arch = Cow::Owned(arch.to_string())
    }

    pub fn get_app_download_architecture(&self) -> Cow<'a, str> {
        self.install_arch.clone()
    }

    pub fn set_whether_exe_setup_installer(&mut self, is_setup: bool) {
        self.exe_setup = is_setup;
    }
    fn init(&mut self, manifest_path: &'a str) -> anyhow::Result<()> {
        let download_cache_dir = if self.options.contains(&Global) {
            get_cache_dir_path_global()
        } else {
            get_cache_dir_path()
        };
        let app_name = Path::new(manifest_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        self.set_download_app_name(app_name);

        self.ensure_version_dir_exist()?;

        let content = std::fs::read_to_string(manifest_path)?;
        let serde_obj = serde_json::from_str::<InstallManifest>(&content)?;
        let version = serde_obj.version.expect("version 不能为空");
        let innosetup = serde_obj.innosetup;
        let exe_setup = innosetup.unwrap_or(false);
        self.set_whether_exe_setup_installer(exe_setup);
        self.set_app_version(&version);
        self.set_scoop_cache_dir(&download_cache_dir);
        let architecture = serde_obj.architecture;

        let hash = serde_obj.hash;

        self.set_hash_format(hash, architecture.clone())?;

        let url = serde_obj.url;
        if url.is_some() {
            self.set_cache_file_name(app_name, &version, &url.clone().unwrap())?;
            let final_arch = self.get_user_options_arch()?;
            self.set_app_download_architecture(&final_arch);
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
            self.set_app_download_architecture(&final_arch);
        }
        self.set_app_current_dir();
        self.set_app_version_dir();
        self.set_input_file();
        self.create_input_file()?;
        self.set_final_cache_file_path()?;
        Ok(())
    }

    pub fn new(
        options: &'a [InstallOptions],
        manifest_path: &'a str,
        bucket_source: Option<&'a str>,
    ) -> DownloadManager<'a> {
        let mut download = Self {
            app_name: "",
            manifest_path,
            app_version: "".into(),
            bucket_source,
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
            install_arch: Cow::from(""),
            origin_cache_file_names: Box::new([]),
            archive_files_format: Box::new([]),
            exe_setup: false,
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

        let archive_formats = extensions
            .iter()
            .map(|extension| match extension.to_lowercase().as_str() {
                "7z" => ArchiveFormat::SevenZip,
                "zip" => ArchiveFormat::ZIP,
                "gz" => ArchiveFormat::GZIP,
                "xz" => ArchiveFormat::XZIP,
                "bz2" => ArchiveFormat::BZIP2,
                "zst" => ArchiveFormat::ZSTD,
                "rar" => ArchiveFormat::RAR,
                "exe" => {
                    if self.exe_setup {
                        ArchiveFormat::INNO
                    } else {
                        ArchiveFormat::EXE
                    }
                }
                "msi" => ArchiveFormat::MSI,
                "tar" => ArchiveFormat::TAR,
                _ => ArchiveFormat::Other,
            })
            .collect::<Vec<_>>();
        self.set_archive_files_format(archive_formats);

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
            let result =  cache_file_path.iter().try_for_each(|path| {
               if !Path::new(path).exists() { 
                  return  Ok(())
               }
                println!(
                    "{}",
                    format!("Override Cache File '{path}'").dark_grey().bold()
                );
                std::fs::remove_file(path)
            });
           if result.is_err() {
             bail!("this app cache file is not exist, you can directly install")
           }
        }
        if self
            .options
            .contains(&InstallOptions::OnlyDownloadNoInstall)
        {
            let end_message = if self.bucket_source.is_none() {
                format!("from manifest file '{}'", self.manifest_path)
            } else {
                format!("from bucket '{}'", self.bucket_source.clone().unwrap())
            };
            println!(
                "{}",
                format!(
                    "Downloading '{}' [{}] {}",
                    self.app_name, self.install_arch, end_message
                )
                .dark_blue()
                .bold()
            );
        }
        let final_caches = self.final_cache_file_path.as_ref();
        let result = final_caches.iter().all(|path| Path::new(path).exists());
        if result {
            self.origin_cache_file_names.iter().for_each(|name| {
                println!(
                    "{} {} {}",
                    "Loading".dark_blue().bold(),
                    name.to_string().dark_cyan().bold(),
                    "from cache".blue().bold()
                )
            });
            if Path::new(&input_file).exists() {
                // log::debug!("start remove aria2 input file");
                std::fs::remove_file(input_file)?;
            }
            return Ok(());
        }
        let output = aria2c.invoke_aria2c_download();

        match output {
            Ok(output) => {
                println!("{}", output);
                if Path::new(&input_file).exists() {
                    log::debug!("start remove aria2 input file");
                    std::fs::remove_file(input_file)?;
                }
                Ok(())
            }
            Err(e) => {
                if Path::new(&input_file).exists() {
                    log::debug!("start remove aria2 input file");
                    std::fs::remove_file(input_file)?;
                }
                eprintln!("Error : {}", e.to_string().dark_red().bold());
                std::fs::remove_file(input_file)?;
                Ok(())
            }
        }
    }

    pub fn check_cache_file_hash(&self) -> anyhow::Result<()> {
        let cache_files = self.get_final_cache_file_path();
        let hash_formats = self.get_hash_format();
        let hash_values = self.get_hash_value();
        let origin_names = self.get_origin_cache_file_names();
        let result = cache_files
            .iter()
            .zip(hash_formats)
            .zip(hash_values)
            .zip(origin_names)
            .try_for_each(|(((file, format), hash_value), origin_name)| {
                let open_file = std::fs::File::open(file)?;
                let mut reader = BufReader::new(open_file);
                let mut buffer = [0; 1024 * 64]; // 一次性读取64KB到缓冲区性能最好

                let caculate_hash = match format {
                    HashFormat::SHA1 => {
                        let mut hasher = Sha1::new();
                        loop {
                            let count = reader.read(&mut buffer)?;
                            if count == 0 {
                                break;
                            }
                            hasher.update(&buffer[..count]);
                        }
                        let caculate_hash = hasher.finalize();
                        let caculate_hash = hex::encode(caculate_hash);
                        caculate_hash
                    }
                    HashFormat::SHA512 => {
                        let mut hasher = Sha512::new();
                        loop {
                            let count = reader.read(&mut buffer)?;
                            if count == 0 {
                                break;
                            }
                            hasher.update(&buffer[..count]);
                        }
                        let caculate_hash = hasher.finalize();
                        let caculate_hash = hex::encode(caculate_hash);
                        caculate_hash
                    }
                    HashFormat::SHA256 => {
                        let mut hasher = Sha256::new();
                        loop {
                            let count = reader.read(&mut buffer)?;
                            if count == 0 {
                                break;
                            }
                            hasher.update(&buffer[..count]);
                        }
                        let caculate_hash = hasher.finalize();
                        let caculate_hash = hex::encode(caculate_hash);
                        caculate_hash
                    }
                    HashFormat::MD5 => {
                        let hash = compute_hash_by_powershell(file, "md5")?;
                        hash
                    }
                };

                if caculate_hash.to_lowercase() != *hash_value.to_lowercase() {
                    bail!(
                        "{} 文件哈希校验失败\n期望hash: {}\n实际hash: {}",
                        file,
                        hash_value,
                        caculate_hash
                    )
                } else {
                    println!(
                        "{} {}......{}",
                        "Checking hash of".dark_blue().bold(),
                        origin_name.to_string().dark_cyan().bold(),
                        "✅"
                    );
                    Ok(())
                }
            }) as anyhow::Result<()>;

        if result.is_err() {
            bail!("{}", result.unwrap_err())
        }
        Ok(())
    }

    pub fn invoke_7z_extract(
        &self,
        extract_dir: Option<StringArrayOrString>,
        extract_to: Option<StringArrayOrString>,
        architecture: Option<ArchitectureObject>,
    ) -> anyhow::Result<()> {
        let mut _7z = SevenZipStruct::new();
        let cache_files = self.get_final_cache_file_path();
        let str_slice: Vec<&str> = cache_files.iter().map(String::as_str).collect();

        if self.options.contains(&Global) {
            let apps = get_apps_path_global();
            _7z.set_apps_root_dir(apps)
        } else {
            _7z.set_apps_root_dir(get_apps_path())
        }
        _7z.set_archive_cache_files_path(str_slice.as_slice());
        _7z.set_app_name(self.app_name);
        _7z.set_archive_names(self.origin_cache_file_names.as_ref());
        _7z.set_app_version(self.app_version.as_str());
        _7z.set_app_manifest_path(self.manifest_path);
        let binding = self.get_target_rename_alias();
        _7z.set_target_alias_name(binding.as_ref());
        _7z.set_archive_format(self.get_archive_files_format());
        _7z.init();
        let extract_dir = if architecture.is_some() {
            let arch = architecture.clone().unwrap();
            let system_arch = self.get_install_arch().as_ref();
            let arch = arch.get_specific_architecture(system_arch);
            if arch.is_some() {
                let arch = arch.unwrap();
                let extract_dir = arch.extract_dir.clone();
                if extract_dir.is_some() {
                    let extract_dir = extract_dir.unwrap();
                    Some(extract_dir)
                } else {
                    extract_dir
                }
            } else {
                extract_dir
            }
        } else {
            extract_dir
        };
        let extract_to = if architecture.is_some() {
            let arch = architecture.unwrap();
            let system_arch = self.get_install_arch().as_ref();
            let arch = arch.get_specific_architecture(system_arch);
            if arch.is_some() {
                let arch = arch.unwrap();
                let extract_to = arch.extract_to.clone();
                if extract_to.is_some() {
                    let extract_to = extract_to.unwrap();
                    Some(extract_to)
                } else {
                    extract_to
                }
            } else {
                extract_to
            }
        } else {
            extract_to
        };
        _7z.invoke_7z_command(extract_dir, extract_to).expect("extract zip failed");
        Ok(())
    }
}

mod test_download_manager {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    fn test_7z_extract() {
        let options = vec![];
        let _d = DownloadManager::new(
            options.as_slice(),
            r"A:\Scoop\buckets\main\bucket\scons.json",
            None,
        );
    }

    #[test]
    fn test_cache_file_name() {
        let option = vec![ForceDownloadNoInstallOverrideCache];
        // let d = DownloadManager::new(&option, r"A:\Scoop\buckets\extras\bucket\7ztm.json");
        // let d = DownloadManager::new(option.as_slice(), r"A:\Scoop\buckets\main\bucket\yazi.json");
        let d = DownloadManager::new(
            option.as_slice(),
            r"A:\Scoop\buckets\main\bucket\bun.json",
            None,
        );
        d.get_final_cache_file_path().iter().for_each(|name| {
            println!("{}", name);
        })
    }

    #[test]
    fn test_check_hash() {
        let options = vec![];
        let d = DownloadManager::new(
            options.as_slice(),
            r"A:\Scoop\buckets\main\bucket\bun.json",
            None,
        );
        d.check_cache_file_hash().unwrap();
    }

    #[test]
    fn test_output() {
        let options = vec![];
        let d = DownloadManager::new(
            options.as_slice(),
            r"A:\Scoop\buckets\main\bucket\bun.json",
            None,
        );
        println!("{}", d.get_app_current_dir());
        d.ensure_install_dir_not_in_env_path().unwrap();
    }

    #[test]
    fn test_target_alias_name() {
        let binding = vec![];
        let d = DownloadManager::new(&binding, r"A:\Scoop\buckets\extras\bucket\sfsu.json", None);
        println!("{:?}", d.get_target_rename_alias())
    }

    #[test]
    fn test_output_arch() {
        let binding = vec![];
        let d = DownloadManager::new(&binding, r"A:\Scoop\buckets\main\bucket\bun.json", None);
        println!("{}", d.get_install_arch());
    }

    #[test]
    fn test_powershell_hash() {
        let temp_file = std::env::temp_dir().join("hash_test.bin");
        use std::process::Command;

        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "(Get-FileHash -Path '{}' -Algorithm {}).Hash",
                    temp_file.to_str().unwrap(),
                    "sha1"
                ),
            ])
            .output()
            .unwrap();
        let output_hash = String::from_utf8_lossy(&output.stdout);
        println!("{}", output_hash);
    }
}
