use crate::command_args::uninstall::UninstallArgs;
use crate::i18n::{tr, tr_fmt};
use anyhow::bail;
use command_util_lib::uninstall::*;
use command_util_lib::utils::system::{is_admin, request_admin};
use crossterm::style::Stylize;
use std::env;

pub fn execute_uninstall_command(args: UninstallArgs) -> Result<(), anyhow::Error> {
    if args.app_names.is_empty() {
        return Ok(());
    }

    if args.global && !is_admin()? {
        let args_env = env::args().skip(1).collect::<Vec<String>>();
        let args_str = args_env.join(" ");
        log::warn!(
            "Global command arguments: {}",
            args_str.clone().dark_yellow()
        );
        request_admin(args_str.as_str())?;
        return Ok(());
    }

    let total = args.app_names.len();
    let mut errors = Vec::new();

    for (i, app_name) in args.app_names.iter().enumerate() {
        if total > 1 {
            println!(
                "{}",
                tr_fmt!(
                    "\n[{i}/{total}] Uninstalling '{name}'...",
                    "\n[{i}/{total}] 正在卸载 '{name}'...",
                    i = i + 1,
                    total = total,
                    name = app_name
                )
                .dark_cyan()
                .bold()
            );
        }

        // Check if other installed apps depend on this app
        if let Ok(dependents) =
            command_util_lib::depends::find_installed_dependents(app_name, args.global)
        {
            let remaining_dependents: Vec<_> = dependents
                .into_iter()
                .filter(|d| {
                    !args
                        .app_names
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(&d.app_name))
                })
                .collect();

            if !remaining_dependents.is_empty() {
                let dep_names = remaining_dependents
                    .iter()
                    .map(|d| format!("'{}'", d.app_name))
                    .collect::<Vec<_>>()
                    .join(", ");

                if !args.force {
                    eprintln!(
                        "{}",
                        tr_fmt!(
                            "⚠️ Cannot uninstall '{name}': it is required by installed app(s): [{deps}].",
                            "⚠️ 无法卸载 '{name}'：以下已安装的应用仍依赖它: [{deps}]。",
                            name = app_name,
                            deps = dep_names
                        )
                        .dark_yellow()
                        .bold()
                    );
                    eprintln!(
                        "{}",
                        tr_fmt!(
                            "   Use 'hp uninstall {name} --force' (-f) to uninstall anyway.",
                            "   可使用 'hp uninstall {name} --force' (-f) 强制卸载。",
                            name = app_name
                        )
                        .dark_grey()
                    );
                    errors.push((
                        app_name.clone(),
                        anyhow::anyhow!(tr_fmt!(
                            "App '{name}' is required by other installed packages",
                            "应用 '{name}' 被其他已安装应用依赖",
                            name = app_name
                        )),
                    ));
                    continue;
                } else {
                    println!(
                        "{}",
                        tr_fmt!(
                            "⚠️ Warning: '{name}' is required by [{deps}]. Proceeding due to --force.",
                            "⚠️ 警告：'{name}' 仍被 [{deps}] 依赖。由于指定了 --force，将强制继续卸载。",
                            name = app_name,
                            deps = dep_names
                        )
                        .dark_yellow()
                    );
                }
            }
        }

        if let Err(e) = uninstall_single_app(app_name, args.global, args.purge) {
            eprintln!(
                "{}",
                tr_fmt!(
                    "Failed to uninstall '{name}': {e}",
                    "卸载 '{name}' 失败: {e}",
                    name = app_name,
                    e = e
                )
                .dark_red()
                .bold()
            );
            errors.push((app_name.clone(), e));
        }
    }

    if !errors.is_empty() {
        if total > 1 {
            eprintln!(
                "{}",
                tr_fmt!(
                    "\n{failed} of {total} apps failed to uninstall.",
                    "\n共有 {failed} 个应用（共 {total} 个）卸载失败。",
                    failed = errors.len(),
                    total = total
                )
                .dark_red()
                .bold()
            );
            bail!(
                "{}",
                tr_fmt!(
                    "{failed} app(s) failed to uninstall",
                    "{failed} 个应用卸载失败",
                    failed = errors.len()
                )
            );
        } else {
            let (_, err) = errors.remove(0);
            return Err(err);
        }
    }

    Ok(())
}

fn uninstall_single_app(app_name: &str, global: bool, purge: bool) -> Result<(), anyhow::Error> {
    if purge {
        log::info!("purging app {}", app_name);
        let result = uninstall_app_with_purge(app_name, global);
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
                bail!(
                    "{}",
                    tr_fmt!("Failed to purge app: {e}", "彻底卸载应用失败: {e}", e = e)
                )
            }
        }
    } else {
        log::info!("Uninstalling app {}", app_name);

        let result = uninstall_app(app_name, global);
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
            Err(e) => {
                bail!(
                    "{}",
                    tr_fmt!("Failed to uninstall app: {e}", "卸载应用失败: {e}", e = e)
                );
            }
        }
    }

    Ok(())
}
