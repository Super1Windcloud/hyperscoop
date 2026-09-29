use clap::ValueEnum;
use std::env;
use std::sync::atomic::{AtomicU8, Ordering};
#[cfg(windows)]
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

const LANG_UNSET: u8 = 0;
const LANG_EN: u8 = 1;
const LANG_ZH: u8 = 2;

static CURRENT_LANG: AtomicU8 = AtomicU8::new(LANG_UNSET);

pub fn set_language(lang: Language) {
    let val = match lang {
        Language::English => LANG_EN,
        Language::Chinese => LANG_ZH,
    };
    CURRENT_LANG.store(val, Ordering::SeqCst);
}

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
    set_language(lang);
    lang
}

pub fn current_language() -> Language {
    match CURRENT_LANG.load(Ordering::SeqCst) {
        LANG_EN => Language::English,
        LANG_ZH => Language::Chinese,
        _ => {
            let choice = detect_language_choice_from_args();
            init_language(choice)
        }
    }
}

pub fn tr<'a>(en: &'a str, zh: &'a str) -> &'a str {
    match current_language() {
        Language::English => en,
        Language::Chinese => zh,
    }
}

#[macro_export]
macro_rules! tr_fmt {
    ($en:literal, $zh:literal $(, $($arg:tt)+)?) => {
        match $crate::i18n::current_language() {
            $crate::i18n::Language::English => format!($en $(, $($arg)+)?),
            $crate::i18n::Language::Chinese => format!($zh $(, $($arg)+)?),
        }
    };
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

#[cfg(windows)]
pub fn get_system_locale() -> String {
    unsafe {
        let mut buffer = [0u16; 85];
        let len = GetUserDefaultLocaleName(&mut buffer);
        if len > 0 {
            String::from_utf16_lossy(&buffer[..(len as usize - 1)])
        } else {
            "en-US".to_string()
        }
    }
}

#[cfg(not(windows))]
pub fn get_system_locale() -> String {
    env::var("LC_ALL")
        .or_else(|_| env::var("LC_MESSAGES"))
        .or_else(|_| env::var("LANG"))
        .unwrap_or_else(|_| "en-US".to_string())
}

pub fn is_chinese_locale() -> bool {
    let loc = get_system_locale().to_lowercase();
    loc.starts_with("zh")
}
