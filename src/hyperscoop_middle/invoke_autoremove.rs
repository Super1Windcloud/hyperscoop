use crate::command_args::autoremove::AutoremoveArgs;
use crate::command_args::uninstall::UninstallArgs;
use crate::hyperscoop_middle::invoke_uninstall::execute_uninstall_command;
use crate::i18n::{tr, tr_fmt};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use command_util_lib::depends::find_orphaned_dependencies;
use command_util_lib::size::{format_bytes, get_app_disk_usage};
use command_util_lib::utils::utility::prompt_confirm;
use crossterm::style::Stylize;

pub fn execute_autoremove_command(args: AutoremoveArgs) -> Result<(), anyhow::Error> {
    println!(
        "{}",
        tr(
            "==> Scanning for orphaned dependencies...",
            "==> 正在扫描残留的孤儿依赖包..."
        )
        .dark_cyan()
        .bold()
    );

    let orphans = find_orphaned_dependencies(args.global)?;

    if orphans.is_empty() {
        println!(
            "{}",
            tr(
                "🎉 No orphaned dependencies found. Your environment is clean!",
                "🎉 未发现任何残留的孤儿依赖包，环境非常整洁！"
            )
            .dark_green()
            .bold()
        );
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new(tr("Orphaned Package", "孤儿依赖包"))
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
            Cell::new(tr("Version", "版本"))
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
            Cell::new(tr("Bucket", "来源 Bucket"))
                .add_attribute(Attribute::Bold)
                .fg(Color::Magenta),
            Cell::new(tr("Reclaimable Space", "可释放空间"))
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
        ]);

    let mut total_reclaimable = 0;
    for orphan in &orphans {
        let size = get_app_disk_usage(&orphan.app_name, orphan.is_global)
            .map(|u| u.total_size)
            .unwrap_or(0);
        total_reclaimable += size;

        table.add_row(vec![
            Cell::new(&orphan.app_name).fg(Color::White),
            Cell::new(&orphan.version).fg(Color::Green),
            Cell::new(&orphan.bucket).fg(Color::DarkMagenta),
            Cell::new(format_bytes(size)).fg(Color::Cyan),
        ]);
    }

    println!("{table}");
    println!(
        "{}",
        tr_fmt!(
            "Found {count} orphaned package(s) that can free up {space}.",
            "共发现 {count} 个孤儿依赖包，预计可释放 {space} 磁盘空间。",
            count = orphans.len(),
            space = format_bytes(total_reclaimable)
        )
        .dark_yellow()
        .bold()
    );

    if args.dry_run {
        println!(
            "\n{}",
            tr(
                "ℹ️ Dry-run mode: No packages were removed.",
                "ℹ️ 演练模式（Dry-run）：未实际卸载任何包。"
            )
            .dark_grey()
        );
        return Ok(());
    }

    if !args.yes {
        let confirm_msg = tr(
            "Do you want to uninstall these orphaned packages?",
            "是否确认卸载以上所有孤儿依赖包？",
        );
        let confirmed = prompt_confirm(confirm_msg, true).unwrap_or(false);

        if !confirmed {
            println!(
                "{}",
                tr("Autoremove cancelled by user.", "用户已取消孤儿清理操作。").dark_grey()
            );
            return Ok(());
        }
    }

    let orphan_names: Vec<String> = orphans.into_iter().map(|o| o.app_name).collect();
    let uninstall_args = UninstallArgs {
        app_names: orphan_names.clone(),
        purge: true,
        global: args.global,
        force: true,
        dry_run: false,
    };

    execute_uninstall_command(uninstall_args)?;

    println!(
        "\n{}",
        tr_fmt!(
            "🎉 Successfully removed {count} orphaned package(s), freed ~{space}!",
            "🎉 成功清理 {count} 个孤儿依赖包，释放约 {space} 空间！",
            count = orphan_names.len(),
            space = format_bytes(total_reclaimable)
        )
        .dark_green()
        .bold()
    );

    Ok(())
}
