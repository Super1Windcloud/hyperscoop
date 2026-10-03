#![allow(
    clippy::collapsible_if,
    clippy::double_ended_iterator_last,
    clippy::empty_line_after_outer_attr,
    clippy::enum_variant_names,
    clippy::format_in_format_args,
    clippy::iter_count,
    clippy::let_and_return,
    clippy::let_unit_value,
    clippy::manual_strip,
    clippy::manual_unwrap_or_default,
    clippy::needless_bool,
    clippy::needless_borrow,
    clippy::needless_borrows_for_generic_args,
    clippy::needless_return,
    clippy::single_component_path_imports,
    clippy::unnecessary_unwrap,
    clippy::useless_asref,
    clippy::useless_conversion,
    clippy::useless_format,
    clippy::useless_vec
)]

mod command_args;

use crate::command_args::install::InstallArgs;
use crate::i18n::tr;
use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use clap_verbosity_flag;
use crossterm::execute;
use std::io::stdout;

mod command;
mod hyperscoop_middle;
use hyperscoop_middle::*;
mod logger_err;
use logger_err::init_logger;
mod check_self_update;
mod i18n;
rust_i18n::i18n!("locales");
use crate::command::{Commands, HoldArgs, execute_credits_command, execute_hold_command};
use crate::command_args::alias::execute_alias_command;
#[allow(unused_imports)]
use crate::logger_err::{init_color_output, invoke_admin_process};
use check_self_update::*;
use crossterm::style::{Print, Stylize};

const WONDERFUL_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Red.on_default().effects(Effects::BOLD));

#[derive(Parser, Debug)]
#[command(
    name = "hp",
    version,
    about = tr(
        "Next-generation faster, stronger, and beautiful Windows package manager",
        "次世代更快、更强、更精美的 Windows 包管理器"
    ),
    long_about = None
)]
#[command(propagate_version = true)] //  版本信息传递
#[command(override_usage = "hp  [COMMAND]  [OPTIONS] ")]
#[command(
    author = "SuperWindCloud",
    name = "hp",
    disable_help_flag = false,
    disable_help_subcommand = true,
    disable_version_flag = false
)]
#[command(
    after_help = tr(
        "For more information about a command, run: hp [COMMAND] -h/--help\nYou can set env $SCOOP to change the installation directory.",
        "查看更多命令信息: hp [COMMAND] -h/--help\n可以设置环境变量 $SCOOP 来调整默认的安装目录。"
    ),
    after_long_help = None
)]
#[command(disable_colored_help = false , styles = WONDERFUL_STYLES )]
struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(
        short,
        long,
        required = false,
        global = true,
        help = tr(
            "Install into the system-wide directory",
            "安装到系统目录"
        ),
        help_heading = tr("Global Options", "全局选项")
    )]
    pub global: bool,
    #[arg(
        short,
        long,
        required = false,
        global = true,
        help = tr(
            "Enable verbose log debugging",
            "开启日志调试模式"
        ),
        help_heading = tr("Global Options", "全局选项")
    )]
    pub debug: bool,
    #[arg(
        short = 'E',
        long,
        required = false,
        global = true,
        help = tr(
            "Force error-only logging",
            "忽略日志调试模式，仅输出错误"
        ),
        help_heading = tr("Global Options", "全局选项")
    )]
    pub error: bool,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[arg(
        short = 'N',
        long,
        required = false,
        global = true,
        help = tr(
            "Disable colored output",
            "禁用颜色输出"
        ),
        help_heading = tr("Global Options", "全局选项")
    )]
    pub no_color: bool,
    #[arg(
        short = 'L',
        long = "lang",
        required = false,
        global = true,
        value_enum,
        default_value_t = i18n::LanguageChoice::Auto,
        help = tr(
            "Force CLI language (auto/en/zh)",
            "强制设置 CLI 语言 (auto/en/zh)"
        ),
        help_heading = tr("Global Options", "全局选项")
    )]
    pub language: i18n::LanguageChoice,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    #[cfg(not(debug_assertions))]
    human_panic::setup_panic!();

    command_util_lib::disable_git2_owner_validation();

    let preselected_language = i18n::detect_language_choice_from_args();
    i18n::init_language(preselected_language);

    if !cfg!(windows) {
        eprintln!(
            "{}",
            tr(
                "Error: hyperscoop (hp) only supports Windows.",
                "错误: hyperscoop (hp) 仅支持 Windows 系统。"
            )
        );
        std::process::exit(1);
    }
    let cli = Cli::parse();
    init_color_output(cli.no_color);
    unsafe {
        init_logger(&cli);
    }
    let _ = color_eyre::install();
    // if cli.command.is_some() && cli.global {
    //     invoke_admin_process()?;
    //     return Ok(());
    // }

    let result = match cli.command {
        None => {
            auto_check_hp_update(None).await?;
            Ok(())
        }
        Some(input_command) => match input_command {
            Commands::Alias(alias_args) => execute_alias_command(alias_args),
            Commands::Autoremove(args) => execute_autoremove_command(args),
            Commands::Bucket(bucket) => execute_bucket_command(bucket),
            Commands::Bundle(args) => execute_bundle_command(args),
            Commands::Cat(cat) => execute_cat_command(cat),
            Commands::Cache(cache_args) => execute_cache_command(cache_args),
            Commands::Checkup(args) => execute_checkup_command(args.global).await,
            Commands::Cleanup(args) => execute_cleanup_command(args),
            Commands::Completion(args) => {
                let mut cmd = Cli::command();
                generate(args.shell, &mut cmd, "hp", &mut std::io::stdout());
                Ok(())
            }
            Commands::Config(args) => execute_config_command(args),
            Commands::Deps(args) => execute_deps_command(args),
            Commands::Desc(args) => execute_desc_command(args),
            Commands::Edit(args) => execute_edit_command(args),
            Commands::Export(file) => execute_export_command(file),
            Commands::Fetch(args) => {
                let install_args = InstallArgs {
                    app_names: args.app_names,
                    skip_download_hash_check: args.skip_download_hash_check,
                    no_use_download_cache: args.no_use_download_cache,
                    no_auto_download_dependencies: false,
                    dry_run: false,
                    only_download_no_install: true,
                    only_download_no_install_with_override_cache: false,
                    update_hp_and_buckets: args.update_hp_and_buckets,
                    check_version_up_to_date: false,
                    interactive: false,
                    force_install_override: false,
                    arch: args.arch,
                    app_alias_from_url_install: None,
                    global: args.global,
                };
                execute_install_command(install_args).await
            }
            Commands::Home(home) => execute_home_command(home),
            Commands::Hold(hold_args) => execute_hold_command(hold_args),
            Commands::Import(args) => execute_import_command(args),
            Commands::Info(info) => execute_info_command(info),
            Commands::Install(args) => execute_install_command(args).await,
            Commands::Leaves(args) => execute_leaves_command(args),
            Commands::Link(args) => execute_link_command(args),
            Commands::List(query_app) => execute_list_installed_apps(query_app),
            Commands::Log(args) => execute_log_command(args),
            Commands::Missing(args) => execute_missing_command(args),
            Commands::Prefix(prefix) => execute_prefix_command(prefix),
            Commands::Reinstall(args) => {
                let install_args = InstallArgs {
                    app_names: args.app_names,
                    skip_download_hash_check: args.skip_download_hash_check,
                    no_use_download_cache: args.no_use_download_cache,
                    no_auto_download_dependencies: args.no_auto_download_dependencies,
                    dry_run: false,
                    only_download_no_install: false,
                    only_download_no_install_with_override_cache: false,
                    update_hp_and_buckets: args.update_hp_and_buckets,
                    check_version_up_to_date: false,
                    interactive: false,
                    force_install_override: true,
                    arch: args.arch,
                    app_alias_from_url_install: None,
                    global: args.global,
                };
                execute_install_command(install_args).await
            }
            Commands::Reset(args) => execute_reset_command(args),
            Commands::Search(search_app) => execute_search_command(search_app),
            Commands::SelfUpdate(args) => execute_self_update_command(args).await,
            Commands::Service(args) => execute_service_command(args),
            Commands::Shellenv(args) => execute_shellenv_command(args),
            Commands::Shim(args) => execute_shim_command(args),
            Commands::Size(args) => execute_size_command(args),
            Commands::Status(args) => execute_status_command(args),
            Commands::Switch(args) => execute_switch_command(args),
            Commands::Uninstall(args) => execute_uninstall_command(args),
            Commands::Unlink(args) => execute_unlink_command(args),
            Commands::Unpin(args) => execute_hold_command(HoldArgs {
                app_names: Some(args.app_names),
                cancel_hold: true,
                global: args.global,
            }),
            Commands::Update(update_args) => execute_update_command(update_args).await,
            Commands::Uses(args) => execute_uses_command(args),
            Commands::Which(which) => execute_which_command(which),
            Commands::Merge(args) => execute_merge_command(args),
            Commands::Credits(_) => execute_credits_command().await,
        },
    };
    if let Err(err) = result {
        let red_err = format!("{:#}", err).dark_red().bold();
        let _ = execute!(stdout(), Print(red_err));
        println!();
        std::process::exit(1);
    }
    Ok(())
}
