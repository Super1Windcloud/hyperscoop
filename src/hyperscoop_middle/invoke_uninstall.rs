use crate::command_args::uninstall::UninstallArgs;
use crate::i18n::tr;
use anyhow::{Context, bail};
use command_util_lib::init_env::{get_app_dir, get_app_dir_global};
use command_util_lib::uninstall::*;
use command_util_lib::utils::system::{is_admin, kill_processes_using_app, request_admin};
use crossterm::style::Stylize;
use std::env;
use std::path::Path;

pub fn execute_uninstall_command(args: UninstallArgs) -> Result<(), anyhow::Error> {
    if let Some(app_name) = args.app_name {
        if args.global && !is_admin()? {
            let args = env::args().skip(1).collect::<Vec<String>>();
            let args_str = args.join(" ");
            log::warn!(
                "Global command arguments: {}",
                args_str.clone().dark_yellow()
            );
            request_admin(args_str.as_str())?;
            return Ok(());
        }

        if args.purge {
            log::info!("purging app {}", &app_name);
            let result = uninstall_app_with_purge(&app_name, args.global);
            match result {
                Ok(_) => {
                    println!(
                        "'{}' {}",
                        app_name.dark_cyan().bold(),
                        tr("was purge uninstalled successfully!", "已执行彻底卸载！")
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

            let result = uninstall_app(&app_name, args.global);
            match result {
                Ok(_) => {
                    println!(
                        "'{}' {}",
                        app_name.dark_cyan().bold(),
                        tr("was already uninstalled successfully!", "已成功卸载！")
                            .dark_green()
                            .bold()
                    );
                }
                Err(_) => {
                    let app_dir = if args.global {
                        get_app_dir_global(&app_name)
                    } else {
                        get_app_dir(&app_name)
                    };
                    let app_dir = Path::new(&app_dir);
                    if app_dir.exists() {
                        if std::fs::remove_dir_all(app_dir)
                            .context(format!(
                                "Failed to remove app directory {}",
                                app_dir.display()
                            ))
                            .is_err()
                        {
                            kill_processes_using_app(&app_name);
                            std::fs::remove_dir_all(app_dir).context(format!(
                                "Failed to remove app dir  {} at line 68",
                                app_dir.display()
                            ))?;
                        }

                        println!(
                            "'{}' {}",
                            app_name.clone().dark_cyan().bold(),
                            tr("has been uninstalled successfully!", "已成功卸载！")
                                .dark_green()
                                .bold()
                        );
                    } else {
                        bail!(
                            "{}",
                            format!(
                                "{} {name}",
                                tr("'{name}' is not installed.", "'{name}' 并没有安装。"),
                                name = app_name
                            )
                        )
                    }
                }
            }
        }
    }

    Ok(())
}
