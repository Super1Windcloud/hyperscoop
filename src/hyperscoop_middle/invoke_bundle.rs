use crate::command_args::bundle::{BundleArgs, BundleSubcommands};
use crate::i18n::{tr, tr_fmt};
use anyhow::bail;
use command_util_lib::bundle::{Hpfile, check_bundle, install_bundle};
use crossterm::style::Stylize;
use std::path::Path;

pub fn execute_bundle_command(args: BundleArgs) -> Result<(), anyhow::Error> {
    match args.command {
        BundleSubcommands::Dump(dump_args) => {
            let target_path = Path::new(&dump_args.file);
            if target_path.exists() && !dump_args.force {
                bail!(
                    "{}",
                    tr_fmt!(
                        "File '{path}' already exists. Use --force (-f) to overwrite.",
                        "文件 '{path}' 已存在。使用 --force (-f) 强制覆盖。",
                        path = dump_args.file
                    )
                );
            }

            println!(
                "{}",
                tr(
                    "==> Exporting current environment to Hpfile...",
                    "==> 正在导出当前环境至 Hpfile..."
                )
                .dark_cyan()
                .bold()
            );

            let hpfile = Hpfile::dump_current()?;
            let content = hpfile.to_string_formatted();
            std::fs::write(target_path, content)?;

            println!(
                "{}",
                tr_fmt!(
                    "🎉 Successfully dumped {buckets} bucket(s) and {apps} app(s) to '{path}'",
                    "🎉 成功导出 {buckets} 个 Bucket 和 {apps} 个应用至 '{path}'",
                    buckets = hpfile.buckets.len(),
                    apps = hpfile.apps.len(),
                    path = dump_args.file
                )
                .dark_green()
                .bold()
            );
        }

        BundleSubcommands::Install(install_args) => {
            let path = Path::new(&install_args.file);
            if !path.is_file() {
                bail!(
                    "{}",
                    tr_fmt!(
                        "Hpfile '{path}' not found.",
                        "未找到 Hpfile 配置文件 '{path}'。",
                        path = install_args.file
                    )
                );
            }

            println!(
                "{}",
                tr_fmt!(
                    "==> Reading bundle configuration from '{path}'...",
                    "==> 正在从 '{path}' 读取配置...",
                    path = install_args.file
                )
                .dark_cyan()
                .bold()
            );

            let hpfile = Hpfile::from_file(path)?;
            install_bundle(&hpfile)?;

            println!(
                "\n{}",
                tr(
                    "🎉 Hpfile bundle install completed!",
                    "🎉 Hpfile 环境安装完成！"
                )
                .dark_green()
                .bold()
            );
        }

        BundleSubcommands::Check(check_args) => {
            let path = Path::new(&check_args.file);
            if !path.is_file() {
                bail!(
                    "{}",
                    tr_fmt!(
                        "Hpfile '{path}' not found.",
                        "未找到 Hpfile 配置文件 '{path}'。",
                        path = check_args.file
                    )
                );
            }

            let hpfile = Hpfile::from_file(path)?;
            let check_result = check_bundle(&hpfile)?;

            println!(
                "{}",
                tr(
                    "==> Checking Hpfile dependencies...",
                    "==> 正在检查 Hpfile 依赖状态..."
                )
                .dark_cyan()
                .bold()
            );

            println!("\n{}", tr("Buckets:", "Bucket 状态:").dark_cyan());
            for b in &check_result.satisfied_buckets {
                println!("  {} {}", "✔".dark_green().bold(), b);
            }
            for b in &check_result.missing_buckets {
                println!(
                    "  {} {} ({})",
                    "✖".dark_red().bold(),
                    b.name,
                    tr("not added", "未添加").dark_yellow()
                );
            }

            println!("\n{}", tr("Applications:", "应用状态:").dark_cyan());
            for a in &check_result.satisfied_apps {
                println!("  {} {}", "✔".dark_green().bold(), a);
            }
            for a in &check_result.missing_apps {
                println!(
                    "  {} {} ({})",
                    "✖".dark_red().bold(),
                    a.name,
                    tr("not installed", "未安装").dark_yellow()
                );
            }

            if check_result.is_all_satisfied() {
                println!(
                    "\n{}",
                    tr(
                        "🎉 All dependencies in Hpfile are satisfied!",
                        "🎉 Hpfile 中声明的所有依赖均已满足！"
                    )
                    .dark_green()
                    .bold()
                );
            } else {
                let missing_count =
                    check_result.missing_buckets.len() + check_result.missing_apps.len();
                bail!(
                    "{}",
                    tr_fmt!(
                        "{count} item(s) in Hpfile are missing or not satisfied. Run 'hp bundle install' to resolve.",
                        "Hpfile 中仍有 {count} 项依赖未满足。可运行 'hp bundle install' 进行安装补充。",
                        count = missing_count
                    )
                );
            }
        }
    }

    Ok(())
}
