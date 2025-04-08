use crate::command_args::list::ListArgs;
use command_util_lib::{display_app_info, list_specific_installed_apps};

pub fn execute_list_installed_apps(option: ListArgs) -> Result<(), anyhow::Error> {
    match option.name {
        Some(query) => list_specific_installed_apps(query, option.global)?,
        None => display_app_info(option.global)?,
    };
    Ok(())
}
