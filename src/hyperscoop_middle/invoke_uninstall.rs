use crate::command_args::uninstall::UninstallArgs;
use anyhow::{bail, Context};
use command_util_lib::init_env::{get_app_dir, get_app_dir_global};
use command_util_lib::uninstall::*;
use crossterm::style::Stylize;
use std::path::Path;
use command_util_lib::utils::system::{is_admin, request_admin};

pub fn execute_uninstall_command(args: UninstallArgs) -> Result<(), anyhow::Error> {
   if   args.global {
       if  !is_admin()?{ 
         request_admin() 
       }
   }
    if let Some(app_name) = args.app_name {
        if args.purge {
            log::info!("purging app {}", &app_name);
            let result = uninstall_app_with_purge(&app_name , args.global);
            match result {
                Ok(_) => {
                    println!(
                        "'{}' {}",
                        app_name.dark_cyan().bold(),
                        "has been purge uninstalled successfully"
                            .dark_green()
                            .bold()
                    );
                }
                Err(e) => {
                    bail!("Failed to purge app, {}", e)
                }
            }
        } else {
            log::info!("Uninstalling app {}", &app_name); 
          
            let result = uninstall_app(&app_name , args.global );
            match result {
                Ok(_) => {
                    println!(
                        "'{}' {}",
                        app_name.dark_cyan().bold(),
                        "has been uninstalled successfully".dark_green().bold()
                    );
                }
                Err(e) => {
                    if args.force {
                      let app_dir = if args.global {
                        get_app_dir_global(&app_name)
                      } else {  get_app_dir(&app_name)} ;
                      let app_dir = Path::new(&app_dir);
                        if app_dir.exists() {
                            println!(
                                "'{}' {}",
                                app_name.dark_cyan().bold(),
                                "has been uninstalled successfully".dark_green().bold()
                            ); 
                            std::fs::remove_dir_all(app_dir)
                              .context(format!("Failed to remove app directory {}", app_dir.display()))?; 
                            return Ok(());
                        }else {
                          bail!("'{app_name}' 并没有安装")
                        }
                    }
                    else { bail!("Failed to uninstall app, {}", e) } 
                }
            }
        }
    }

    Ok(())
}
