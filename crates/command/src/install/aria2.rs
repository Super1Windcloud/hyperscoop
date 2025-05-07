use crate::config::get_config_value_no_print;
use crate::utils::utility::is_valid_url;
use anyhow::bail;
use crossterm::style::Stylize;
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs};
use which::which;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Aria2C<'a> {
    aria2c_path: Cow<'a, str>,
    aria2c_download_config: Vec<&'a str>,
    input_file: &'a str,
    cache_file_name: Box<[String]>,
    scoop_cache_dir: &'a str,
    final_download_path: Box<[String]>,
}

impl<'a> Aria2C<'a> {
    pub fn get_final_download_path(&self) -> &Box<[String]> {
        &self.final_download_path
    }
    pub fn set_final_download_path(&mut self) {}
    pub fn get_scoop_cache_dir(&self) -> &'a str {
        self.scoop_cache_dir
    }
    pub fn set_scoop_cache_dir(&mut self, scoop_cache_dir: &'a str) {
        self.scoop_cache_dir = scoop_cache_dir;
    }
    fn init(&mut self) -> anyhow::Result<()> {
        self.init_aria2c_config();
        let aria2_path = self.extract_aria2()?;
        log::debug!("aria2c.exe : {}", aria2_path.as_str());
        self.set_aria2c_path(Cow::Owned(aria2_path));
        Ok(())
    }
    pub fn get_input_file(&self) -> String {
        self.input_file.to_string()
    }
    pub fn set_input_file(&mut self, input_file: &'a str) {
        self.input_file = input_file;
    }
    pub fn load_input_file(&self) -> anyhow::Result<()> {
        let input_file = self.get_input_file();
        if !Path::new(&input_file).exists() {
            bail!("{} does not exist", input_file);
        }
        let _ = fs::read_to_string(&input_file)?;
        Ok(())
    }

    pub fn new() -> Self {
        let mut aria = Self {
            aria2c_path: Cow::from(""),
            aria2c_download_config: vec![],
            input_file: "",
            cache_file_name: Box::new([]),
            scoop_cache_dir: "",
            final_download_path: Box::new([]),
        };
        match aria.init() {
            Ok(_) => aria,
            Err(e) => {
                println!("Error: {}", e.to_string().dark_red().bold());
                aria
            }
        }
    }
    pub fn get_aria2c_download_config(&self) -> Vec<&'a str> {
        self.aria2c_download_config.clone()
    }
    pub fn add_aria2c_download_config(&mut self, config: &'a str) {
        self.aria2c_download_config.push(config);
    }
    pub fn init_aria2c_config(&mut self) {
        let args = vec![
            "--optimize-concurrent-downloads=true",
            "--enable-http-pipelining=true",
            "--enable-color=true",      //  启用颜色输出
            "retry-wait=3",             // 重试等待时间
            "auto-file-renaming=false", // 不自动重命名文件
            "--allow-overwrite=true",
            "--metalink-preferred-protocol=https", // 优先使用 HTTPS 协议下载 Metalink 文件
            "--min-tls-version=TLSv1.2",           // 最小 TLS 版本
            "--check-certificate=false",           // 跳过证书验证
            "--max-connection-per-server=16",      // 单服务器最大连接数
            "--split=16",                          // 分片数
            "--console-log-level=warn",            // 日志级别
            "--follow-metalink=true",              // 支持 Metalink 下载
            "--min-split-size=5M",                 // 最小分片大小
            "--continue=true",                     // 断点续传
            "--file-allocation=trunc",             // windows下文件预分配磁盘空间（SSD推荐）
            "--summary-interval=0",                // 不频繁输出日志减少IO
            "--auto-save-interval=1",              //  自动保存间隔
            "--disable-ipv6=true",                 // 禁用 IPv6（如果不需要）
            "--no-file-allocation-limit=500M",     // 大文件不预分配（SSD 可启用）
            "--async-dns=true",                    // 异步 DNS 解析
        ];
        self.aria2c_download_config = args;
    }

    pub fn set_aria2c_path(&mut self, path: Cow<'a, str>) {
        self.aria2c_path = path
    }
    pub fn get_aria2c_path(&self) -> &str {
        &self.aria2c_path
    }
    pub fn get_scoop_user_agent(&self) -> String {
        let os_info = os_info::get();
        let os_version = os_info.version().to_string();
        // 检测系统架构
        let arch = env::consts::ARCH;
        let mut arch_info = String::new();

        // 检查是否是 ARM64
        if cfg!(target_arch = "aarch64") {
            arch_info.push_str("ARM64; ");
        }
        // 检查是否是 AMD64 (x86_64)
        else if arch == "x86_64" {
            arch_info.push_str("Win64; x64; ");
        }

        // 检查是否运行在 WOW64 模式下（32位程序在64位系统）
        if let Ok(program_files_arm) = env::var("ProgramFiles(Arm)") {
            if !program_files_arm.is_empty() {
                arch_info.push_str("WOW64; ");
            }
        }

        format!(
            "Scoop/1.0 (+http://scoop.sh/) Rust/{} (Windows NT {}; {}){}",
            env!("CARGO_PKG_VERSION"),
            os_version,
            arch_info,
            if cfg!(windows) { "Windows" } else { "" }
        )
    }
    pub fn invoke_aria2c_download<'cmd>(&self) -> anyhow::Result<String> {
        let aria2_exe = self.get_aria2c_path();
        println!(
            "{}",
            "Starting Aria2 Download Files......".dark_blue().bold()
        );
        let proxy = get_config_value_no_print("proxy");
        let proxy =
            if !proxy.contains("http://") && !proxy.contains("https://") && !proxy.is_empty() {
                "http://".to_string() + &proxy
            } else {
                proxy
            };
      let proxy = if proxy.is_empty() {
        env::var_os("HTTPS_PROXY")
          .ok_or(env::var_os("HTTP_PROXY"))
          .unwrap_or_default()
          .to_str()
          .unwrap()
          .to_string()
      } else {
        proxy
      };
      
        if !is_valid_url(&proxy) && !proxy.is_empty() {
            bail!("Proxy is not valid, url format error");
        };
        log::info!("aria2 download proxy : {}", proxy);
        let input_file = self.get_input_file();
        let user_agent = self.get_scoop_user_agent();
        let cache_dir = self.get_scoop_cache_dir();
        let child = Command::new(&aria2_exe)
            .arg(format!("--dir={}", &cache_dir))
            .arg(format!("--user-agent={}", user_agent))
            .arg(format!("--all-proxy={proxy}"))
            .arg(format!("--input-file={input_file}"))
            .args(self.get_aria2c_download_config())
            .stdout(Stdio::inherit()) // 将标准输出重定向到父进程终端
            .output()?; // 阻塞进程

        let status = child.status;
        let result = if status.success() {
            Ok("Aria2c download completed successfully!"
                .dark_green()
                .bold()
                .to_string())
        } else {
            bail!(
                "Aria2c download failed, exit code: {},\n Error :{}",
                status.code().unwrap(),
                String::from_utf8_lossy(&child.stderr).as_ref()
            )
        };
        result
    }
    pub fn invoke_aria2c_download_async<'cmd>(&self) -> anyhow::Result<String> {
        let aria2_exe = self.get_aria2c_path();
        println!(
            "{}",
            "Starting aria2 download file ......".dark_blue().bold()
        );
        let proxy = get_config_value_no_print("proxy");
        let proxy =
            if !proxy.contains("http://") && !proxy.contains("https://") && !proxy.is_empty() {
                "http://".to_string() + &proxy
            } else {
                proxy
            };
        let proxy = if proxy.is_empty() {
            env::var_os("HTTPS_PROXY")
                .ok_or(env::var_os("HTTP_PROXY"))
                .unwrap_or_default()
                .to_str()
                .unwrap()
                .to_string()
        } else {
            proxy
        };
        if !is_valid_url(&proxy) {
            bail!("Proxy is not valid, url format error");
        };
        let input_file = self.get_input_file();
        let user_agent = self.get_scoop_user_agent();
        let cache_dir = self.get_scoop_cache_dir();
        let mut child = Command::new(&aria2_exe)
            .arg(format!("--dir={}", &cache_dir))
            .arg(format!("--user-agent={}", user_agent))
            .arg(format!("--all-proxy={proxy}"))
            .arg(format!("--input-file={input_file}"))
            .args(self.get_aria2c_download_config())
            .stdout(Stdio::piped()) // 将标准输出到管道
            .stderr(Stdio::piped())
            .spawn()?; // 阻塞进程

        if let Some(mut stdout) = child.stdout.take() {
            let mut buffer = [0u8; 1024];
            loop {
                let n = stdout.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                let output = String::from_utf8_lossy(&buffer[..n]);
                for line in output.lines() {
                    println!("Download: {}", line);
                }
            }
        }
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                let line = line?;
                println!("Download Error: {}", line);
            }
        }
        let status = child.wait()?; // 等待子进程结束
        let result = if status.success() {
            Ok("Aria2c download completed successfully"
                .dark_green()
                .bold()
                .to_string())
        } else {
            bail!(
                "Aria2c download failed, exit code: {}",
                status.code().unwrap()
            )
        };
        result
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
        let aria2 = which("aria2c");

        let aria2_exe = if aria2.is_ok() {
            aria2?.to_str().unwrap().to_string()
        } else {
            self.get_temp_aria2_path()
        };
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
        let _a = Aria2C::new();
    }

    #[test]
    fn test_agent() {
        let a = Aria2C::new();
        println!("{}", a.get_scoop_user_agent())
    }
}
