use std::ffi::{ OsString};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::windows::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use windows::core::{s, BOOL, PCWSTR, PWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::System::Console::*;
use windows::Win32::System::JobObjects::CreateJobObjectW;
use windows::Win32::System::JobObjects::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::Shell::{
    PathUnquoteSpacesW, SHGetFileInfoW, ShellExecuteW, SEE_MASK_NOCLOSEPROCESS, SHFILEINFOW,
    SHGFI_EXETYPE,
};

use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
use windows::Win32::{
    Foundation::{HANDLE },
    UI::Shell::{ShellExecuteExW, SHELLEXECUTEINFOW},
};

type WStringOpt = Option<String>;

#[derive(Debug)]
struct ShimInfo {
    pub path: WStringOpt,
    pub args: WStringOpt,
}

fn get_directory(exe_path: &str) -> String {
    let path = PathBuf::from(exe_path);
    path.parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string())
}

fn normalize_args(args: &mut WStringOpt, cur_dir: &str) {
    if let Some(arg_str) = args {
        if arg_str.contains("%~dp0") {
            *arg_str = arg_str.replace("%~dp0", cur_dir);
        }
    }
}

fn get_shim_info() -> color_eyre::Result<ShimInfo> {
    let mut exe_path = vec![0u16; MAX_PATH as usize];
    let exe_len = unsafe { GetModuleFileNameW(None, &mut exe_path) } as usize;
    if exe_len == 0 {
        eprintln!("Error: Unable to retrieve module file name.");
        return Ok(ShimInfo {
            path: None,
            args: None,
        });
    }

    let exe_str = String::from_utf16_lossy(&exe_path[..exe_len]);
    let mut shim_file_path = exe_str.clone();
    shim_file_path.truncate(shim_file_path.len() - 3);
    shim_file_path.push_str("shim");

    let file = File::open(&shim_file_path).ok();
    let reader = file.map(BufReader::new);

    let mut path: WStringOpt = None;
    let mut args: WStringOpt = None;

    if let Some(reader) = reader {
        for line in reader.lines().flatten() {
            if line.starts_with("path = ") {
                path = Some(line[7..].trim().to_string());
            } else if line.starts_with("args = ") {
                args = Some(line[7..].trim().to_string());
            }
        }
    }

    let cur_dir = get_directory(&exe_str);
    normalize_args(&mut args, &cur_dir);

    Ok(ShimInfo { path, args })
}

fn is_elevation_required(error: &std::io::Error) -> bool {
    #[cfg(windows)]
    {
        error.raw_os_error() == Some(740) // ERROR_ELEVATION_REQUIRED
    }
    #[cfg(not(windows))]
    {
        false
    }
}


fn  remove_extra_quotes ( str: &str ) -> String {
  str .trim_matches(|c| c == '\'' || c == '"').to_string()
}
fn make_process(info: &ShimInfo) -> Option<std::process::Child> {
    let path = info.path.as_ref()?; 
   let  path = remove_extra_quotes(path);
    let args = info.args.as_ref()?.to_string(); 
   let  args = remove_extra_quotes(&args); 
    let  args_split = args.split_whitespace().collect::<Vec<_>>().join(" ");
    let process = Command::new(&path).args(args.split_whitespace()).spawn();
    match process {
        Ok(child) => Some(child),
        Err(e) => {
            eprintln!("Error starting process: {}. Trying as admin...", e);
            //  **尝试使用管理员权限启动**
            if is_elevation_required(&e) {
                if elevate_process(&path, &args) {
                    None // 进程已提权启动，不返回 `Child`
                } else {
                    eprintln!("Failed to start process as administrator.");
                    None
                }
            } else {
                eprintln!("Failed to start process. {:?}", e);
                None
            }
        }
    }
}

/// *以管理员权限启动进程*
fn elevate_process(exe_path: &str, params: &str) -> bool {
    let path_wide: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();
    let args_wide: Vec<u16> = params.encode_utf16().chain(std::iter::once(0)).collect();

    // 初始化 SHELLEXECUTEINFOW 结构体
    let mut sei = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: Default::default(),
        lpVerb: Default::default(),
        lpFile: PCWSTR(path_wide.as_ptr()),
        lpParameters: if args_wide.is_empty() {
            PCWSTR::null()
        } else {
            PCWSTR(args_wide.as_ptr())
        },
        lpDirectory: Default::default(),
        nShow: SW_SHOW.0,
        hInstApp: Default::default(),
        lpIDList: std::ptr::null_mut(),
        lpClass: Default::default(),
        hkeyClass: Default::default(),
        dwHotKey: 0,
        Anonymous: Default::default(),
        hProcess: Default::default(),
    };

    let pi = unsafe {
        let result = ShellExecuteExW(&mut sei);
        if result.is_err() {
            let error = GetLastError();
            eprintln!("Failed to create elevated process: error {}", error.0);
            Err(color_eyre::eyre::eyre!(
                "Failed to create elevated process: error {}",
                error.0
            ))
        } else {
            Ok(sei.hProcess)
        }
    };
    if let Ok(pi) = pi {
        true
    } else {
        eprintln!("Failed to create elevated process.");
        false
    }
}

fn create_job_object() -> Option<HANDLE> {
    let job = unsafe { CreateJobObjectW(None, None) }.unwrap();
    if job.is_invalid() {
        return None;
    }

    let mut jeli = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    jeli.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK;

    let result = unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &jeli as *const _ as *const _,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };
    if result.is_err() {
        println!("Error setting job object limit. {:?}", result.unwrap_err());
    }
    Some(job)
}
fn set_console_ctrl_handler() {
    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), TRUE.into()).expect("TODO: panic message");
    }
}

pub unsafe extern "system" fn ctrl_handler(_ctrl_type: u32) -> BOOL {
    TRUE // 忽略所有 Ctrl+C 等信号
}

fn is_windows_gui_app(exe_path: &str) -> bool {
    let mut wide_path: Vec<u16> = OsString::from(exe_path).encode_wide().collect();
    wide_path.push(0); // Null 终止符
                       // 去除路径中的引号
    unsafe {
        let _ = PathUnquoteSpacesW(PWSTR(wide_path.as_mut_ptr()));
    }
    let dw_file_attributes: FILE_FLAGS_AND_ATTRIBUTES = FILE_FLAGS_AND_ATTRIBUTES(u32::MAX);
    let mut sfi = SHFILEINFOW::default();
    let ret = unsafe {
        SHGetFileInfoW(
            PWSTR(wide_path.as_mut_ptr()),
            dw_file_attributes,
            Some(&mut sfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_EXETYPE,
        )
    };
    ret != 0 && ret & 0xFFFF_0000 != 0
}

fn main() -> color_eyre::Result<()> {
    let shim_info = get_shim_info()?;

    if shim_info.path.is_none() {
        eprintln!("Error: Could not read shim file.");
        std::process::exit(1);
    }

    let path = shim_info.path.clone().unwrap();
    let args = shim_info.args.clone().unwrap_or_default();

    // 解析当前命令行参数并追加
    let cmd_line = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let full_args = if args.is_empty() {
        cmd_line
    } else {
        format!("{} {}", args, cmd_line)
    };

    let is_gui = is_windows_gui_app(&path);
    if is_gui {
        unsafe { FreeConsole() }?; // GUI 进程，释放控制台
    }

    set_console_ctrl_handler();
    let job = create_job_object();

    if let Some(mut child) = make_process(&ShimInfo {
        path: Some(path),
        args: Some(full_args),
    }) {
        if let Some(job) = job {
            unsafe {
                AssignProcessToJobObject(job, HANDLE(child.as_raw_handle()))?;
            }
        }

        let status = child.wait().expect("Failed to wait for process");
        match status.code() {
            Some(code) => std::process::exit(code),
            None => std::process::exit(1),
        }
    } else {
        std::process::exit(1);
    }
}



#[test]
fn test_create_process(){ 
  let path = r#""A:\Scoop\apps\zigmod\current\zigmod.exe""#; 
  println!("{}", path);
    let result   =Command::new(path).spawn(); 
      match result {
        Ok(child) => {
            println!("Process started successfully. PID: {}", child.id());
        }
        Err(e) => {
            eprintln!("Failed to start process: {}", e);
        }
      }  
}