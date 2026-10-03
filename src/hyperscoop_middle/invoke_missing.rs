use crate::command_args::missing::MissingArgs;
use crate::i18n::tr;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use command_util_lib::init_env::{get_app_dir_manifest_json, get_app_dir_manifest_json_global};
use command_util_lib::list::list_all_installed_apps_refactor;
use crossterm::style::Stylize;
use serde_json::Value;
use std::path::Path;

pub fn execute_missing_command(args: MissingArgs) -> anyhow::Result<()> {
    let apps_to_check: Vec<(String, bool)> = if let Some(app_name) = args.app_name {
        vec![(app_name, args.global)]
    } else {
        let mut apps = Vec::new();
        if let Ok(installed) = list_all_installed_apps_refactor(false) {
            for a in installed {
                apps.push((a.name, false));
            }
        }
        if args.global {
            if let Ok(installed_global) = list_all_installed_apps_refactor(true) {
                for a in installed_global {
                    apps.push((a.name, true));
                }
            }
        }
        apps
    };

    let mut missing_records: Vec<(String, String)> = Vec::new();

    for (app, is_global) in apps_to_check {
        let manifest_path = if is_global {
            get_app_dir_manifest_json_global(&app)
        } else {
            get_app_dir_manifest_json(&app)
        };

        if !Path::new(&manifest_path).exists() {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(deps_val) = json.get("depends") {
                    let mut dep_names = Vec::new();
                    match deps_val {
                        Value::String(s) => {
                            if !s.is_empty() {
                                dep_names.push(s.clone());
                            }
                        }
                        Value::Array(arr) => {
                            for item in arr {
                                if let Some(s) = item.as_str() {
                                    if !s.is_empty() {
                                        dep_names.push(s.to_string());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    for dep in dep_names {
                        // Extract package name if it has bucket prefix like "main/7zip"
                        let clean_dep = dep.split('/').last().unwrap_or(&dep).trim().to_lowercase();
                        let dep_user_exists =
                            Path::new(&get_app_dir_manifest_json(&clean_dep)).exists();
                        let dep_global_exists =
                            Path::new(&get_app_dir_manifest_json_global(&clean_dep)).exists();

                        if !dep_user_exists && !dep_global_exists {
                            missing_records.push((app.clone(), dep));
                        }
                    }
                }
            }
        }
    }

    if missing_records.is_empty() {
        println!(
            "{}",
            tr(
                "All installed apps have satisfied dependencies!",
                "所有已安装应用的基础依赖均已完整满足！"
            )
            .green()
            .bold()
        );
    } else {
        println!(
            "{}\n",
            tr(
                "Found missing dependencies for installed apps:",
                "发现以下已安装应用缺失前置依赖项："
            )
            .dark_red()
            .bold()
        );

        let mut table = Table::new();
        table
            .load_preset(UTF8_BORDERS_ONLY)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new(tr("Installed App", "已安装应用"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Green),
                Cell::new(tr("Missing Dependency", "缺失依赖项"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Red),
            ]);

        for (app, dep) in &missing_records {
            table.add_row(vec![
                Cell::new(app).add_attribute(Attribute::Bold),
                Cell::new(dep).fg(Color::Red),
            ]);
        }

        println!("{table}\n");
        println!(
            "{}",
            tr(
                "Tip: Run 'hp install <dependency>' to install any missing requirements.",
                "提示：运行 'hp install <缺失依赖名>' 即可快速补充安装。"
            )
            .cyan()
        );
    }

    Ok(())
}
