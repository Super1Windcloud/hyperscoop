use crate::command_args::prefix::PrefixArgs;
use anyhow::bail;
use command_util_lib::init_env::{get_app_current_dir, get_app_current_dir_global};
use crossterm::style::Stylize;
use std::path::Path;

pub fn execute_prefix_command(prefix: PrefixArgs) -> Result<(), anyhow::Error> {
    if let Some(name) = prefix.name {
        let app_path = if prefix.global {
            get_app_current_dir_global(name.as_str())
        } else {
            get_app_current_dir(name.as_str())
        };
        if !Path::new(&app_path).exists() {
            bail!("{} 不存在", app_path.red().bold());
        }
        println!("{}", app_path.dark_green().bold());
    }
    Ok(())
}
