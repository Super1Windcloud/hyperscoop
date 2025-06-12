use crate::init_env::init_user_scoop;
use crate::install::InstallOptions;
use crate::install::InstallOptions::InteractiveInstall;
use crate::merge::Merge;
use crate::utils::system::get_system_current_time;
use anyhow::{bail, Context};
use chrono::Local;
use crossterm::style::Stylize;
use regex::Regex;
use serde_json::Value;
use std::cmp::Ordering;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path};
use textwrap::LineEnding;
use url::Url;

pub fn compare_versions(ver1: String, ver2: String) -> Ordering {
    // 分割版本号并转换为数字数组
    let v1: Vec<i32> = ver1.split('.').flat_map(|s| s.parse()).collect();
    let v2: Vec<i32> = ver2.split('.').flat_map(|s| s.parse()).collect();

    // 动态比较每一级（自动补零）
    let max_len = v1.len().max(v2.len());
    (0..max_len)
        .map(|i| {
            (
                v1.get(i).copied().unwrap_or(0),
                v2.get(i).copied().unwrap_or(0),
            )
        })
        .find_map(|(a, b)| match a.cmp(&b) {
            Ordering::Equal => None,
            diff => Some(diff),
        })
        .unwrap_or(Ordering::Equal)
}

pub fn add_key_value_to_json(
    file_path: &str,
    new_key: &str,
    new_value: Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read_to_string(file_path)
        .context(format!("Failed to read file {} at line 46", file_path))?;

    let mut json_data: Value = serde_json::from_str(&data)
        .context(format!("Failed to parse file {} at line 49", file_path))?;

    if let Value::Object(ref mut map) = json_data {
        map.insert(new_key.to_string(), new_value);
    } else {
        return Err("Invalid JSON: Expected an object".into());
    }
    fs::write(
        file_path,
        serde_json::to_string_pretty(&json_data)
            .context("Failed to transform JSON to pretty string at line 57")?,
    )
    .context(format!("Failed to write file {} at line 58", file_path))?;
    Ok(())
}

pub fn update_scoop_config_last_update_time() {
    let current = get_system_current_time().unwrap();
    let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home_dir = std::env::var("USERPROFILE").unwrap();
        format!("{}\\.config\\scoop\\config.json", home_dir)
    });
    let config_path = Path::new(&config_path);
    if config_path.exists() {
        let config_file = File::open(config_path).unwrap();
        let mut config_json: Value = serde_json::from_reader(config_file).unwrap();
        if let Some(obj) = config_json.as_object_mut() {
            obj.insert("last_update".into(), current.into());
        }
        let file = File::create(config_path).unwrap();
        serde_json::to_writer_pretty(file, &config_json).unwrap();
    }
}

pub fn get_official_buckets_name() -> Vec<String> {
    let exclude_dirs = [
        "main",
        "extras",
        "versions",
        "nirsoft",
        "sysinternals",
        "php",
        "nerd-fonts",
        "nonportable",
        "java",
        "games",
    ];
    exclude_dirs.iter().map(|s| s.to_string()).collect()
}

pub fn get_official_bucket_path(bucket_name: String) -> String {
    let scoop_home = init_user_scoop();
    format!("{}\\buckets\\{}", scoop_home, bucket_name)
}

pub fn get_official_bucket_urls<'a>() -> Vec<&'a str> {
    vec![
        "https://github.com/ScoopInstaller/Main",
        "https://github.com/ScoopInstaller/Extras",
        "https://github.com/ScoopInstaller/Versions",
        "https://github.com/niheaven/scoop-sysinternals",
        "https://github.com/ScoopInstaller/PHP",
        "https://github.com/matthewjberger/scoop-nerd-fonts",
        "https://github.com/ScoopInstaller/Nonportable",
        "https://github.com/ScoopInstaller/Java",
        "https://github.com/Calinou/scoop-games",
    ]
}

pub fn get_official_with_social_bucket_urls<'a>() -> Vec<&'a str> {
    let mut urls = get_official_bucket_urls();
    urls.extend_from_slice(&[
        "https://github.com/cmontage/scoopbucket",
        "https://github.com/anderlli0053/DEV-tools",
        "https://github.com/okibcn/ScoopMaster",
    ]);
    urls
}
pub fn write_into_log_file_append_mode(path: &str, content: String) {
    let root = Path::new(r"A:\Rust_Project\hyperscoop\log");
    let log_dir = root.join(path);
    if !log_dir.exists() {
        log::info!("{} not exist, create it ", log_dir.to_str().unwrap());
        File::create(&log_dir).unwrap();
    }
    let file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_dir)
        .unwrap();
    let mut writer = std::io::BufWriter::new(file);
    writer
        .write_all((content.to_string() + "\n").as_bytes())
        .unwrap();
}

pub fn write_into_log_one_time(msg: &Vec<Merge>) {
    let log_file = r"A:\Rust_Project\hyperscoop\manifests.txt";
    let file = File::create(log_file).unwrap();
    let mut writer = std::io::BufWriter::new(file);
    let mut str = String::new();
    for merge in msg.iter() {
        str.push_str(&format!(
            "Name :{} Version :{} \n",
            merge.app_name, merge.app_version
        ));
    }
    writer.write_all(str.as_bytes()).unwrap();
}

pub const LARGE_COMMUNITY_BUCKET: [&str; 8] = [
    "https://github.com/anderlli0053/DEV-tools",
    "https://github.com/cmontage/scoopbucket",
    "https://github.com/duzyn/scoop-cn",
    "https://github.com/lzwme/scoop-proxy-cn",
    "https://github.com/kkzzhizhou/scoop-apps",
    "https://github.com/cmontage/scoopbucket-third",
    "https://github.com/okibcn/ScoopMaster",
    "http://github.com/okibcn/ScoopMaster",
];

pub fn remove_bom_and_control_chars_from_utf8_file<P: AsRef<Path>>(
    path: P,
) -> anyhow::Result<String> {
    // 读取文件内容到字节数组
    let data = fs::read(&path)?;

    // 检查是否存在 BOM（0xEF 0xBB 0xBF）
    let data = if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        // 截取 BOM 之后的内容
        &data[3..]
    } else {
        &data
    };

    let mut filtered_data = Vec::new();
    let mut idx = 0;

    while idx < data.len() {
        // 跳过控制字符（0x00 到 0x1F 和 0x7F）
        if data[idx] <= 0x1F || data[idx] == 0x7F && !matches!(data[idx], b'\n' | b'\r' | b' ') {
            idx += 1;
            continue; // 跳过控制字符
        }

        // 尝试解析 UTF-8 字符
        match std::str::from_utf8(&data[idx..]) {
            Ok(s) => {
                if let Some(c) = s.chars().next() {
                    // 保留空格、制表符、换行、回车等
                    if !c.is_control() || c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                        filtered_data.extend_from_slice(&data[idx..idx + c.len_utf8()]);
                    }
                    idx += c.len_utf8(); // 移动到下一个字符
                } else {
                    // 空字符串，直接跳过
                    break;
                }
            }
            Err(_) => {
                // 如果解析失败，跳过当前字节
                idx += 1;
            }
        }
    }
    let content = serde_json::to_string_pretty(&filtered_data)
        .context("Failed to transform filtered data to JSON string at line 189")?;
    fs::write(&path, content).context(format!(
        "Failed to write file {} at line 191",
        path.as_ref().to_str().unwrap()
    ))?;
    let content = fs::read_to_string(&path).context(format!(
        "Failed to read file {} at line 193",
        path.as_ref().to_str().unwrap()
    ))?;
    Ok(content)
}

pub fn assume_yes_to_cover_shim(path: &str) -> anyhow::Result<bool> {
    use dialoguer::Confirm;
    let message = format!("文件{path}已存在,建议检查,是否进行覆盖?(y/n)")
        .dark_cyan()
        .bold()
        .to_string();

    match Confirm::new()
        .with_prompt(message)
        .show_default(false)
        .default(false)
        .interact()
    {
        Ok(yes) => {
            if yes {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(_) => Ok(false),
    }
}

pub fn assume_yes_to_cover_shortcuts(path: &str) -> anyhow::Result<bool> {
    use dialoguer::Confirm;
    let message = format!("快捷方式'{path}'已存在,建议检查,是否进行覆盖?(y/n)")
        .dark_cyan()
        .bold()
        .to_string();

    match Confirm::new()
        .with_prompt(message)
        .show_default(false)
        .default(false)
        .interact()
    {
        Ok(yes) => {
            if yes {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(_) => Ok(false),
    }
}

pub fn assume_yes_to_cover_folder(path: &str) -> anyhow::Result<bool> {
    use dialoguer::Confirm;
    let message = format!("该目录'{path}'已存在,建议检查,是否进行删除?(y/n)")
        .dark_cyan()
        .bold()
        .to_string();

    match Confirm::new()
        .with_prompt(message)
        .show_default(false)
        .default(false)
        .interact()
    {
        Ok(yes) => {
            if yes {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(_) => Ok(false),
    }
}

pub fn write_utf8_file(path: &str, content: &str, option: &[InstallOptions]) -> anyhow::Result<()> {
    // log::info!("shim content :{}", content);
    if content.is_empty() {
        bail!("shim content is empty");
    }

    if Path::new(&path).exists() && option.contains(&InteractiveInstall) {
        let result = assume_yes_to_cover_shim(path)?;
        if !result {
            return Ok(());
        } else {
            log::warn!("{}", "覆盖写入".dark_yellow().bold());
        }
    }
    let mut file =
        File::create(path).context(format!("Failed to create utf8 file {} at line 281", path))?;
    /*
     File::create(path) 的默认行为
    如果文件存在： 会 直接清空文件内容（相当于 truncate 模式），然后写入新数据。
    不会报错，但原内容会丢失！
    如果文件不存在： 创建新文件并写入内容。*/
    let crlf_content = content.replace(LineEnding::LF.as_str(), LineEnding::CRLF.as_str());
    file.write_all(crlf_content.as_bytes())
        .context(format!("Failed to write utf8 file {} at line 289", path))?; // 一次性全部写入
    Ok(())
}

pub fn is_valid_url(url_str: &str) -> bool {
    if let Ok(url) = Url::parse(url_str) {
        matches!(url.scheme(), "http" | "https")
    } else {
        false
    }
}

pub fn validate_version(version: &str) -> anyhow::Result<()> {
    // 定义允许的字符：字母、数字、点(.)、横线(-)、加号(+)、下划线(_)
    let re = Regex::new(r"[^\w.\-+]")?; // \w 包含下划线，所以不需要单独加 _

    if let Some(captures) = re.captures(version) {
        let invalid_char = captures.get(0).unwrap().as_str();
        bail!(format!(
            "Manifest version has unsupported character '{}'.",
            invalid_char
        ));
    }

    Ok(())
}

pub fn nightly_version() -> anyhow::Result<String> {
    eprintln!("⚠️ This is a nightly version. Downloaded files won't be verified.");
    let date = Local::now().format("%Y%m%d").to_string();
    Ok(format!("nightly-{}", date))
}

/// 判断URl是否存在query参数
pub fn get_parse_url_query(url: &str) -> anyhow::Result<String> {
    let url = Url::parse(url).context(format!("Failed to parse '{}' as a URL", url))?;
    if let Some(query) = url.query() {
        let last_equal_item = query.rsplit('=').next().unwrap();
        Ok(last_equal_item.to_string())
    } else {
        Ok(url.path().split('/').last().unwrap().to_string())
    }
}

pub fn strip_extended_prefix(path: &Path) -> String {
    let s = path.display().to_string();
    if s.starts_with(r"\\?\") {
        s[4..].to_string()
    } else {
        s
    }
}

pub fn clap_args_to_lowercase(s: &str) -> Result<String, String> {
    Ok(s.to_lowercase())
}

pub fn target_version_dir_to_current_dir(
    target_path: &str,
    options: &[InstallOptions],
) -> anyhow::Result<String> {
    let version = if let Some(InstallOptions::CurrentInstallApp { app_version, .. }) =
        options.iter().find_map(|opt| {
            if let InstallOptions::CurrentInstallApp {
                app_name,
                app_version,
            } = opt
            {
                Some(InstallOptions::CurrentInstallApp {
                    app_name: app_name.clone(),
                    app_version: app_version.clone(),
                })
            } else {
                None
            }
        }) {
        app_version
    } else {
        "".into()
    };
    if version.is_empty() {
        eprintln!("No version found, will use canonicalize path to replace link dir.")
    }
    let symbolic_dir = if version.is_empty() {
        target_path.to_string()
    } else {
        target_path.replace(version.as_str(), "current")
    };
    log::info!("target link dir: {}", symbolic_dir);
    Ok(symbolic_dir)
}

pub fn exclude_scoop_self_scripts(
    script_name: &str,
    alias_name: Option<&str>,
) -> anyhow::Result<u8> {
    let split = script_name.split(".").collect::<Vec<&str>>();
    if split.len() != 2 && split.len() != 1 {
        bail!("shim target {script_name} 文件名格式错误, WTF?")
    }
    if alias_name.is_some() {
        let script_name = alias_name.unwrap();
        #[cfg(debug_assertions)]
        dbg!(script_name);
        let exclude_list = vec!["scoop", "scoop-pre", "scoop-premake", "scoop-rm_nm"];
        if exclude_list.contains(&script_name) {
            return Ok(1);
        }
        return Ok(0);
    }
    let script_name = split.get(0).unwrap();
    let exclude_list = vec!["scoop", "scoop-pre", "scoop-premake", "scoop-rm_nm"];
    if exclude_list.contains(&script_name) {
        return Ok(1);
    }
    Ok(0)
}

pub fn extract_target_path_from_shell_script(file_path: &str) -> anyhow::Result<String> {
    let content = fs::read_to_string(file_path)?;
    let second_line = content.lines().nth(1).unwrap();

    // 移除 # 注释符号和前后空格
    let path = second_line.trim_start_matches('#').trim();

    if !path.is_empty() {
        Ok(path.to_string())
    } else {
        bail!("脚本文件{}第二行为空", file_path)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    #[ignore]
    fn test_compare_versions() {
        assert_eq!(
            compare_versions("1.2.3".to_string(), "1.2.3".to_string()),
            Ordering::Equal
        );
    }

    #[test]
    #[ignore]
    fn test_rm_bom() {
        let path = r"A:\Scoop\buckets\echo\bucket\hdtune.json";
        let content = remove_bom_and_control_chars_from_utf8_file(path).unwrap();
        let _ = serde_json::from_str::<Value>(&content).unwrap();
    }

    #[test]
    fn test_log_file() {
        write_into_log_file_append_mode("git2_clone.txt", "superwindcloud".to_string());
    }

    #[test]
    fn test_path_to_str() {
        let path = r"A:\Scoop\apps\mise\current\bin/mise.exe";
        match fs::canonicalize(Path::new(path)) {
            Ok(standardized) => println!("标准路径: {}", standardized.display()),
            Err(e) => eprintln!("错误: {}", e),
        }
    }
}
