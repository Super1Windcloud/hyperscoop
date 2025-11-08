use crate::i18n::tr;
use anyhow::Context;
use color_eyre::owo_colors::OwoColorize;
use command_util_lib::init_env::{init_scoop_global, init_user_scoop};
use crossterm::style::Stylize;
use std::process::Command;
use std::{env, path::Path};
use which::which;
use windows::core::PCWSTR;
use windows::Wdk::System::SystemServices::RtlGetVersion;
use windows::Win32::Storage::FileSystem::GetVolumeInformationW;
use windows::Win32::System::Diagnostics::Debug::VER_PLATFORM_WIN32_NT;
use windows::Win32::System::SystemInformation::OSVERSIONINFOW;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
enum CheckupError {
    #[error("Windows API error")]
    WindowsError,
    #[error("Registry error")]
    RegistryError,
    #[error("Service error")]
    ServiceError,
}

struct CheckupResult {
    passed: bool,
    message: String,
    fix_hint: Option<String>,
}

fn localized(en: &str, zh: &str) -> String {
    tr(en, zh).to_string()
}

pub async fn execute_checkup_command(global: bool) -> anyhow::Result<()> {
    let mut total_issues = 0;
    let defender_issues = 0;

    let main_bucket_result = check_main_bucket(global)?;
    if !main_bucket_result.passed {
        total_issues += 1;
        print_result(&main_bucket_result);
    }
    let lessmsi_result = check_lessmsi()?;
    if !lessmsi_result.passed {
        total_issues += 1;
        print_result(&lessmsi_result);
    }

    let av_result = check_common_antivirus()?;
    if !av_result.passed {
        total_issues += 1;
        print_result(&av_result);
    }

    let aria2c_result = check_aria2c_path()?;
    if !aria2c_result.passed {
        total_issues += 1;
        print_result(&aria2c_result);
    }

    let curl_result = check_curl_path()?;
    if !curl_result.passed {
        total_issues += 1;
        print_result(&curl_result);
    }

    let innounp_result = check_innounp()?;
    if !innounp_result.passed {
        total_issues += 1;
        print_result(&innounp_result);
    }
    let github_result = check_github().await?;
    let git_result = check_git()?;
    if !github_result.passed {
        total_issues += 1;
        print_result(&github_result);
    }
    if !git_result.passed {
        total_issues += 1;
        print_result(&git_result);
    }
    let dark_result = check_dark()?;
    if !dark_result.passed {
        total_issues += 1;
        print_result(&dark_result);
    }

    let long_paths_result = check_long_paths()?;
    if !long_paths_result.passed {
        total_issues += 1;
        print_result(&long_paths_result);
    }

    let dev_mode_result = check_developer_mode()?;
    if !dev_mode_result.passed {
        total_issues += 1;
        print_result(&dev_mode_result);
    }

    let seven_zip_result = check_7zip()?;
    if !seven_zip_result.passed {
        total_issues += 1;
        print_result(&seven_zip_result);
    }

    let ntfs_result = check_ntfs_volumes()?;
    if !ntfs_result.passed {
        total_issues += 1;
        print_result(&ntfs_result);
    }

    let virus_result = check_antivirus()?;
    if !virus_result.passed {
        total_issues += 1;
        print_result(&virus_result);
    }
    if total_issues > 0 {
        println!(
            "{}",
            format!(
                tr(
                    "Found {count} potential issues.",
                    "发现 {count} 个潜在问题。"
                ),
                count = total_issues
            )
            .yellow()
        );
    } else if defender_issues > 0 {
        println!(
            "{}",
            format!(
                tr(
                    "Found {count} performance issues.",
                    "发现 {count} 个性能问题。"
                ),
                count = defender_issues
            )
            .blue()
        );
        println!(
            "{}",
            tr(
                "Security is more important than performance in most cases.",
                "大多数情况下，安全性比性能更重要。"
            )
            .yellow()
        );
    } else {
        println!(
            "{}",
            tr("No problems identified!", "未发现任何问题。").green()
        );
    }

    Ok(())
}

fn print_result(result: &CheckupResult) {
    if result.passed {
        println!("{} {}", "[✓]".green(), result.message.clone().green());
    } else {
        println!("{} {}", "[!]".yellow(), result.message.clone().yellow());
        if let Some(fix) = &result.fix_hint {
            println!("  {}", fix.cyan());
        }
    }
}

fn check_main_bucket(global: bool) -> anyhow::Result<CheckupResult> {
    let scoop_home = if global {
        init_scoop_global()
    } else {
        init_user_scoop()
    };
    let main_bucket_path = format!("{}\\buckets\\main", scoop_home);

    if !Path::new(&main_bucket_path).exists() {
        Ok(CheckupResult {
            passed: false,
            message: localized("Main bucket is not added", "尚未添加 main 桶"),
            fix_hint: Some(localized(
                "Run: hp bucket add main",
                "运行: hp bucket add main",
            )),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized("Main bucket is installed", "Main 桶已安装"),
            fix_hint: None,
        })
    }
}

fn check_long_paths() -> anyhow::Result<CheckupResult> {
    let mut os_info = OSVERSIONINFOW {
        dwOSVersionInfoSize: size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };

    unsafe {
        let status = RtlGetVersion(&mut os_info);
        if status.0 != 0 {
            // NTSTATUS 成功时为0
            return Err(CheckupError::WindowsError.into());
        }
    }

    if os_info.dwPlatformId != VER_PLATFORM_WIN32_NT.0
        || os_info.dwMajorVersion < 10
        || (os_info.dwMajorVersion == 10 && os_info.dwBuildNumber < 1607)
    {
        return Ok(CheckupResult {
            passed: false,
            message: localized(
                "This version of Windows does not support configuring LongPaths.",
                "当前 Windows 版本不支持配置 LongPaths。",
            ),
            fix_hint: None,
        });
    }
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let fs_key = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\FileSystem")?;
    let long_paths_enabled: u32 = fs_key.get_value("LongPathsEnabled").unwrap_or(0);

    if long_paths_enabled == 0 {
        Ok(CheckupResult {
            passed: false,
            message: localized(
                "LongPaths support is not enabled",
                "LongPaths 支持尚未启用"
            ),
            fix_hint: Some(localized(
                "Run: Set-ItemProperty 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\FileSystem' -Name 'LongPathsEnabled' -Value 1",
                "运行: Set-ItemProperty 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\FileSystem' -Name 'LongPathsEnabled' -Value 1"
            )),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized("LongPaths support is enabled", "LongPaths 支持已启用"),
            fix_hint: None,
        })
    }
}

fn check_developer_mode() -> anyhow::Result<CheckupResult> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let dev_key =
        hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock")?;
    let dev_mode_enabled: u32 = dev_key
        .get_value("AllowDevelopmentWithoutDevLicense")
        .unwrap_or(0);

    if dev_mode_enabled == 0 {
        Ok(CheckupResult {
            passed: false,
            message: localized(
                "Windows Developer Mode is not enabled",
                "Windows 开发者模式未启用",
            ),
            fix_hint: Some(localized(
                "Enable Developer Mode in Settings > Update & Security > For developers",
                "在 设置 > 更新和安全 > 开发人员 中启用开发者模式",
            )),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized(
                "Windows Developer Mode is enabled",
                "Windows 开发者模式已启用",
            ),
            fix_hint: None,
        })
    }
}

fn check_7zip() -> anyhow::Result<CheckupResult> {
    if which("7z").is_err() {
        Ok(CheckupResult {
            passed: false,
            message: localized(
                "'7-Zip' is not installed. It's required for unpacking most programs.",
                "'7-Zip' 未安装，很多程序的解压都需要它。",
            ),
            fix_hint: Some(localized("Run: hp install 7zip", "运行: hp install 7zip")),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized("7-Zip is installed", "7-Zip 已安装"),
            fix_hint: None,
        })
    }
}

fn check_aria2c_path() -> anyhow::Result<CheckupResult> {
    if which("aria2c").is_err() {
        Ok(CheckupResult {
            passed: false,
            message: localized(
                "'aria2c' is not installed. It's required for downloading some programs.",
                "'aria2c' 未安装，部分程序下载需要它。",
            ),
            fix_hint: Some(localized("Run: hp install aria2", "运行: hp install aria2")),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized("aria2c is installed", "aria2c 已安装"),
            fix_hint: None,
        })
    }
}

fn check_lessmsi() -> anyhow::Result<CheckupResult> {
    if which("lessmsi").is_err() {
        Ok(CheckupResult {
            passed: false,
            message: localized(
                "'lessmsi' is not installed. It's required for unpacking some programs.",
                "'lessmsi' 未安装，部分程序解压需要它。",
            ),
            fix_hint: Some(localized(
                "Run: hp install lessmsi",
                "运行: hp install lessmsi",
            )),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized("lessmsi is installed", "lessmsi 已安装"),
            fix_hint: None,
        })
    }
}

fn check_innounp() -> anyhow::Result<CheckupResult> {
    if which("innounp").is_err() {
        Ok(CheckupResult {
            passed: false,
            message: localized(
                "'innounp' is not installed. It's required for unpacking some programs.",
                "'innounp' 未安装，部分程序解压需要它。",
            ),
            fix_hint: Some(localized(
                "Run: hp install innounp",
                "运行: hp install innounp",
            )),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized("innounp is installed", "innounp 已安装"),
            fix_hint: None,
        })
    }
}

async fn check_github() -> anyhow::Result<CheckupResult> {
    let client = reqwest::Client::new();
    let response = client
        .head("https://github.com")
        .send()
        .await
        .context("Failed to get response from GitHub at 308")?;

    if response.status().is_success() {
        Ok(CheckupResult {
            passed: true,
            message: localized("GitHub is accessible", "GitHub 可正常访问"),
            fix_hint: None,
        })
    } else {
        Ok(CheckupResult {
            passed: false,
            message: localized("GitHub is not accessible", "GitHub 无法访问"),
            fix_hint: Some(localized(
                "Check your internet connection",
                "请检查网络连接",
            )),
        })
    }
}

fn check_git() -> anyhow::Result<CheckupResult> {
    if which("git").is_err() {
        Ok(CheckupResult {
            passed: false,
            message: localized(
                "'git' is not installed. It's required for installing some programs.",
                "'git' 未安装，某些程序的安装需要它。",
            ),
            fix_hint: Some(localized(
                "Download and install git from https://git-scm.com/download/win",
                "请从 https://git-scm.com/download/win 下载并安装 git",
            )),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized("git is installed", "git 已安装"),
            fix_hint: None,
        })
    }
}

fn check_curl_path() -> anyhow::Result<CheckupResult> {
    if which("curl").is_err() {
        Ok(CheckupResult {
            passed: false,
            message: localized(
                "'curl' is not installed. It's required for downloading some programs.",
                "'curl' 未安装，部分程序下载需要它。",
            ),
            fix_hint: Some(localized("Run: hp install curl", "运行: hp install curl")),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized("curl is installed", "curl 已安装"),
            fix_hint: None,
        })
    }
}

fn check_dark() -> anyhow::Result<CheckupResult> {
    if which("dark").is_err() {
        Ok(CheckupResult {
            passed: false,
            message: localized(
                "'dark' is not installed. It's required for some programs.",
                "'dark' 未安装，部分程序需要它。",
            ),
            fix_hint: Some(localized("Run: hp install dark", "运行: hp install dark")),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized("dark is installed", "dark 已安装"),
            fix_hint: None,
        })
    }
}

use windows::Win32::System::SecurityCenter::{
    WscGetSecurityProviderHealth, WSC_SECURITY_PROVIDER_ANTIVIRUS, WSC_SECURITY_PROVIDER_HEALTH,
};

fn check_antivirus() -> anyhow::Result<CheckupResult> {
    let health = unsafe {
        let mut health = WSC_SECURITY_PROVIDER_HEALTH::default();
        WscGetSecurityProviderHealth(WSC_SECURITY_PROVIDER_ANTIVIRUS.0 as u32, &mut health)
    };

    match health {
        Ok(_) => Ok(CheckupResult {
            passed: true,
            message: localized("Antivirus is healthy", "杀毒软件状态正常"),
            fix_hint: None,
        }),
        Err(e) => {
            eprintln!("{}", format!(tr("Error: {}", "错误: {}"), e.to_string()));
            Err(CheckupError::ServiceError.into())
        }
    }
}

fn check_common_antivirus() -> anyhow::Result<CheckupResult> {
    let output = Command::new("tasklist")
        .output()
        .expect("Failed to execute tasklist");

    let tasklist = String::from_utf8_lossy(&output.stdout);

    // 常见杀毒软件进程列表
    let av_processes = [
        "bdagent.exe",    // Bitdefender
        "avguard.exe",    // Avira
        "egui.exe",       // ESET
        "mcshield.exe",   // McAfee
        "hipsdaemon.exe", // Trend Micro
        "360sd.exe",      // 360杀毒
        "360Safe.exe",    // 360安全卫士
        "MsMpEng.exe",    // Windows Defender
        "avp.exe",        // 卡巴斯基
        "QQPCTray.exe",   // 腾讯电脑管家
        "McAfee.exe",     // McAfee
        "AvastUI.exe",    // Avast
        "wsctrlsvc.exe",  // huorong
        "HipsDaemon.exe", // huorong
        "HipsTray.exe",   // huorong
    ];

    let mut issues = Vec::new();

    for process in av_processes {
        if tasklist.contains(process) {
            issues.push(format!(
                tr("{name} is running", "{name} 正在运行"),
                name = process
            ));
        }
    }

    if !issues.is_empty() {
        Ok(CheckupResult {
            passed: false,
            message: issues.join("\n"),
            fix_hint: Some(localized(
                "Please stop the antivirus and try again",
                "请暂时关闭杀毒软件后重试",
            )),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized("Antivirus is not running", "杀毒软件未运行"),
            fix_hint: None,
        })
    }
}
fn check_ntfs_volumes() -> anyhow::Result<CheckupResult> {
    let scoop_path = env::var("SCOOP").unwrap_or_else(|_| {
        let user_profile = env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users".to_string());
        format!("{}\\scoop", user_profile)
    });
    let global_path = env::var("SCOOP_GLOBAL").unwrap_or_else(|_| {
        let app_data = env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".to_string());
        format!("{}\\scoop", app_data)
    });

    let scoop_drive = Path::new(&scoop_path)
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("");
    let global_drive = Path::new(&global_path)
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("");

    let mut issues = Vec::new();

    if !is_ntfs(scoop_drive) {
        issues.push(format!(
            tr(
                "hp requires an NTFS volume to work! Current path: {path}",
                "hp 需要安装在 NTFS 分区上！当前路径: {path}"
            ),
            path = scoop_path
        ));
    }

    if !is_ntfs(global_drive) {
        issues.push(format!(
            tr(
                "hp global requires an NTFS volume to work! Current path: {path}",
                "hp global 需要安装在 NTFS 分区上！当前路径: {path}"
            ),
            path = global_path
        ));
    }

    if !issues.is_empty() {
        Ok(CheckupResult {
            passed: false,
            message: issues.join("\n"),
            fix_hint: Some(localized(
                "Move hp installation to an NTFS formatted drive",
                "请将 hp 安装目录迁移到 NTFS 分区",
            )),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: localized(
                "hp directories are on NTFS volumes",
                "hp 目录位于 NTFS 分区",
            ),
            fix_hint: None,
        })
    }
}

fn is_ntfs(drive: &str) -> bool {
    if drive.is_empty() {
        return false;
    }

    let root = format!(r"{}:\\", drive.chars().next().unwrap());
    let wide_path: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut fs_name_buffer = [0u16; 32];
        let success = GetVolumeInformationW(
            PCWSTR(wide_path.as_ptr()), // lpRootPathName
            None,                       // lpVolumeNameBuffer
            None,                       // lpVolumeSerialNumber
            None,                       // lpMaximumComponentLength
            None,                       // lpFileSystemFlags
            Some(&mut fs_name_buffer),  // lpFileSystemNameBuffer
        );

        if success.is_err() {
            let fs_name = String::from_utf16_lossy(
                &fs_name_buffer[..fs_name_buffer.iter().position(|&x| x == 0).unwrap_or(0)],
            )
            .to_ascii_uppercase();
            fs_name == "NTFS"
        } else {
            false
        }
    }
}
