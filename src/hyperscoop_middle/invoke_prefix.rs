use std::path::Path;
use anyhow::bail;
use crossterm::style::Stylize;
use command_util_lib::init_env::{get_apps_path, get_apps_path_global};
use crate::command_args::prefix::PrefixArgs;

pub  fn  execute_prefix_command(prefix : PrefixArgs ) -> Result<(), anyhow::Error> {
   if let Some(name) = prefix.name 
   {  
      let app_name = name.to_lowercase(); 
      let app_path = if prefix.global {
         get_apps_path_global()
      }else { get_apps_path() } ; 
      #[cfg(debug_assertions)] 
      dbg!(&app_path );
     let path = app_path+"\\"+&app_name+"\\current"; 
      if !Path::new(&path).exists() {
        bail!("{} dir does not exist", app_name);
      }
     println!("{}", path.green().bold()); 
   } 
  Ok(()) 
}