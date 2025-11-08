use windows::Win32::Globalization::GetUserDefaultLocaleName;

fn main() {
    unsafe {
        let mut buffer = [0u16; 85];

        // 调用 WinAPI 函数
        let len = GetUserDefaultLocaleName(&mut buffer);

        if len > 0 {
            // 去掉 null terminator 并转换为 String
            let locale = String::from_utf16_lossy(&buffer[..(len as usize - 1)]);
            println!("系统语言 (System locale): {}", locale);
        } else {
            println!("无法获取系统语言");
        }
    }
}
