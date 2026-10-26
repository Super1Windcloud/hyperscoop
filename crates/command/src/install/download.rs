use crate::i18n::tr;
use crate::init_env::{
    get_app_current_dir, get_app_current_dir_global, get_app_version_dir,
    get_app_version_dir_global, get_apps_path, get_apps_path_global, get_cache_dir_path,
    get_cache_dir_path_global, get_persist_app_data_dir, get_persist_app_data_dir_global,
};
use crate::install::InstallOptions::{
    ArchOptions, ForceDownloadNoInstallOverrideCache, ForceInstallOverride, Global,
    NoUseDownloadCache,
};
use crate::install::{ArchiveFormat, Aria2C, HashFormat, InstallOptions, SevenZipStruct};
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::{ArchitectureObject, StringArrayOrString};
use crate::tr_fmt;
use crate::utils::system::{compute_hash_by_powershell, get_system_default_arch};
use crate::utils::utility::{assume_yes_to_cover_folder, get_parse_url_query, is_valid_url};
use anyhow::{Context, bail};
use crossterm::style::Stylize;
use hex;
use sha1::Sha1;
use sha2::Digest;
use sha2::{Sha256, Sha512};
use std::borrow::Cow;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::vec;
#[cfg(windows)]
use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
#[cfg(windows)]
use winreg::RegKey;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DownloadManager<'a> {
    app_name: &'a str,
    manifest_path: &'a str,
    app_version: String,
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
    pub seven_zip: SevenZipStruct<'a>,
    cache_locks: Option<Arc<crate::cache::download::DownloadLocks>>,
}

fn archive_format_from_extension(extension: &str, exe_setup: bool) -> ArchiveFormat {
    match extension.to_lowercase().as_str() {
        "7z" => ArchiveFormat::SevenZip,
        "zip" => ArchiveFormat::ZIP,
        "gz" | "tgz" => ArchiveFormat::GZIP,
        "xz" => ArchiveFormat::XZIP,
        "bz2" => ArchiveFormat::BZIP2,
        "zst" => ArchiveFormat::ZSTD,
        "rar" => ArchiveFormat::RAR,
        "exe" => {
            if exe_setup {
                ArchiveFormat::INNO
            } else {
                ArchiveFormat::EXE
            }
        }
        "msi" => ArchiveFormat::MSI,
        "tar" => ArchiveFormat::TAR,
        "" => ArchiveFormat::Shell,
        "nupkg" => ArchiveFormat::NUPKG,
        _ => ArchiveFormat::Other,
    }
}

impl<'a> DownloadManager<'a> {
    pub fn get_senven_zip_struct(&'a mut self) -> &'a mut SevenZipStruct<'a> {
        &mut self.seven_zip
    }

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
            std::fs::remove_file(&install_json_path)
                .context(format!("Failed to remove old {}", &install_json_path))?;
        }

        let file = std::fs::File::create(&install_json_path)
            .context("Failed to create app install.json")?;

        serde_json::to_writer_pretty(&file, &install_json)
            .context("Failed to write pretty json to install.json")?;

        if Path::new(&write_manifest_path).exists() {
            std::fs::remove_file(&write_manifest_path)
                .context("Failed to remove app old manifest.json".to_string())?;
        }
        std::fs::copy(self.manifest_path, write_manifest_path)
            .context("copy bucket manifest to app current dir failed")?;
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

    pub fn get_final_cache_file_path(&self) -> Box<[String]> {
        self.final_cache_file_path.clone()
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
                let x64 = arch
                    .x64bit
                    .context("Missing 64bit architecture definition")?;
                x64.hash.context("Missing 64bit hash")?
            } else if options_arch == "32bit" {
                let x86 = arch
                    .x86bit
                    .context("Missing 32bit architecture definition")?;
                x86.hash.context("Missing 32bit hash")?
            } else if options_arch == "arm64" {
                let arm64 = arch
                    .arm64
                    .context("Missing arm64 architecture definition")?;
                arm64.hash.context("Missing arm64 hash")?
            } else {
                bail!("Unsupported architecture");
            }
        } else {
            bail!("hash str is empty");
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

    pub fn get_target_rename_alias(&self) -> Vec<String> {
        self.target_rename_alias
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn set_target_rename_alias(&mut self, new_alias: Vec<&str>) {
        let a = new_alias.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        if self.app_name == "hp" {
            log::debug!("start updating self process");
            self.target_rename_alias = Box::from(vec!["hp_updater.exe".to_string()]);
            // self.target_rename_alias = a.into_boxed_slice();
        } else {
            self.target_rename_alias = a.into_boxed_slice();
        }
    }

    pub fn get_download_urls(&self) -> Vec<String> {
        self.download_urls.iter().map(|s| s.to_string()).collect()
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
                    get_parse_url_query(url)
                        .unwrap_or_else(|_| url.split('/').last().unwrap_or("download").to_string())
                } else {
                    let file_name = url.split('/').last().unwrap_or("download");
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
            "{}\\{}_{}_{}_input_file.txt",
            self.get_scoop_cache_dir(),
            self.app_name,
            std::process::id(),
            rand::random::<u64>()
        );
        self.input_file = aria2c_input_file
    }

    pub fn create_input_file(&self) -> anyhow::Result<()> {
        let final_caches = self.final_cache_file_path.as_ref();
        log::debug!("final caches file : {:?}", final_caches);

        let mut file =
            std::fs::File::create(self.get_input_file()).context("Failed to create input file")?;
        let urls = self.get_download_urls();
        let mut seen = std::collections::HashSet::new();
        let result =
            urls.iter()
                .zip(self.get_cache_file_name())
                .try_for_each(|(url, cache_name)| {
                    if !seen.insert(cache_name) {
                        return Ok(());
                    }
                    let target = Path::new(self.get_scoop_cache_dir()).join(cache_name);
                    if crate::cache::download::is_complete(&target) {
                        return Ok(());
                    }
                    crate::cache::download::prepare_aria2(&target)?;
                    let content = format!("{}\n\tout={}.incomplete", url, cache_name);
                    writeln!(file, "{}", content).context("Failed to write input file")?;
                    Ok(())
                }) as anyhow::Result<_>;
        if result.is_err() {
            bail!(
                "{}",
                tr_fmt!(
                    "Failed to create cache input file {file}",
                    "创建缓存输入文件{file}失败",
                    file = self.get_input_file()
                )
            );
        }
        log::debug!("create input file {}", self.get_input_file());

        Ok(())
    }

    pub fn create_aria2c_instance(&self) -> Aria2C<'_> {
        let aria2c = Aria2C::new();
        aria2c
    }

    pub fn set_options(&mut self, options: &'a [InstallOptions]) {
        self.options = options;
    }

    pub fn get_options(&self) -> &'a [InstallOptions<'_>] {
        self.options
    }

    pub fn ensure_install_dir_not_in_env_path(&self) -> anyhow::Result<()> {
        #[cfg(windows)]
        {
            let env_path: String = if self.options.contains(&Global) {
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                let environment_key =
                    r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
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
                log::warn!(
                    "{}",
                    tr_fmt!(
                        "{dir} already exists in PATH environment variable, please check",
                        "{dir} 已经存在于 PATH 环境变量中, 请检查",
                        dir = dir
                    )
                );
            }
        }
        Ok(())
    }

    pub fn ensure_version_dir_exist(&self) -> anyhow::Result<PathBuf> {
        let path = Path::new(self.app_version_dir.as_str());

        if !path.exists() {
            std::fs::create_dir_all(path).context("Failed to create app version directory")?;
        }
        let absolute_path = if path.is_relative() {
            std::env::current_dir()?.join(path)
        } else {
            path.to_path_buf()
        };
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
        if is_valid_url(manifest_path) {
            self.set_scoop_cache_dir(download_cache_dir.as_str());
            self.set_input_file();

            self.set_app_version("remote_url");
            self.set_app_current_dir();
            self.set_app_version_dir();
            let url: StringArrayOrString = StringArrayOrString::String(manifest_path.to_string());
            self.set_cache_file_name(app_name, "remote_url", &url)?;
            self.set_final_cache_file_path()?;
            self.lock_cache()?;
            return Ok(());
        }
        let content = std::fs::read_to_string(manifest_path)
            .context(format!("Failed to read manifest file {}", manifest_path))?;
        let serde_obj = serde_json::from_str::<InstallManifest>(&content)
            .context(format!("Failed to parse manifest file {}", manifest_path))?;
        let version = serde_obj.version.ok_or_else(|| {
            anyhow::anyhow!("{}", tr("Version cannot be empty", "version 不能为空"))
        })?;
        let innosetup = serde_obj.innosetup;
        let exe_setup = innosetup.unwrap_or(false);
        self.set_whether_exe_setup_installer(exe_setup);
        self.set_app_version(&version);
        self.set_scoop_cache_dir(&download_cache_dir);
        let architecture = serde_obj.architecture;

        let hash = serde_obj.hash;

        if self
            .options
            .contains(&InstallOptions::SkipDownloadHashCheck)
        {
            self.hash_format = Box::new([]);
            self.set_hash_value(Box::new([]));
        } else {
            self.set_hash_format(hash, architecture.clone())?;
        }

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
                bail!("{}", tr("Architecture option error", "架构选项错误"));
            }
            self.set_app_download_architecture(&final_arch);
        }
        self.set_app_current_dir();
        self.set_app_version_dir();
        self.ensure_version_dir_exist()?;

        self.set_input_file();
        self.set_final_cache_file_path()?;
        self.lock_cache()?;
        Ok(())
    }

    fn lock_cache(&mut self) -> anyhow::Result<()> {
        self.cache_locks = Some(Arc::new(crate::cache::download::DownloadLocks::acquire(
            Path::new(&self.scoop_cache_dir),
            &self.final_cache_file_path,
        )?));
        Ok(())
    }

    pub fn finish_cache_cleanup(self) {
        let root = PathBuf::from(&self.scoop_cache_dir);
        let protected = self.get_final_cache_file_path();
        // Reused packages age from the last successful fetch/install, not the first download.
        for path in &protected {
            if let Ok(file) = std::fs::OpenOptions::new().write(true).open(path) {
                let _ = file.set_modified(std::time::SystemTime::now());
            }
        }
        drop(self);
        if let Err(error) = crate::cache::auto_cleanup(&root, &protected) {
            log::warn!("Cache cleanup failed: {error:#}");
        }
    }

    pub fn new(
        options: &'a [InstallOptions],
        manifest_path: &'a str,
        bucket_source: Option<&'a str>,
    ) -> anyhow::Result<DownloadManager<'a>> {
        let mut download = Self {
            app_name: "",
            manifest_path,
            app_version: "".into(),
            bucket_source,
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
            seven_zip: SevenZipStruct::new(),
            cache_locks: None,
        };
        download.init(manifest_path)?;
        Ok(download)
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
                bail!("{}", tr("URL cannot be empty", "url 不能为空"));
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
                let file_extension = if last_item.contains('.') {
                    last_item.split('.').last().unwrap()
                } else {
                    ""
                };
                file_extension
            })
            .collect::<Vec<&str>>();
        let archive_formats = extensions
            .iter()
            .map(|extension| archive_format_from_extension(extension, self.exe_setup))
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
        #[cfg(debug_assertions)]
        dbg!(&final_file_names);
        self.cache_file_name = final_file_names;
        Ok(())
    }

    pub fn start_download(&self) -> anyhow::Result<()> {
        let aria2_config = crate::config::get_config_value_no_print("aria2-enabled");
        let aria2_enabled =
            aria2_config.is_empty() || aria2_config.to_lowercase() == "true" || aria2_config == "1";
        let result = self.start_download_inner(aria2_enabled);
        let _ = crate::cache::download::remove_if_exists(Path::new(self.get_input_file()));
        result
    }

    fn start_download_inner(&self, aria2_enabled: bool) -> anyhow::Result<()> {
        self.ensure_install_dir_not_in_env_path()?;
        let scoop_cache_dir = self.get_scoop_cache_dir();
        let input_file = self.get_input_file();
        log::info!("input  file: {}", input_file);
        if self.options.contains(&ForceDownloadNoInstallOverrideCache)
            || self.options.contains(&NoUseDownloadCache)
            || (self.app_name == "hp" && self.options.contains(&ForceInstallOverride))
        {
            let cache_file_path = self
                .get_cache_file_name()
                .iter()
                .map(|name| format!("{}\\{}", self.get_scoop_cache_dir(), name))
                .collect::<Vec<String>>();
            let result: anyhow::Result<()> = cache_file_path.iter().try_for_each(|path| {
                println!(
                    "{}",
                    tr_fmt!(
                        "Overwriting cache file '{path}'",
                        "正在覆盖缓存文件 '{path}'",
                        path = path
                    )
                    .dark_grey()
                    .bold()
                );
                crate::cache::download::remove_download(Path::new(path))?;
                Ok(())
            });
            if let Err(e) = result {
                bail!(
                    "{}",
                    tr_fmt!(
                        "Failed to override cache file: {e}",
                        "覆盖缓存文件失败: {e}",
                        e = e
                    )
                );
            }
        }
        if self
            .options
            .contains(&InstallOptions::OnlyDownloadNoInstall)
        {
            let end_message = if self.bucket_source.is_none() {
                tr_fmt!(
                    "from manifest file '{path}'",
                    "来自 manifest 文件 '{path}'",
                    path = self.manifest_path
                )
            } else {
                tr_fmt!(
                    "from bucket '{bucket}'",
                    "来自仓库 '{bucket}'",
                    bucket = self.bucket_source.clone().unwrap()
                )
            };
            println!(
                "{}",
                tr_fmt!(
                    "Downloading '{name}' [{arch}] {msg}",
                    "正在下载 '{name}' [{arch}] {msg}",
                    name = self.app_name,
                    arch = self.install_arch,
                    msg = end_message
                )
                .dark_blue()
                .bold()
            );
        }
        let final_caches = self.final_cache_file_path.as_ref();
        let result = !final_caches.is_empty()
            && final_caches
                .iter()
                .all(|path| crate::cache::download::is_complete(Path::new(path)));
        if result {
            log::info!("cache file already exist, skip download");
            self.origin_cache_file_names.iter().for_each(|name| {
                println!(
                    "{} {}",
                    tr("Loading from cache:", "正在从缓存加载:")
                        .dark_blue()
                        .bold(),
                    name.to_string().dark_cyan().bold()
                )
            });
            if Path::new(input_file).exists() && !is_valid_url(self.manifest_path) {
                log::debug!("start remove aria2 input file {}", input_file);
                std::fs::remove_file(input_file).context("failed to remove aria2 input file")?;
            }
            return Ok(());
        }
        if !aria2_enabled {
            println!(
                "{}",
                tr(
                    "aria2 is disabled in configuration, downloading via native HTTP client...",
                    "配置中已禁用 aria2，使用原生 HTTP 下载..."
                )
                .dark_yellow()
                .bold()
            );
            return self.download_files_native();
        }

        self.create_input_file()?;
        let mut aria2c = self.create_aria2c_instance();
        aria2c.set_input_file(input_file);
        aria2c.set_scoop_cache_dir(scoop_cache_dir);
        aria2c.set_download_urls(self.get_download_urls().as_slice());
        aria2c.init_aria2c_config()?;

        let output = aria2c.invoke_aria2c_download().and_then(|output| {
            for path in final_caches {
                let target = Path::new(path);
                if !crate::cache::download::is_complete(target) {
                    crate::cache::download::publish(
                        &crate::cache::download::with_suffix(target, ".incomplete"),
                        target,
                    )?;
                }
            }
            Ok(output)
        });

        match output {
            Ok(output) => {
                println!("{}", output);
                if Path::new(&input_file).exists() {
                    log::debug!("start remove aria2 input file");
                    let _ = std::fs::remove_file(&input_file);
                }
                Ok(())
            }
            Err(e) => {
                if Path::new(&input_file).exists() {
                    log::debug!("start remove aria2 input file");
                    let _ = std::fs::remove_file(&input_file);
                }
                eprintln!(
                    "{} ({}), {}",
                    tr("Aria2 download failed", "Aria2 下载失败")
                        .dark_red()
                        .bold(),
                    e,
                    tr(
                        "falling back to native HTTP download...",
                        "正在回退到原生 HTTP 下载..."
                    )
                    .dark_yellow()
                    .bold()
                );
                self.download_files_native()
            }
        }
    }

    pub fn download_files_native(&self) -> anyhow::Result<()> {
        super::native_download::download_files(&self.download_urls, &self.final_cache_file_path)
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
                print!(
                    "{} {}......",
                    "Checking hash of".dark_blue().bold(),
                    origin_name.to_string().dark_cyan().bold(),
                );
                std::io::stdout().flush().unwrap(); // 不刷新缓冲区会等待换行

                let open_file = std::fs::File::open(file)
                    .with_context(|| format!("failed to open cache file {file}"))?;
                let caculate_hash = match format {
                    HashFormat::SHA1 => crate::utils::hash::hash_reader::<Sha1>(open_file)?,
                    HashFormat::SHA512 => crate::utils::hash::hash_reader::<Sha512>(open_file)?,
                    HashFormat::SHA256 => crate::utils::hash::hash_reader::<Sha256>(open_file)?,
                    HashFormat::MD5 => compute_hash_by_powershell(file, "md5")?,
                };

                if !caculate_hash.eq_ignore_ascii_case(hash_value) {
                    let _ = std::fs::remove_file(file);
                    let aria2_file = format!("{}.aria2", file);
                    if Path::new(&aria2_file).exists() {
                        let _ = std::fs::remove_file(&aria2_file);
                    }
                    bail!(
                        "{}",
                        tr_fmt!(
                            "Hash validation failed for '{file}'\nExpected: {expected}\nActual:   {actual}\n(Corrupted cache file was deleted; re-run to download again)",
                            "{file} 文件哈希校验失败\n期望hash: {expected}\n实际hash: {actual}\n(已自动清除损坏的缓存文件，再次运行将重新下载)",
                            file = file,
                            expected = hash_value,
                            actual = caculate_hash
                        )
                    )
                } else {
                    println!("✅");
                    Ok(())
                }
            }) as anyhow::Result<()>;

        result?;
        Ok(())
    }

    pub fn invoke_7z_extract(
        &self,
        extract_dir: Option<StringArrayOrString>,
        extract_to: Option<StringArrayOrString>,
        architecture: Option<ArchitectureObject>,
    ) -> anyhow::Result<SevenZipStruct<'_>> {
        let mut _7z = SevenZipStruct::new();

        let cache_files = self.get_final_cache_file_path();
        let cache_files = cache_files.iter().cloned().collect::<Vec<String>>();

        if self.options.contains(&Global) {
            let apps = get_apps_path_global();
            _7z.set_apps_root_dir(apps)
        } else {
            _7z.set_apps_root_dir(get_apps_path())
        }
        _7z.set_options(self.get_options());
        _7z.set_archive_cache_files_path(cache_files);
        _7z.set_app_name(self.app_name);
        _7z.set_archive_names(self.origin_cache_file_names.as_ref());
        _7z.set_app_version(self.app_version.as_str());
        _7z.set_app_manifest_path(self.manifest_path);
        let binding = self.get_target_rename_alias();
        _7z.set_target_alias_name(binding);
        _7z.set_archive_format(self.get_archive_files_format());
        _7z.init();
        _7z.set_final_cache_file_name(self.get_cache_file_name());

        let extract_dir = if architecture.is_some() {
            let arch = architecture.clone().unwrap();
            let system_arch = self.get_install_arch().as_ref();
            let arch = arch.get_specific_architecture(system_arch);
            if arch.is_some() {
                let arch = arch.unwrap();
                let _extract_dir = arch.extract_dir.clone();
                log::info!("architecture extract_dir: {:?}", _extract_dir);
                if _extract_dir.is_some() {
                    let _extract_dir = _extract_dir.unwrap();
                    Some(_extract_dir)
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
                let _extract_to = arch.extract_to.clone();
                log::info!("architecture extract_to: {:?}", _extract_to);
                if _extract_to.is_some() {
                    let _extract_to = _extract_to.unwrap();
                    Some(_extract_to)
                } else {
                    extract_to
                }
            } else {
                extract_to
            }
        } else {
            extract_to
        };
        log::debug!(
            "extract_dir: {:?}, extract_to: {:?}",
            extract_dir,
            extract_to
        );
        _7z.invoke_7z_command(extract_dir, extract_to)
            .context("Failed to extract archive with 7-Zip")?;
        Ok(_7z.clone())
    }

    pub fn copy_file_to_app_dir_from_remote_url(
        &self,
        app_alias: Option<String>,
        suffix: &str,
    ) -> anyhow::Result<String> {
        let app_name = if app_alias.is_some() {
            let splits = app_alias.clone().unwrap();
            let splits = splits.split(".").collect::<Vec<&str>>();
            if splits.len() == 0 {
                bail!("别名为空，请检查")
            } else if splits.len() == 1 {
                let temp = format!("{}.{}", splits[0], suffix);
                log::debug!("temp: {}", temp);
                temp
            } else if splits.len() == 2 {
                let temp = app_alias.unwrap().to_string();
                log::debug!("temp: {}", temp);
                temp
            } else {
                bail!("app_alias 格式错误，请使用 app_name.exe 或者 app_name 格式")
            }
        } else {
            self.get_download_app_name().to_string() + "." + suffix
        };
        let target = format!("{}\\{}", self.get_app_version_dir(), app_name); // with_extension
        if !Path::new(&target).exists() {
            std::fs::create_dir_all(self.get_app_version_dir())
                .context("failed to create app remote_url dir")?;
        } else {
            let result = assume_yes_to_cover_folder(&target)?;
            if result {
                std::fs::remove_file(&target).context("failed to remove old app file")?;
            } else {
                bail!("取消删除")
            }
        }
        let cache_files = self.get_final_cache_file_path();
        if cache_files.len() != 1 {
            bail!("缓存文件数量不正确")
        }
        let cache_path = cache_files.first().unwrap();
        if !Path::new(&cache_path).exists() {
            bail!("缓存文件不存在 {}", cache_path)
        }
        let cache_file = Path::new(&cache_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        print!(
            "{}  {}......",
            "Extracting archive".dark_blue().bold(),
            cache_file.dark_cyan().bold()
        );
        std::io::stdout().flush()?; // 不刷新缓冲区会等待换行

        std::fs::copy(cache_path.as_str(), target)
            .context("failed to copy cache file to app target")?;

        println!("✅");
        Ok(app_name)
    }

    pub fn link_current_from_remote_url(&self) -> anyhow::Result<()> {
        let version_dir = self.get_app_version_dir();
        if !Path::new(&version_dir).exists() {
            bail!("版本目录不存在 {}", version_dir)
        }
        let current_dir = self.get_app_current_dir();
        if Path::new(&current_dir).is_file() {
            std::fs::remove_file(&current_dir).context("failed to remove app current link file")?;
        }

        let result = crate::utils::system::create_dir_link(version_dir, &current_dir);
        if result.is_err() {
            let _ = std::fs::remove_dir_all(&current_dir);
            crate::utils::system::create_dir_link(version_dir, &current_dir)
                .context("failed to create app current link dir")?;
        }
        println!(
            "{}  {} => {}",
            "Linking".dark_blue().bold(),
            current_dir.dark_green().bold(),
            version_dir.dark_green().bold()
        );
        if self.options.contains(&NoUseDownloadCache) {
            let cache_file_path = self
                .get_cache_file_name()
                .iter()
                .map(|name| format!("{}\\{}", self.get_scoop_cache_dir(), name))
                .collect::<Vec<String>>();
            cache_file_path.iter().for_each(|path| {
                if Path::new(path).exists() {
                    std::fs::remove_file(path).expect("failed to remove cache file");
                }
            });
        }

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
        if let Ok(d) = DownloadManager::new(
            option.as_slice(),
            r"A:\Scoop\buckets\main\bucket\bun.json",
            None,
        ) {
            d.get_final_cache_file_path().iter().for_each(|name| {
                println!("{}", name);
            });
        }
    }

    #[test]
    fn test_check_hash() {
        let options = vec![];
        if let Ok(d) = DownloadManager::new(
            options.as_slice(),
            r"A:\Scoop\buckets\main\bucket\bun.json",
            None,
        ) {
            d.check_cache_file_hash().unwrap();
        }
    }

    #[test]
    fn test_output() {
        let options = vec![];
        if let Ok(d) = DownloadManager::new(
            options.as_slice(),
            r"A:\Scoop\buckets\main\bucket\bun.json",
            None,
        ) {
            println!("{}", d.get_app_current_dir());
            d.ensure_install_dir_not_in_env_path().unwrap();
        }
    }

    #[test]
    fn test_target_alias_name() {
        let binding = vec![];
        if let Ok(d) =
            DownloadManager::new(&binding, r"A:\Scoop\buckets\extras\bucket\sfsu.json", None)
        {
            println!("{:?}", d.get_target_rename_alias())
        }
    }

    #[test]
    fn test_output_arch() {
        let binding = vec![];
        if let Ok(d) =
            DownloadManager::new(&binding, r"A:\Scoop\buckets\main\bucket\bun.json", None)
        {
            println!("{}", d.get_install_arch());
        }
    }

    #[test]
    fn test_powershell_hash() {
        let _temp_file = std::env::temp_dir().join("hash_test.bin");
        let temp_file = Path::new(r"A:\Scoop\cache\goreleaser#2.9.0#c9688c4.zip");
        use std::process::Command;
        let start = std::time::Instant::now();
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "(Get-FileHash -Path '{}' -Algorithm {}).Hash",
                    temp_file.to_str().unwrap(),
                    "sha256"
                ),
            ])
            .output()
            .unwrap();
        let output_hash = String::from_utf8_lossy(&output.stdout);
        println!("{}", output_hash);
        let end = std::time::Instant::now();
        println!(" 用时：{:?}", end - start);
    }
}

#[cfg(test)]
mod cache_regression_tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    fn manager<'a>(
        root: &Path,
        options: &'a [InstallOptions<'a>],
        urls: Vec<String>,
    ) -> DownloadManager<'a> {
        let mut manager = DownloadManager {
            app_name: "fixture",
            manifest_path: "fixture.json",
            app_version: "1".into(),
            scoop_cache_dir: root.to_string_lossy().into_owned(),
            cache_file_name: vec![],
            app_version_dir: String::new(),
            app_current_dir: String::new(),
            bucket_source: None,
            options,
            hash_format: vec![HashFormat::SHA256].into_boxed_slice(),
            hash_value: vec![hex::encode(Sha256::digest(b"abc"))].into_boxed_slice(),
            input_file: String::new(),
            download_urls: Box::new([]),
            target_rename_alias: Box::new([]),
            persist_data_dir: String::new(),
            final_cache_file_path: Box::new([]),
            install_arch: Cow::Borrowed("64bit"),
            origin_cache_file_names: Box::new([]),
            archive_files_format: Box::new([]),
            exe_setup: false,
            seven_zip: SevenZipStruct::new(),
            cache_locks: None,
        };
        manager
            .set_cache_file_name("fixture", "1", &StringArrayOrString::StringArray(urls))
            .unwrap();
        manager.set_final_cache_file_path().unwrap();
        manager.set_input_file();
        manager.lock_cache().unwrap();
        manager
    }

    #[test]
    fn cache_regression_aria2_input_excludes_hits_deduplicates_and_migrates_partials()
    -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let manager = manager(
            temp.path(),
            &[],
            vec![
                "https://example.invalid/a.zip".into(),
                "https://example.invalid/b.zip".into(),
                "https://example.invalid/b.zip".into(),
            ],
        );
        let cached = Path::new(&manager.final_cache_file_path[0]);
        let partial = Path::new(&manager.final_cache_file_path[1]);
        std::fs::write(cached, b"complete")?;
        std::fs::write(partial, b"partial")?;
        std::fs::write(
            crate::cache::download::with_suffix(partial, ".aria2"),
            b"resume",
        )?;
        manager.create_input_file()?;
        let input = std::fs::read_to_string(manager.get_input_file())?;
        assert!(!input.contains("a.zip"));
        assert_eq!(input.matches("https://example.invalid/b.zip").count(), 1);
        assert!(input.contains(&format!("out={}.incomplete", manager.cache_file_name[1])));
        assert!(!partial.exists());
        assert_eq!(std::fs::read(cached)?, b"complete");
        Ok(())
    }

    #[test]
    fn cache_regression_real_aria2_publishes_complete_download_and_reuses_it() -> anyhow::Result<()>
    {
        let temp = tempfile::tempdir()?;
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let url = format!("http://{}/fixture.zip", listener.local_addr()?);
        let manager = manager(temp.path(), &[], vec![url]);
        let target = manager.final_cache_file_path[0].clone();
        let server_target = target.clone();
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(15);
            while Instant::now() < deadline {
                let (mut socket, _) = match listener.accept() {
                    Ok(pair) => pair,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => panic!("{error}"),
                };
                socket
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(&mut socket);
                let mut request = String::new();
                reader.read_line(&mut request).unwrap();
                loop {
                    let mut header = String::new();
                    reader.read_line(&mut header).unwrap();
                    if header == "\r\n" || header.is_empty() {
                        break;
                    }
                }
                assert!(
                    !Path::new(&server_target).exists(),
                    "final cache exposed before download completion"
                );
                socket
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\n")
                    .unwrap();
                if request.starts_with("GET ") {
                    socket.write_all(b"abc").unwrap();
                    return;
                }
            }
            panic!("aria2 did not request the test package");
        });
        manager.start_download_inner(true)?;
        server.join().unwrap();
        manager.check_cache_file_hash()?;
        assert_eq!(std::fs::read(&target)?, b"abc");
        assert!(!crate::cache::download::with_suffix(Path::new(&target), ".incomplete").exists());
        // The server is gone: a second invocation must use the completed cache.
        manager.start_download_inner(true)?;
        Ok(())
    }
}

#[cfg(test)]
mod archive_regression_tests {
    use super::*;

    #[test]
    fn tgz_is_extracted_as_gzip_instead_of_copied() {
        for extension in ["tgz", "TGZ", "gz"] {
            assert_eq!(
                archive_format_from_extension(extension, false),
                ArchiveFormat::GZIP
            );
        }
        assert_eq!(
            archive_format_from_extension("zip", false),
            ArchiveFormat::ZIP
        );
        assert_eq!(
            archive_format_from_extension("exe", false),
            ArchiveFormat::EXE
        );
        assert_eq!(
            archive_format_from_extension("exe", true),
            ArchiveFormat::INNO
        );
        assert_eq!(
            archive_format_from_extension("json", false),
            ArchiveFormat::Other
        );
    }
}
