use crossterm::style::Stylize;
use crate::command_args::uninstall::UninstallArgs;
use command_util_lib::uninstall::* ; 
pub fn execute_uninstall_command(args: UninstallArgs) ->Result<() ,anyhow::Error>  { 
     if let Some(app_name) = args.app_name {  
       if args.purge { 
         log::info!("purging app {}", &app_name);   
          uninstall_app_with_purge(&app_name)?;
         println!("'{}' {}", app_name.dark_green().bold(), "has been uninstalled".dark_green().bold());
         
       } 
       else {
         log::info!("Uninstalling app {}", &app_name);
         uninstall_app(&app_name)?;
         println!("'{}' {}", app_name.dark_green().bold(), "has been uninstalled".dark_green().bold());
          
       }
     }
  
  
  Ok(() )
}