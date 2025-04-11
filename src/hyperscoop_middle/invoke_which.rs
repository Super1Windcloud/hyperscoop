use crate::command_args::which::WhichArgs;
use anyhow::bail;
use command_util_lib::init_env::{get_apps_path, get_apps_path_global};
use crossterm::style::Stylize;
use std::path::Path;

pub fn execute_which_command(command: WhichArgs) -> Result<(), anyhow::Error> {
    if let Some(name) = command.name {
        let app_name = name.to_lowercase();

        let app_path = if command.global {
            get_apps_path_global()
        } else {
            get_apps_path()
        };
        if !Path::new(&app_path).exists() {
            bail!("{} 不存在", app_path.red().bold());
        }
        for entry in std::fs::read_dir(&app_path)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_str().unwrap();
                if file_name != app_name {
                    continue;
                }
                let path = app_path + "\\" + &app_name + "\\current\\" + &app_name + ".exe";
                println!("{}", path.green().bold());
                return Ok(());
            }
        }
    }
    Ok(())
}
