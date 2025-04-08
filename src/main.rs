#![feature(str_as_str)]
#![deny(clippy::shadow)]
mod command_args;

use clap::builder::styling::{AnsiColor, Effects};
use clap::builder::Styles;
use clap::{command, Parser};
use clap_verbosity_flag;
use crossterm::execute;
use crossterm::style::{Print, Stylize};
use std::io::stdout;
mod command;
mod hyperscoop_middle;
use command::Commands;
use hyperscoop_middle::*;
mod logger_err;
use logger_err::init_logger;
mod check_self_update;
use crate::command::{execute_credits_command, execute_hold_command};
use check_self_update::*;

const WONDERFUL_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Red.on_default().effects(Effects::BOLD));

#[derive(Parser, Debug)]
#[command(name="hp" , version, about= "Next Generation Faster, Stronger and Beautiful Windows Package Manager" , long_about = None)]
#[command(propagate_version = true)] //  版本信息传递
#[command(override_usage = "hp  [COMMAND]  [OPTIONS] ")]
#[command(
    author = "superwindcloud",
    name = "hp",
    disable_help_flag = false,
    disable_help_subcommand = true,
    disable_version_flag = false
)]
#[command(after_help = "For more information about a command, run: hp  [COMMAND] -h/--help", after_long_help = None)]
#[command(disable_colored_help = false , styles = WONDERFUL_STYLES )]
struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(
        short,
        long,
        required = false,
        global = true,
        help = "安装到系统目录",
        help_heading = "Global Options"
    )]
    pub global: bool,
    #[arg(
        short,
        long,
        required = false,
        global = true,
        help = "开启日志调试模式",
        help_heading = "Global Options"
    )]
    pub debug: bool,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}
#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    println!(
        "{ } \n ",
        "🦀 次世代更快更强更精美的Windows包管理器!"
            .dark_magenta()
            .bold()
    );
    let cli = Cli::parse();
    init_logger(&cli);
    color_eyre::install().unwrap();

    let result = match cli.command {
        None => {
            eprintln!("No command provided. Run `hp  --help` to see available commands!");
            Ok(())
        }
        Some(input_command) => match input_command {
            Commands::Bucket(bucket) => execute_bucket_command(&bucket.command).await,
            Commands::Cat(cat) => execute_cat_command(cat),
            Commands::Cache(cache_args) => execute_cache_command(cache_args),
            Commands::Checkup( args ) => execute_checkup_command(args.global),
            Commands::Cleanup(args) => execute_cleanup_command(args),
            Commands::Config(args) => execute_config_command(args),
            Commands::Export(file) => execute_export_command(file),
            Commands::Home(home) => execute_home_command(home),
            Commands::Import(args) => execute_import_command(args),
            Commands::Info(info) => execute_info_command(info),
            Commands::Install(args) => {
                auto_check_hp_update().await?;
                execute_install_command(args).await
            }
            Commands::List(query_app) => execute_list_installed_apps(query_app ),
            Commands::Prefix(prefix) => execute_prefix_command(prefix),
            Commands::Reset(args) => execute_reset_command(args),
            Commands::Search(search_app) => execute_search_command(search_app),
            Commands::Shim(args) => execute_shim_command(args),
            Commands::Status(args ) => execute_status_command(args ),
            Commands::Uninstall(args) => execute_uninstall_command(args),
            Commands::Update(update_args) => {
                auto_check_hp_update().await?;
                execute_update_command(update_args).await
            }
            Commands::Which(which) => execute_which_command(which),
            Commands::Merge(args) => execute_merge_command(args),
            Commands::Credits(_) => execute_credits_command(),
            Commands::Hold( hold_args ) => execute_hold_command( hold_args),
        },
    };
    if let Err(err) = result {
        let red_err = err.to_string().dark_red().bold();
        execute!(stdout(), Print(red_err))?;
        println!();
    }
    Ok(())
}
