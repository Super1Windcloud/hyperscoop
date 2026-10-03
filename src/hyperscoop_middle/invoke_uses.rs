use crate::command_args::uses::UsesArgs;
use crate::i18n::{tr, tr_fmt};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use command_util_lib::depends::{find_bucket_dependents, find_installed_dependents};
use crossterm::style::Stylize;

pub fn execute_uses_command(args: UsesArgs) -> Result<(), anyhow::Error> {
    let app_name = &args.app_name;

    if args.all {
        println!(
            "{}",
            tr_fmt!(
                "Searching for manifests depending on '{name}' across all buckets...",
                "正在所有 Bucket 中搜索依赖 '{name}' 的应用清单...",
                name = app_name
            )
            .dark_cyan()
            .bold()
        );

        let bucket_dependents = find_bucket_dependents(app_name)?;

        if bucket_dependents.is_empty() {
            println!(
                "{}",
                tr_fmt!(
                    "No bucket manifests found that depend on '{name}'.",
                    "未在任何 Bucket 中找到依赖 '{name}' 的应用清单。",
                    name = app_name
                )
                .dark_grey()
            );
            return Ok(());
        }

        if args.quiet {
            for (manifest_path, _) in &bucket_dependents {
                println!("{manifest_path}");
            }
            return Ok(());
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_BORDERS_ONLY)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new(tr("Manifest", "应用清单"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new(tr("Declared Dependency", "声明的依赖项"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::DarkYellow),
            ]);

        for (manifest_path, dep) in &bucket_dependents {
            table.add_row(vec![
                Cell::new(manifest_path).fg(Color::White),
                Cell::new(dep).fg(Color::DarkCyan),
            ]);
        }

        println!("{table}");
        println!(
            "{}",
            tr_fmt!(
                "Found {count} bucket manifest(s) depending on '{name}'.",
                "共找到 {count} 个 Bucket 清单依赖 '{name}'。",
                count = bucket_dependents.len(),
                name = app_name
            )
            .dark_green()
            .bold()
        );
    } else {
        let dependents = find_installed_dependents(app_name, args.global)?;

        if dependents.is_empty() {
            println!(
                "{}",
                tr_fmt!(
                    "No installed applications depend on '{name}'.",
                    "当前没有已安装的应用依赖 '{name}'。",
                    name = app_name
                )
                .dark_grey()
            );
            return Ok(());
        }

        if args.quiet {
            for dep in &dependents {
                println!("{}", dep.app_name);
            }
            return Ok(());
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_BORDERS_ONLY)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new(tr("Installed App", "已安装应用"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new(tr("Version", "版本"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Green),
                Cell::new(tr("Bucket", "来源 Bucket"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Magenta),
                Cell::new(tr("Scope", "作用域"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Yellow),
                Cell::new(tr("Declared Dependency", "声明的依赖项"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::DarkCyan),
            ]);

        for dep in &dependents {
            let scope_str = if dep.is_global {
                tr("Global", "全局")
            } else {
                tr("User", "用户")
            };

            table.add_row(vec![
                Cell::new(&dep.app_name).fg(Color::White),
                Cell::new(&dep.version).fg(Color::Green),
                Cell::new(&dep.bucket).fg(Color::DarkMagenta),
                Cell::new(scope_str).fg(if dep.is_global {
                    Color::Yellow
                } else {
                    Color::Grey
                }),
                Cell::new(&dep.declared_dep).fg(Color::Cyan),
            ]);
        }

        println!("{table}");
        println!(
            "{}",
            tr_fmt!(
                "Found {count} installed app(s) depending on '{name}'.",
                "共发现 {count} 个已安装应用依赖 '{name}'。",
                count = dependents.len(),
                name = app_name
            )
            .dark_green()
            .bold()
        );
    }

    Ok(())
}
