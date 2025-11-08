use std::env;
use windows::Win32::Globalization::GetUserDefaultLocaleName;

fn get_gitee_env_path() -> String {
    let cwd = env::current_dir().unwrap();
    let env = cwd.join(".env");
    if !env.exists() {
        eprintln!("{}", "当前目录下不存在.env".to_string());
        return "".to_string();
    }
    let env = std::fs::read_to_string(&env).unwrap();
    env.trim().to_string()
}

fn get_github_token_path() -> String {
    let cwd = env::current_dir().unwrap();
    let env = cwd.join(".github_token");
    if !env.exists() {
        eprintln!("{}", "当前目录下不存在.github_token".to_string());
        return "".to_string();
    }
    let env = std::fs::read_to_string(&env).unwrap();
    env.trim().to_string()
}

fn get_system_locale() -> String {
    unsafe {
        let mut buffer = [0u16; 85];

        let len = GetUserDefaultLocaleName(&mut buffer);

        if len > 0 {
            // 去掉 null terminator 并转换为 String
            let locale = String::from_utf16_lossy(&buffer[..(len as usize - 1)]);
            return locale;
        } else {
            panic!("无法获取系统语言");
        }
    }
}

fn main() {
    if !cfg!(target_os = "windows") {
        panic!("This crate can only be built on Windows.");
    }

    let lang = get_system_locale();
    println!("lang {}", lang);

    let gitee_env = get_gitee_env_path();
    let github_token = get_github_token_path();

    println!("cargo:rustc-env=BUILD_SYSTEM_LANG={}", lang);
    println!("cargo:rustc-env=GITEE_TOKEN={}", gitee_env);
    println!("cargo:rustc-env=GITHUB_TOKEN={}", github_token);
    if !github_token.is_empty() && !gitee_env.is_empty() {
        println!("cargo:rustc-cfg=token_local");
    } else {
        println!("cargo:rustc-cfg=token_cloud");
    }
    if lang == "zh-CN" {
        println!("cargo:rustc-cfg=system_lang_zh");
    }
}
