use crate::command_args::leaves::LeavesArgs;
use crate::i18n::{tr, tr_fmt};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use command_util_lib::depends::find_leaf_apps;
use crossterm::style::Stylize;

pub fn execute_leaves_command(args: LeavesArgs) -> Result<(), anyhow::Error> {
    let leaves = find_leaf_apps(args.global)?;

    if leaves.is_empty() {
        println!(
            "{}",
            tr(
                "No leaf applications found.",
                "未发现任何顶层独立应用（叶子节点）。"
            )
            .dark_grey()
        );
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new(tr("Leaf Application", "顶层独立应用"))
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
        ]);

    for leaf in &leaves {
        let scope_str = if leaf.is_global {
            tr("Global", "全局")
        } else {
            tr("User", "用户")
        };

        table.add_row(vec![
            Cell::new(&leaf.app_name).fg(Color::White),
            Cell::new(&leaf.version).fg(Color::Green),
            Cell::new(&leaf.bucket).fg(Color::DarkMagenta),
            Cell::new(scope_str).fg(if leaf.is_global {
                Color::Yellow
            } else {
                Color::Grey
            }),
        ]);
    }

    println!("{table}");
    println!(
        "{}",
        tr_fmt!(
            "Found {count} top-level leaf application(s) (not required by any other installed package).",
            "共找到 {count} 个顶层应用（未被任何其他已安装应用依赖）。",
            count = leaves.len()
        )
        .dark_green()
        .bold()
    );

    Ok(())
}
