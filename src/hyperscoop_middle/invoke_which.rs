use crossterm::style::Stylize;
use command_util_lib::init_hyperscoop;
use crate::command_args::which::WhichArgs;

pub fn execute_which_command(command: WhichArgs) ->  Result<(), anyhow::Error> {
  if let Some(name) = command.name {
    let app_name = name.to_lowercase();
    let hyperscoop = init_hyperscoop().expect("Failed to initialize hyperscoop");
    let app_path = hyperscoop.get_apps_path();
    for entry in std::fs::read_dir(&app_path)? {
      let entry = entry?;
      let path = entry.path();
      if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_string_lossy();
          if file_name .as_str()!=app_name {  
            let  app = app_name.clone();
            // println!("{} is not installed or not exist ", app.red().bold());
          continue; }
        let path = app_path + "\\" + &app_name + "\\current\\" + &app_name + ".exe";
        println!("{}", path.green().bold());
        return Ok(());
      }
    }
  }
  Ok(())
}