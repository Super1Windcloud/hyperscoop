use windows::Win32::Globalization::GetUserDefaultLocaleName;

fn get_system_locale() -> String {
    unsafe {
        let mut buffer = [0u16; 85];

        let len = GetUserDefaultLocaleName(&mut buffer);

        if len > 0 {
            // 去掉 null terminator 并转换为 String
            String::from_utf16_lossy(&buffer[..(len as usize - 1)])
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

    println!("cargo:rustc-env=BUILD_SYSTEM_LANG={}", lang);
    if lang == "zh-CN" {
        println!("cargo:rustc-cfg=system_lang_zh");
    }
}
