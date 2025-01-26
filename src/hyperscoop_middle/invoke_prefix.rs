use crossterm::style::Stylize;
use command_util_lib::init_hyperscoop;
use crate::command_args::prefix::PrefixArgs;

pub  fn  execute_prefix_command(prefix : PrefixArgs ) -> Result<(), anyhow::Error> {
   if let Some(name) = prefix.name 
   {  
      let app_name = name.to_lowercase(); 
     let hyperscoop =init_hyperscoop().expect("Failed to initialize hyperscoop"); 
      let app_path = hyperscoop.get_apps_path(); 
      #[cfg(debug_assertions)] 
      dbg!(&app_path );
     let path = app_path+"\\"+&app_name+"\\current";
     println!("{}", path.green().bold()); 
   } 
  Ok(()) 
}