use anyhow::bail;
use std::error::Error;
use std::path::Path;
use widestring::WideCString;
use windows_sys::Win32::Foundation::{GetLastError, HWND, LPARAM, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, MessageBoxW, SendMessageTimeoutW, ShowWindow, MB_ICONINFORMATION, MB_OK,
    SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

pub fn publish_env_var() -> Result<(), Box<dyn Error>> {
    const HWND_BROADCAST: HWND = 0xFFFF as HWND;

    // 准备lParam参数，转为宽字符串并确保以null结尾
    let lparam_str = WideCString::from_str("Environment")?;
    let lparam_ptr = lparam_str.as_ptr() as LPARAM;

    // fuFlags设置为SMTO_ABORTIFHUNG (0x0002)
    let fu_flags = SMTO_ABORTIFHUNG;
    let u_timeout = 5000;

    // 调用SendMessageTimeoutW
    let result = unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0 as WPARAM,
            lparam_ptr,
            fu_flags,
            u_timeout,
            std::ptr::null_mut(),
        )
    };

    if result == 0 {
        let error_code = unsafe { GetLastError() };
        Err(format!("SendMessageTimeoutW failed with error code: {}", error_code).into())
    } else {
        Ok(())
    }
}
pub fn alert_dialog() -> Result<(), Box<dyn Error>> {
    use widestring::WideCString;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage, RegisterClassW,
        TranslateMessage, MSG, WM_DESTROY, WM_SETTINGCHANGE, WNDCLASSW,
    };

    const CLASS_NAME: &str = "MyWindowClass";

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_SETTINGCHANGE => {
                // 弹出提示框
                MessageBoxW(
                    hwnd,
                    "环境变量已更新"
                        .encode_utf16()
                        .chain(Some(0))
                        .collect::<Vec<_>>()
                        .as_ptr(),
                    "提示"
                        .encode_utf16()
                        .chain(Some(0))
                        .collect::<Vec<_>>()
                        .as_ptr(),
                    MB_OK | MB_ICONINFORMATION,
                );
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());

        let class_name = WideCString::from_str(CLASS_NAME)?;
        let class_name_ptr = class_name.as_ptr();

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name_ptr,
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            0,
            class_name_ptr,
            "环境变量更新"
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>()
                .as_ptr(),
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null_mut(),
        );

        ShowWindow(hwnd, 1);
        use std::mem;
        let mut msg: MSG = mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

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

pub fn update_scoop_config_last_update_time() {
    let current = get_system_current_time().unwrap();
    let config_path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home_dir = std::env::var("USERPROFILE").unwrap();
        format!("{}\\.config\\scoop\\config.json", home_dir)
    });
    let config_path = Path::new(&config_path);
    if config_path.exists() {
      let config_file = std::fs::File::open(config_path).unwrap();
      let mut config_json: serde_json::Value = serde_json::from_reader(config_file).unwrap();
      if let Some(obj) = config_json.as_object_mut() {
        obj.insert("last_update".into() ,  current.into());
      }
      let file = std::fs::File::create(config_path).unwrap();
      serde_json::to_writer_pretty(file, &config_json).unwrap();
    }
}
mod test {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    fn test_publish_env_var() {
        set_env_var("super", r"A:\Rust_Project\hyperscoop\target").unwrap()
    }
}
