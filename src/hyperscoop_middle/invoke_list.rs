use command_util_lib::{list_specific_installed_apps, display_app_info};

pub fn execute_list_installed_apps(option: Option<&String>) -> Result<(), anyhow::Error> {
  match option {
    Some(query) => {
      list_specific_installed_apps(query)
    }
    None => {
      display_app_info()
    }
  };
  Ok(())
}

