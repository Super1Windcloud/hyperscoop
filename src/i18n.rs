#![allow(unused_imports)]

pub use command_util_lib::i18n::{
    Language, LanguageChoice, current_language, detect_language_choice_from_args,
    get_system_locale, is_chinese_locale, set_language, tr,
};
pub use command_util_lib::tr_fmt;
pub use rust_i18n::t;

pub fn init_language(choice: LanguageChoice) -> Language {
    let lang = command_util_lib::i18n::init_language(choice);
    rust_i18n::set_locale(lang.code());
    lang
}
