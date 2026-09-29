#![allow(unsafe_code)]

#[cfg(not(windows))]
fn main() {
    eprintln!("shim is only supported on Windows.");
}

#[cfg(windows)]
fn main() -> color_eyre::Result<()> {
    windows_shim::run()
}

#[cfg(windows)]
mod windows_shim {
    use std::ffi::OsString;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::os::windows::prelude::*;
    use std::path::PathBuf;
    use std::process::Command;
    use windows::Win32::Foundation::*;
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::System::Console::*;
    use windows::Win32::System::JobObjects::*;
    use windows::Win32::System::LibraryLoader::*;
    use windows::Win32::UI::Shell::{
        PathUnquoteSpacesW, SEE_MASK_NOCLOSEPROCESS, SHFILEINFOW, SHGFI_EXETYPE, SHGetFileInfoW,
    };
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
    use windows::Win32::{
        Foundation::HANDLE,
        UI::Shell::{SHELLEXECUTEINFOW, ShellExecuteExW},
    };
    use windows::core::{BOOL, PCWSTR, PWSTR};

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
        if let Some(arg_str) = args
            && arg_str.contains("%~dp0")
        {
            *arg_str = arg_str.replace("%~dp0", cur_dir);
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
            for line in reader.lines().map_while(Result::ok) {
                if let Some(value) = line.strip_prefix("path = ") {
                    path = Some(value.trim().to_string());
                } else if let Some(value) = line.strip_prefix("args = ") {
                    args = Some(value.trim().to_string());
                }
            }
        }

        let cur_dir = get_directory(&exe_str);
        normalize_args(&mut args, &cur_dir);

        Ok(ShimInfo { path, args })
    }

    fn is_elevation_required(error: &std::io::Error) -> bool {
        error.raw_os_error() == Some(740) // ERROR_ELEVATION_REQUIRED
    }

    fn remove_extra_quotes(value: &str) -> String {
        value.trim_matches(|c| c == '\'' || c == '"').to_string()
    }

    pub(crate) fn parse_args_string(s: &str) -> Vec<OsString> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = '"';
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    if let Some(&next) = chars.peek() {
                        if next == '"' || next == '\'' || next == '\\' {
                            current.push(next);
                            chars.next();
                            continue;
                        }
                    }
                    current.push('\\');
                }
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = c;
                }
                c if in_quotes && c == quote_char => {
                    in_quotes = false;
                }
                c if c.is_whitespace() && !in_quotes => {
                    if !current.is_empty() {
                        args.push(OsString::from(std::mem::take(&mut current)));
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }
        if !current.is_empty() {
            args.push(OsString::from(current));
        }
        args
    }

    pub(crate) fn format_args_for_shellexecute(args: &[OsString]) -> String {
        let mut result = String::new();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            let s = arg.to_string_lossy();
            if s.contains(' ') || s.contains('\t') || s.contains('"') {
                result.push('"');
                for c in s.chars() {
                    if c == '"' {
                        result.push('\\');
                    }
                    result.push(c);
                }
                result.push('"');
            } else {
                result.push_str(&s);
            }
        }
        result
    }

    fn make_process(path: &str, all_args: &[OsString]) -> Option<std::process::Child> {
        let clean_path = remove_extra_quotes(path);
        let process = Command::new(&clean_path).args(all_args).spawn();
        match process {
            Ok(child) => Some(child),
            Err(e) => {
                eprintln!("Error starting process: {}. Trying as admin...", e);
                if is_elevation_required(&e) {
                    if elevate_process(&clean_path, all_args) {
                        None
                    } else {
                        eprintln!("Failed to start process as administrator.");
                        None
                    }
                } else {
                    eprintln!("Failed to start process: {:?}", e);
                    None
                }
            }
        }
    }

    /// *以管理员权限启动进程*
    fn elevate_process(exe_path: &str, args: &[OsString]) -> bool {
        let params_str = format_args_for_shellexecute(args);
        let path_wide: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();
        let args_wide: Vec<u16> = params_str
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut sei = SHELLEXECUTEINFOW {
            cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            hwnd: Default::default(),
            lpVerb: Default::default(),
            lpFile: PCWSTR(path_wide.as_ptr()),
            lpParameters: if args_wide.len() <= 1 {
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
        if let Ok(_pi) = pi {
            true
        } else {
            eprintln!("Failed to create elevated process.");
            false
        }
    }

    fn create_job_object() -> Option<HANDLE> {
        let job = unsafe { CreateJobObjectW(None, None) }.ok()?;
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
        if let Err(err) = result {
            eprintln!("Error setting job object limit: {:?}", err);
        }
        Some(job)
    }

    fn set_console_ctrl_handler() {
        unsafe {
            let _ = SetConsoleCtrlHandler(Some(ctrl_handler), TRUE.into());
        }
    }

    /// # Safety
    ///
    /// Windows invokes this callback from the console control handler context.
    /// The implementation does not dereference pointers or touch shared Rust state.
    pub unsafe extern "system" fn ctrl_handler(_ctrl_type: u32) -> BOOL {
        TRUE // 忽略所有 Ctrl+C 等信号，交由子进程处理
    }

    fn is_windows_gui_app(exe_path: &str) -> bool {
        let mut wide_path: Vec<u16> = OsString::from(exe_path).encode_wide().collect();
        wide_path.push(0); // Null 终止符
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
                size_of::<SHFILEINFOW>() as u32,
                SHGFI_EXETYPE,
            )
        };
        ret != 0 && ret & 0xFFFF_0000 != 0
    }

    pub fn run() -> color_eyre::Result<()> {
        let shim_info = get_shim_info()?;

        let Some(path) = shim_info.path else {
            eprintln!("Error: Could not read shim file.");
            std::process::exit(1);
        };

        // 收集参数：先组合 shim 配置的预设参数，再加上当前调用的命令行参数（完整保留空格与引号）
        let mut all_args: Vec<OsString> = Vec::new();
        if let Some(configured_args) = shim_info.args {
            let clean_args = remove_extra_quotes(&configured_args);
            if !clean_args.is_empty() {
                all_args.extend(parse_args_string(&clean_args));
            }
        }
        // 直接使用系统原生切片，绝不进行 join(" ") 序列化与空格二次分割
        all_args.extend(std::env::args_os().skip(1));

        let is_gui = is_windows_gui_app(&path);
        if is_gui {
            unsafe { FreeConsole() }?; // GUI 进程，释放控制台
        }

        set_console_ctrl_handler();
        let job = create_job_object();

        if let Some(mut child) = make_process(&path, &all_args) {
            if let Some(job) = job {
                unsafe {
                    let _ = AssignProcessToJobObject(job, HANDLE(child.as_raw_handle()));
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
    fn test_parse_args_string() {
        let parsed = parse_args_string("-m \"commit message with spaces\" --flag 'another value'");
        let strings: Vec<String> = parsed
            .into_iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();
        assert_eq!(
            strings,
            vec![
                "-m",
                "commit message with spaces",
                "--flag",
                "another value"
            ]
        );
    }

    #[test]
    fn test_format_args_for_shellexecute() {
        let args = vec![
            OsString::from("commit"),
            OsString::from("-m"),
            OsString::from("hello world"),
        ];
        let formatted = format_args_for_shellexecute(&args);
        assert_eq!(formatted, "commit -m \"hello world\"");
    }
}
