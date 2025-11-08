use clap::ValueEnum;
use once_cell::sync::OnceCell;
pub use rust_i18n::t;
use std::env;
use windows::Win32::Globalization::GetUserDefaultLocaleName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Chinese => "zh",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum LanguageChoice {
    Auto,
    En,
    Zh,
}

static SELECTED_LANGUAGE: OnceCell<Language> = OnceCell::new();

pub fn init_language(choice: LanguageChoice) -> Language {
    let lang = match choice {
        LanguageChoice::Auto => {
            if is_chinese_locale() {
                Language::Chinese
            } else {
                Language::English
            }
        }
        LanguageChoice::En => Language::English,
        LanguageChoice::Zh => Language::Chinese,
    };
    rust_i18n::set_locale(lang.code());
    let _ = SELECTED_LANGUAGE.set(lang);
    lang
}

pub fn current_language() -> Language {
    *SELECTED_LANGUAGE.get().unwrap_or(&Language::English)
}

/// Inspect CLI args before Clap so --lang already affects help text rendering.
pub fn detect_language_choice_from_args() -> LanguageChoice {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--" {
            break;
        }
        match arg.as_str() {
            "--lang" | "-L" => {
                if let Some(value) = args.next() {
                    if let Some(choice) = parse_choice(&value) {
                        return choice;
                    }
                }
            }
            _ => {
                if let Some(value) = arg.strip_prefix("--lang=") {
                    if let Some(choice) = parse_choice(value) {
                        return choice;
                    }
                }
                if let Some(value) = arg.strip_prefix("-L") {
                    if !value.is_empty() {
                        if let Some(choice) = parse_choice(value) {
                            return choice;
                        }
                    }
                }
            }
        }
    }
    LanguageChoice::Auto
}

fn parse_choice(raw: &str) -> Option<LanguageChoice> {
    LanguageChoice::from_str(raw, true).ok()
}

pub fn tr<'a>(en: &'a str, zh: &'a str) -> &'a str {
    match current_language() {
        Language::English => en,
        Language::Chinese => zh,
    }
}

pub fn get_system_locale() -> String {
    unsafe {
        let mut buffer = [0u16; 85];

        let len = GetUserDefaultLocaleName(&mut buffer);

        if len > 0 {
            // 去掉 null terminator 并转换为 String
            let locale = String::from_utf16_lossy(&buffer[..(len as usize - 1)]);
            //zh-CN
            locale
        } else {
            panic!("无法获取系统语言");
        }
    }
}

pub fn is_chinese_locale() -> bool {
    if get_system_locale() == "zh-CN" {
        true
    } else {
        false
    }
}
