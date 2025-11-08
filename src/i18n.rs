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
        LanguageChoice::Auto => detect_language().unwrap_or(Language::English),
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

fn detect_language() -> Option<Language> {
    const ENV_KEYS: [&str; 6] = [
        "HP_LANG",
        "hp_lang",
        "LANG",
        "LC_ALL",
        "LC_MESSAGES",
        "PreferredUILanguages",
    ];
    ENV_KEYS
        .iter()
        .find_map(|key| env::var(key).ok())
        .and_then(parse_language_hint)
}

fn parse_language_hint(raw: String) -> Option<Language> {
    raw.split([';', ',', ':'])
        .find_map(|segment| parse_tag(segment.trim()))
}

fn parse_tag(tag: &str) -> Option<Language> {
    let lower = tag.to_ascii_lowercase();
    if lower.starts_with("zh")
        || lower.starts_with("cn")
        || lower.contains("chinese")
        || lower.contains("zhong")
    {
        Some(Language::Chinese)
    } else if lower.starts_with("en") || lower.contains("english") {
        Some(Language::English)
    } else {
        None
    }
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
