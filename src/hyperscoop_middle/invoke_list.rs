use std::fs::read_dir;
use crossterm::style::Stylize;
use command_util_lib::{list_specific_installed_apps, list_all_installed_apps};

pub fn execute_list_installed_apps(option: Option<&String>) -> Result<(), anyhow::Error> {
  match option {
    Some(query) => {
      list_specific_installed_apps(query)
    }
    None => {
      list_all_installed_apps()
    }
  }
  Ok(())
}

