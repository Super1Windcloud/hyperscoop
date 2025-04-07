use crate::config::get_config_value;
use anyhow::bail;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::{env, fs};
use crate::init_env::get_cache_dir_path;
use crate::utils::utility::is_valid_url;

pub struct Aria2C <'a>  {
     aria2c_path: String,
     download_cache_dir: String,
     download_urls  :  &'a  [ &'a str ]
}

impl <'a> Aria2C<'a > {
    pub fn new() -> Self {
      let  download_cache_dir =  get_cache_dir_path();

      let mut aria = Self {
            aria2c_path: "".to_string(),
            download_cache_dir,
            download_urls: &[],
      };
        aria.init().unwrap();
        aria
    }
    pub fn set_aria2c_path(&mut self, path: &str) {
        self.aria2c_path = path.to_string();
    }
    pub fn get_aria2c_path(&self) -> &str {
        &self.aria2c_path
    }
    fn init(&mut self) -> anyhow::Result<String> {
        let aria2_exe = self.extract_aria2()?;
        self.set_aria2c_path(&aria2_exe);
        Ok(aria2_exe)
    }
    /// 命令行字符串, 或字符串数组, 传递多个参数时, 请使用字符串数组
    pub fn execute_aria2_download_command<'cmd>(
        &self,app_name  :&str,  app_version :&str, url :&str,
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
        let proxy = if !proxy.contains("http://") &&  ! proxy.contains("https://") {
            "http://".to_string() + &proxy
        } else {
            proxy
        };
       if  !is_valid_url(&proxy) {  bail!("Proxy is not valid, url format error"); };
       // let  saved_file_name  = generate_file_name() ;
      
      let output = Command::new(&aria2_exe).arg(format!("--all-proxy={proxy}"))
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

    fn  set_download_urls(&mut self, urls: &'a [&str]) {
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
        let a = Aria2C::new();
         
    }
}
