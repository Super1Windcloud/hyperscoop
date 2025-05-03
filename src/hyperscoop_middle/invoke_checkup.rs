use color_eyre::owo_colors::OwoColorize;
use command_util_lib::init_env::{init_scoop_global, init_user_scoop};
use crossterm::style::Stylize;
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

pub fn execute_checkup_command(global: bool) -> anyhow::Result<()> {
    let mut total_issues = 0;
    let  defender_issues = 0;

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
    let innounp_result = check_innounp()?;
    if !innounp_result.passed {
        total_issues += 1;
        print_result(&innounp_result);
    }
    let github_result = check_github()?;
    if !github_result.passed {
        total_issues += 1;
        print_result(&github_result);
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

    if total_issues > 0 {
        println!(
            "{}",
            format!("Found {} potential issues.", total_issues).yellow()
        );
    } else if defender_issues > 0 {
        println!(
            "{}",
            format!("Found {} performance issues.", defender_issues).blue()
        );
        println!(
            "{}",
            "Security is more important than performance, in most cases.".yellow()
        );
    } else {
        println!("{}", "No problems identified!".green());
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
            message: "Main bucket is not added".to_string(),
            fix_hint: Some("Run: hp bucket add main".to_string()),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: "Main bucket is installed".to_string(),
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
            message: "This version of Windows does not support configuration of LongPaths"
                .to_string(),
            fix_hint: None,
        });
    }
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let fs_key = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\FileSystem")?;
    let long_paths_enabled: u32 = fs_key.get_value("LongPathsEnabled").unwrap_or(0);

    if long_paths_enabled == 0 {
        Ok(CheckupResult {
      passed: false,
      message: "LongPaths support is not enabled".to_string(),
      fix_hint: Some(
        "Run: Set-ItemProperty 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\FileSystem' -Name 'LongPathsEnabled' -Value 1".to_string(),
      ),
    })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: "LongPaths support is enabled".to_string(),
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
            message: "Windows Developer Mode is not enabled".to_string(),
            fix_hint: Some(
                "Enable Developer Mode in Settings > Update & Security > For developers"
                    .to_string(),
            ),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: "Windows Developer Mode is enabled".to_string(),
            fix_hint: None,
        })
    }
}

fn check_7zip() -> anyhow::Result<CheckupResult> {
    if which("7z").is_err() {
        Ok(CheckupResult {
            passed: false,
            message: "'7-Zip' is not installed! It's required for unpacking most programs"
                .to_string(),
            fix_hint: Some("Run: hp install 7zip".to_string()),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: "7-Zip is installed".to_string(),
            fix_hint: None,
        })
    }
}


fn check_lessmsi() -> anyhow::Result<CheckupResult> {
  if which("lessmsi").is_err() {  
      Ok(CheckupResult {
          passed: false,
          message: "'lessmsi' is not installed! It's required for unpacking some programs"
             .to_string(),
          fix_hint: Some("Run: hp install lessmsi".to_string()),
      })
  } else {
      Ok(CheckupResult {
          passed: true,
          message: "lessmsi is installed".to_string(),
          fix_hint: None,
      })  
  }
}

fn  check_innounp() -> anyhow::Result<CheckupResult> {
  if which("innounp").is_err() {  
      Ok(CheckupResult {
          passed: false,
          message: "'innounp' is not installed! It's required for unpacking some programs"
             .to_string(),
          fix_hint: Some("Run: hp install innounp".to_string()),
      })
  } else {
      Ok(CheckupResult {
          passed: true,
          message: "innounp is installed".to_string(),
          fix_hint: None,
      })  
  }
}

fn  check_github() -> anyhow::Result<CheckupResult> {
  if which("git").is_err() {  
      Ok(CheckupResult {
          passed: false,
          message: "'git' is not installed! It's required for installing some programs"
             .to_string(),
          fix_hint: Some("Download and install git from https://git-scm.com/download/win".to_string()),
      })
  } else {
      Ok(CheckupResult {
          passed: true,
          message: "git is installed".to_string(),
          fix_hint: None,
      })  
  } 
}

fn check_dark() -> anyhow::Result<CheckupResult> {
  if which("dark").is_err() {  
      Ok(CheckupResult {
          passed: false,
          message: "'dark' is not installed! It's required for some programs"
             .to_string(),
          fix_hint: Some("Run: hp install dark".to_string()),
      })
  } else {
      Ok(CheckupResult {
          passed: true,
          message: "dark is installed".to_string(),
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
            "hp requires an NTFS volume to work! Current path: {}",
            scoop_path
        ));
    }

    if !is_ntfs(global_drive) {
        issues.push(format!(
            "hp global requires an NTFS volume to work! Current path: {}",
            global_path
        ));
    }

    if !issues.is_empty() {
        Ok(CheckupResult {
            passed: false,
            message: issues.join("\n"),
            fix_hint: Some("Move hp installation to an NTFS formatted drive".to_string()),
        })
    } else {
        Ok(CheckupResult {
            passed: true,
            message: "hp directories are on NTFS volumes".to_string(),
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
