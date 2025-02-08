use std::error::Error;
use anyhow::bail;
use widestring::WideCString;
use windows_sys::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG, MessageBoxW, MB_OK, MB_ICONINFORMATION, CreateWindowExW, ShowWindow};
use windows_sys::Win32::Foundation::{HWND, WPARAM, LPARAM, GetLastError};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

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
  use windows_sys::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage, RegisterClassW, TranslateMessage,
    MSG, WNDCLASSW, WM_DESTROY, WM_SETTINGCHANGE,
  };
  use windows_sys::Win32::Foundation::{ HWND, LPARAM, LRESULT, WPARAM};
  use widestring::WideCString;

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
          "环境变量已更新".encode_utf16().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
          "提示".encode_utf16().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
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
      "环境变量更新".encode_utf16().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
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
  use winreg::RegKey;
  use winreg::enums::*;
  let hkcu = RegKey::predef(HKEY_CURRENT_USER);
  let environment_key = hkcu.open_subkey_with_flags("Environment", KEY_ALL_ACCESS)?;
  if environment_key.get_value::<String, &str>(var_key).is_ok() {
    environment_key.delete_value(var_key)?;
    return  Ok(()) ; 
  }
  bail!("Environment variable not  exists");  
}

pub fn set_env_var(var_key: &str, var_value: &str) -> Result<(), anyhow::Error> {
  use winreg::RegKey;
  use winreg::enums::*; 
  if var_key.is_empty()||var_value.is_empty() {
     bail!("Environment variable  can't be empty ");
  }
  let hkcu = RegKey::predef(HKEY_CURRENT_USER);
  let environment_key = hkcu.open_subkey_with_flags("Environment", KEY_ALL_ACCESS)?;
  environment_key.set_value(var_key, &var_value)?; 
  Ok(() )
}


mod test {
  #[allow(unused_imports)]
  use super::*;
  #[test]
  fn test_publish_env_var() { 
    set_env_var("super" , r"A:\Rust_Project\hyperscoop\target").unwrap()
  }
}