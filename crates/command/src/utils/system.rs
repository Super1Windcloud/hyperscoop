use crate::init_env::get_persist_dir_path_global;
use anyhow::bail;
use std::path::Path;
use std::process::{Command, Stdio};
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
use winreg::enums::*;
use winreg::RegKey;

pub fn delete_env_var(var_key: &str) -> Result<(), anyhow::Error> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment_key = hkcu.open_subkey_with_flags("Environment", KEY_ALL_ACCESS)?;
    if environment_key.get_value::<String, &str>(var_key).is_ok() {
        environment_key.delete_value(var_key)?;
        return Ok(());
    }
    bail!("Environment variable not  exists");
}
pub fn delete_global_env_var(var_key: &str) -> Result<(), anyhow::Error> {
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

    let environment_key = hkcu.open_subkey_with_flags(key, KEY_ALL_ACCESS)?;
    if environment_key.get_value::<String, &str>(var_key).is_ok() {
        environment_key.delete_value(var_key)?;
        return Ok(());
    }
    bail!("Environment variable not  exists");
}

pub  fn  get_user_env_var(var_key: &str) -> Result<String , anyhow::Error> {
  let hkcu = RegKey::predef(HKEY_CURRENT_USER);
  let environment = hkcu.open_subkey("Environment")?;
  let value :String  = environment.get_value(var_key)?;
   Ok(value)
}

pub fn get_system_env_var(var_key: &str) -> Result<String, anyhow::Error> {
  let  hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
  let key = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
  let environment_key = hkcu.open_subkey_with_flags(key, KEY_ALL_ACCESS)?;
   let value :String  = environment_key.get_value(var_key)?; 
   Ok(value)
}

pub fn set_user_env_var(var_key: &str, var_value: &str) -> Result<(), anyhow::Error> {
    if var_key.is_empty() || var_value.is_empty() {
        bail!("Environment variable  can't be empty ");
    }
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment_key = hkcu.open_subkey_with_flags("Environment", KEY_ALL_ACCESS)?;
    environment_key.set_value(var_key, &var_value)?;

    Ok(())
}
pub fn set_global_env_var(var_key: &str, var_value: &str) -> Result<(), anyhow::Error> {
    if var_key.is_empty() || var_value.is_empty() {
        bail!("Environment variable  can't be empty ");
    }
    if !is_admin()? {
        request_admin();
    }
    let powershell_command = format!(
        "[System.Environment]::SetEnvironmentVariable(\"{}\", \"{}\", \"Machine\")",
        var_key, var_value
    );
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(powershell_command)
        .stdin(Stdio::null()) // 防止出现输入提示
        .output()?;

    if output.status.success() {
        println!("Successfully added system environment variable.");
    } else {
        eprintln!("Failed to add system environment variable.");
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        bail!("Failed to add system environment variable.");
    }
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

pub fn get_system_env_path() -> Vec<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let environment_key = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
    let env_key = hklm.open_subkey(environment_key).unwrap();
    let result: String = env_key
        .get_value("PATH")
        .expect("Failed to get system path");
    result
        .split(';')
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}
pub fn get_system_env_str() -> String {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let environment_key = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
    let env_key = hklm.open_subkey(environment_key).unwrap();
    env_key
        .get_value("PATH")
        .expect("Failed to get system path")
}
pub fn get_user_env_path() -> Vec<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment_key = hkcu
        .open_subkey("Environment")
        .expect("Failed to open Environment key");
    let user_path: String = environment_key
        .get_value("PATH")
        .expect("Failed to get user path");

    user_path
        .split(';')
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}

pub fn get_user_env_str() -> String {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment_key = hkcu
        .open_subkey("Environment")
        .expect("Failed to open Environment key");
    environment_key
        .get_value("PATH")
        .expect("Failed to get user path")
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

pub fn compute_hash_by_powershell(file_path: &str, algorithm: &str) -> anyhow::Result<String> {
    let cmd = format!(
        r#"$env:PSModulePath ="$PSHOME/Modules";(Get-FileHash -Algorithm {} -Path "{}").hash"#,
        algorithm, file_path
    );
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(cmd)
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        let hash = String::from_utf8_lossy(&output.stdout);
        Ok(hash.trim().to_string())
    } else {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        bail!("powershell.exe  compute cache hash failed")
    }
}

pub fn ensure_persist_permission() -> anyhow::Result<()> {
    if is_admin()? {
        let persist_dir = get_persist_dir_path_global();
        let cmd = format!(
            r"
        $path={persist_dir};
        $user = New-Object System.Security.Principal.SecurityIdentifier 'S-1-5-32-545';
        $target_rule = New-Object System.Security.AccessControl.FileSystemAccessRule($user, 'Write', 'ObjectInherit', 'none', 'Allow');
        $acl = Get-Acl -Path $path;
        $acl.SetAccessRule($target_rule);
        $acl | Set-Acl -Path $path;
       "
        );
        let output = Command::new(&cmd).output()?;
        if !output.status.success() {
            bail!("Error : {}", String::from_utf8_lossy(&output.stderr));
        }
    }
    Ok(())
}

mod test_system {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    fn test_publish_env_var() {
        // set_user_env_var("super", r"A:\Rust_Project\hyperscoop\target").unwrap()
        set_global_env_var("AAAFUCK", r"A:\Rust_Project\hyperscoop\target").unwrap()
    }

    #[test]
    fn test_hash() {
        println!(
            "{}",
            compute_hash_by_powershell(r"A:\Scoop\cache\yazi#25.4.8#1319a47.zip", "sha256")
                .unwrap()
        );
    }
}
