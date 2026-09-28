use crate::command_args::prefix::PrefixArgs;
use crate::i18n::tr;
use anyhow::bail;
use command_util_lib::init_env::{get_app_current_dir, get_app_current_dir_global};
use crossterm::style::Stylize;
use std::path::Path;

pub fn execute_prefix_command(prefix: PrefixArgs) -> Result<(), anyhow::Error> {
    let mut errors = Vec::new();
    for name in &prefix.names {
        let app_path = if prefix.global {
            get_app_current_dir_global(name.as_str())
        } else {
            get_app_current_dir(name.as_str())
        };
        if !Path::new(&app_path).exists() {
            let msg = format!(
                "{} {path}",
                tr("{path} does not exist", "{path} 不存在"),
                path = app_path.as_str()
            );
            eprintln!("{}", msg.as_str().red().bold());
            errors.push(msg);
        } else {
            println!("{}", app_path.dark_green().bold());
        }
    }
    if !errors.is_empty() {
        bail!("Failed to get prefix for {} app(s)", errors.len());
    }
    Ok(())
}
