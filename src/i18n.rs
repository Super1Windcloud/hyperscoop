use clap::ValueEnum;
use once_cell::sync::OnceCell;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
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
    let _ = SELECTED_LANGUAGE.set(lang);
    lang
}

pub fn current_language() -> Language {
    *SELECTED_LANGUAGE.get().unwrap_or(&Language::English)
}

pub fn tr<'a>(en: &'a str, zh: &'a str) -> &'a str {
    match current_language() {
        Language::English => en,
        Language::Chinese => zh,
    }
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

#[macro_export]
macro_rules! hp_bilingual {
    ($en:literal, $zh:literal) => {
        concat!($en, " / ", $zh)
    };
}
