use crate::command_args::uninstall::UninstallArgs;
use anyhow::bail;
use command_util_lib::init_env::get_app_dir;
use command_util_lib::uninstall::*;
use crossterm::style::Stylize;
use std::path::Path;
pub fn execute_uninstall_command(args: UninstallArgs) -> Result<(), anyhow::Error> {
    if let Some(app_name) = args.app_name {
        if args.purge {
            log::info!("purging app {}", &app_name);
            let result = uninstall_app_with_purge(&app_name);
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
            let result = uninstall_app(&app_name);
            match result {
                Ok(_) => {
                    println!(
                        "'{}' {}",
                        app_name.dark_cyan().bold(),
                        "has been uninstalled successfully".dark_green().bold()
                    );
                }
                Err(e) => {
                    let app_dir = get_app_dir(&app_name);
                    let app_dir = Path::new(&app_dir);
                    if args.force {
                        if app_dir.exists() {
                            println!(
                                "'{}' {}",
                                app_name.dark_cyan().bold(),
                                "has been uninstalled successfully".dark_green().bold()
                            );
                            return Ok(());
                        }else {
                          bail!("'{app_name}' 并没有安装")
                        }
                    }
                    bail!("Failed to uninstall app, {}", e)
                }
            }
        }
    }

    Ok(())
}
