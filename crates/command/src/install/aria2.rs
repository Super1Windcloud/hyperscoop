use crate::config::get_config_value;
use crate::init_env::{get_cache_dir_path, get_cache_dir_path_global};
use crate::install::InstallOptions::Global;
use crate::install::{HashFormat, InstallOptions};
use crate::utils::utility::is_valid_url;
use anyhow::bail;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::{env, fs};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Aria2C<'a> {
    aria2c_path: String,
    scoop_cache_dir: String,
    aria2c_download_config: Vec<&'a str>,
    download_urls: &'a [&'a str],
    install_options: &'a [InstallOptions],
    app_name: &'a str,
    app_version: &'a str,
    hash: HashFormat,
}

impl<'a> Aria2C<'a> {
    pub fn new(options: &'a [InstallOptions]) -> Self {
        let mut aria = Self {
            aria2c_path: "".to_string(),
            scoop_cache_dir: "".into(),
            aria2c_download_config: vec![],
            download_urls: &[],
            install_options: options,
            app_name: "",
            app_version: "",
            hash: HashFormat::SHA256,
        };
        aria.init(options).unwrap();
        aria
    }
    pub fn get_aria2c_download_config(&self) -> Vec<&'a str> {
        self.aria2c_download_config.clone()
    }
    pub fn add_aria2c_download_config(&mut self, config: &'a str) {
        self.aria2c_download_config.push(config);
    }
    pub fn init_aria2c_config(&mut self) {
        let args = vec![
            "-x16", // 最大连接数
            "-s16", // 分片数量
            "-k1M", // 每块大小
            "--file-allocation=falloc".into(),
            "--enable-http-keep-alive=true".into(),
            "--enable-http-pipelining=true".into(),
            "--max-connection-per-server=16".into(),
            "--min-split-size=1M".into(),
            "--summary-interval=0".into(),
            "--continue=true".into(),
            "--timeout=10".into(),
            "--retry-wait=3".into(),
            "--allow-overwrite=true".into(),
            "--auto-file-renaming=false".into(),
        ];
        self.aria2c_download_config = args;
    }
    pub fn set_download_app_name(&mut self, app_name: &'a str) {
        self.app_name = app_name;
    }
    pub fn get_download_app_name(&self) -> &'a str {
        self.app_name
    }
    pub fn get_app_version(&self) -> &'a str {
        self.app_version
    }
    pub fn set_app_version(&mut self, app_version: &'a str) {
        self.app_version = app_version;
    }
    pub fn set_aria2c_path(&mut self, path: &str) {
        self.aria2c_path = path.to_string();
    }
    pub fn get_aria2c_path(&self) -> &str {
        &self.aria2c_path
    }
    pub fn get_scoop_cache_dir(&self) -> &str {
        &self.scoop_cache_dir
    }
    pub fn set_scoop_cache_dir(&mut self, path: &str) {
        self.scoop_cache_dir = path.to_string();
    }
    fn init(&mut self, options: &[InstallOptions]) -> anyhow::Result<String> {
        let download_cache_dir = if options.contains(&Global) {
            get_cache_dir_path_global()
        } else {
            get_cache_dir_path()
        };
        self.set_scoop_cache_dir(&download_cache_dir);
        let aria2_exe = self.extract_aria2()?;
        self.set_aria2c_path(&aria2_exe);
        Ok(aria2_exe)
    }
    #[must_use]
    pub fn execute_aria2_download_command<'cmd>(
        &self,
        url: &str,
        command_str: impl Into<Option<&'cmd str>>,
        command_arr: impl Into<Option<&'cmd [&'cmd str]>>,
    ) -> anyhow::Result<String> {
        let aria2_exe = self.get_aria2c_path();
        let (command_str, command_arr) = (command_str.into(), command_arr.into());
        if command_str.is_none() && command_arr.is_none() {
            bail!("No command str  or command arr provided");
        };
        let (command_str, command_arr) = (
            command_str.unwrap_or_default(),
            command_arr.unwrap_or_default(),
        );
        let command_arr = command_arr
            .iter()
            .map(|item| item.trim())
            .collect::<Vec<&str>>();
        let proxy = get_config_value("proxy");
        let proxy = if !proxy.contains("http://") && !proxy.contains("https://") {
            "http://".to_string() + &proxy
        } else {
            proxy
        };
        if !is_valid_url(&proxy) {
            bail!("Proxy is not valid, url format error");
        };
        // let  saved_file_name  = generate_file_name() ;

        let output = Command::new(&aria2_exe)
            .arg(format!("--all-proxy={proxy}"))
            .args(command_arr)
            .arg(command_str.trim())
            .output()?;
        let error_str = String::from_utf8_lossy(&output.stderr).to_string();
        let output_str = String::from_utf8_lossy(&output.stdout).to_string();
        if !error_str.is_empty() {
            bail!(error_str);
        }
        if output_str.is_empty() {
            bail!("aria2c 命令执行失败,返回结果为空");
        }
        Ok(output_str)
    }

    fn write_aria2_to_temp(&self, aria2_exe: &str) -> anyhow::Result<()> {
        const COMPRESSED_ARIA2: &[u8] = include_bytes!("../../../../resources/aria2c_data.gz");
        let mut decoder = flate2::read::GzDecoder::new(COMPRESSED_ARIA2);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        let mut file = File::create(aria2_exe)?;
        file.write_all(&decompressed_data)?;
        file.flush()?;
        file.sync_all()?;
        drop(file); // 需关闭句柄才能调用
        log::warn!("写入成功 , aria2c_path  = {}", aria2_exe);
        Ok(())
    }

    fn extract_aria2(&self) -> anyhow::Result<String> {
        let aria2_exe = self.get_temp_aria2_path();
        if !Path::new(&aria2_exe).exists() {
            self.write_aria2_to_temp(&aria2_exe)?;
        }
        Ok(aria2_exe)
    }

    fn get_temp_aria2_path(&self) -> String {
        let temp_dir = env::temp_dir();
        let exe_path = temp_dir.join("aria2c.exe");
        exe_path.to_str().unwrap().to_string()
    }

    fn set_download_urls(&mut self, urls: &'a [&str]) {
        self.download_urls = urls;
    }
}

pub fn write_message_to_aria2_log(message: &str) {
    let cwd = env::current_dir().unwrap();
    let log = cwd
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("log")
        .join("aria2.log");
    let log_path = log.as_path();
    if !log_path.exists() {
        File::create(log_path).unwrap();
    }
    let file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_path)
        .unwrap();
    let mut writer = std::io::BufWriter::new(file);
    writer
        .write_all((message.to_string() + "\n").as_bytes())
        .unwrap();
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_aria2() {
        // let a = Aria2C::new();
    }
}
