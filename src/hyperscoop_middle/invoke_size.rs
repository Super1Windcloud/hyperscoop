use crate::command_args::size::SizeArgs;
use crate::i18n::{tr, tr_fmt};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use command_util_lib::size::{format_bytes, get_all_apps_disk_usage, get_app_disk_usage};
use crossterm::style::Stylize;

pub fn execute_size_command(args: SizeArgs) -> Result<(), anyhow::Error> {
    if let Some(ref app_name) = args.app_name {
        let usage = get_app_disk_usage(app_name, args.global)?;

        println!(
            "{}",
            tr_fmt!(
                "Disk usage breakdown for '{name}':",
                "'{name}' 的磁盘空间占用明细：",
                name = app_name
            )
            .dark_cyan()
            .bold()
        );

        let mut table = Table::new();
        table
            .load_preset(UTF8_BORDERS_ONLY)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new(tr("Component", "项目"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new(tr("Size", "占用大小"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Green),
            ]);

        table.add_row(vec![
            Cell::new(tr("Current Version Dir", "当前版本目录")),
            Cell::new(format_bytes(usage.current_dir_size)),
        ]);

        let old_versions_size = usage
            .total_app_dir_size
            .saturating_sub(usage.current_dir_size);
        if old_versions_size > 0 {
            table.add_row(vec![
                Cell::new(tr("Old Versions", "历史旧版本")),
                Cell::new(format_bytes(old_versions_size)),
            ]);
        }

        table.add_row(vec![
            Cell::new(tr("Persisted Data (persist/)", "持久化数据 (persist/)")),
            Cell::new(format_bytes(usage.persist_size)),
        ]);

        table.add_row(vec![
            Cell::new(tr("Download Cache (cache/)", "下载安装包缓存 (cache/)")),
            Cell::new(format_bytes(usage.cache_size)),
        ]);

        table.add_row(vec![
            Cell::new(tr("Total Occupied", "总占用空间")).add_attribute(Attribute::Bold),
            Cell::new(format_bytes(usage.total_size))
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
        ]);

        println!("{table}");
    } else {
        println!(
            "{}",
            tr(
                "Calculating disk usage for installed applications...",
                "正在统计已安装应用的磁盘空间占用..."
            )
            .dark_cyan()
            .bold()
        );

        let usages = get_all_apps_disk_usage(args.global)?;
        if usages.is_empty() {
            println!(
                "{}",
                tr("No installed applications found.", "未检测到已安装的应用。").dark_grey()
            );
            return Ok(());
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_BORDERS_ONLY)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new(tr("Application", "应用程序"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new(tr("Version", "版本"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Green),
                Cell::new(tr("App Dir", "程序目录"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::White),
                Cell::new(tr("Persist", "数据目录"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Magenta),
                Cell::new(tr("Cache", "下载缓存"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::DarkCyan),
                Cell::new(tr("Total Size", "总计占用"))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Yellow),
            ]);

        let mut grand_total = 0;
        for u in &usages {
            grand_total += u.total_size;
            table.add_row(vec![
                Cell::new(&u.app_name).fg(Color::White),
                Cell::new(&u.version).fg(Color::Green),
                Cell::new(format_bytes(u.total_app_dir_size)),
                Cell::new(format_bytes(u.persist_size)),
                Cell::new(format_bytes(u.cache_size)),
                Cell::new(format_bytes(u.total_size))
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Yellow),
            ]);
        }

        println!("{table}");
        println!(
            "{}",
            tr_fmt!(
                "Total disk space used by {count} app(s): {total}",
                "共统计 {count} 个应用，总磁盘占用空间：{total}",
                count = usages.len(),
                total = format_bytes(grand_total)
            )
            .dark_green()
            .bold()
        );
    }

    Ok(())
}
