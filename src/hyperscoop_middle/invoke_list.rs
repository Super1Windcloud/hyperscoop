use crate::command_args::list::ListArgs;
use command_util_lib::list::{display_apps_info_extra, list_specific_installed_apps_extra};

pub fn execute_list_installed_apps(option: ListArgs) -> Result<(), anyhow::Error> {
    match option.name {
        Some(query) => list_specific_installed_apps_extra(query, option.global)?,
        None => display_apps_info_extra(option.global)?,
    };
    Ok(())
}
