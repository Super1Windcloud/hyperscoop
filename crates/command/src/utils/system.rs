use anyhow::bail;
use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
 
pub fn delete_env_var(var_key: &str) -> Result<(), anyhow::Error> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment_key = hkcu.open_subkey_with_flags("Environment", KEY_ALL_ACCESS)?;
    if environment_key.get_value::<String, &str>(var_key).is_ok() {
        environment_key.delete_value(var_key)?;
        return Ok(());
    }
    bail!("Environment variable not  exists");
}

pub fn set_env_var(var_key: &str, var_value: &str) -> Result<(), anyhow::Error> {
    use winreg::enums::*;
    use winreg::RegKey;
    if var_key.is_empty() || var_value.is_empty() {
        bail!("Environment variable  can't be empty ");
    }
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment_key = hkcu.open_subkey_with_flags("Environment", KEY_ALL_ACCESS)?;
    environment_key.set_value(var_key, &var_value)?;
    Ok(())
}

pub fn is_shortcut(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("lnk")
}

pub fn get_system_current_time() -> Result<String, anyhow::Error> {
    use chrono::Local;
    let local_time = Local::now();
    let local_time = local_time.format("%+");
    Ok(local_time.to_string())
}

pub fn get_system_default_arch() -> Result<String, anyhow::Error> {
    if cfg!(target_arch = "x86_64") {
        Ok("64bit".to_string())
    } else if cfg!(target_arch = "x86") {
        Ok("32bit".to_string())
    } else if cfg!(target_arch = "arm") {
        Ok("arm64".to_string())
    } else if cfg!(target_arch = "aarch64") {
        Ok("arm64".to_string())
    } else {
        Ok(String::new())
    }
}

#[cfg(windows)]
pub fn request_admin() {
    let exe_path = std::env::current_exe().expect("无法获取程序路径");
    let exe_path_wide: Vec<u16> = exe_path
        .to_string_lossy()
        .encode_utf16()
        .chain([0])
        .collect();

    unsafe {
        ShellExecuteW(
            Some(HWND(std::ptr::null_mut())),
            PCWSTR("runas\0".encode_utf16().collect::<Vec<_>>().as_ptr()),
            PCWSTR(exe_path_wide.as_ptr()),
            PCWSTR(std::ptr::null()),
            PCWSTR(std::ptr::null()),
            windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD(1),
        );
    }
}

#[cfg(windows)]
pub fn is_admin() -> anyhow::Result<bool> {
    unsafe { Ok(IsUserAnAdmin().as_bool()) }
}
mod test {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    fn test_publish_env_var() {
        set_env_var("super", r"A:\Rust_Project\hyperscoop\target").unwrap()
    }
}
