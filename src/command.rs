use crate::check_self_update::auto_check_hp_update;
use crate::command_args::alias::AliasArgs;
use crate::command_args::cat::CatArgs;
use crate::command_args::checkup::CheckupArgs;
use crate::command_args::cleanup::CleanupArgs;
use crate::command_args::config::ConfigArgs;
use crate::command_args::export::ExportArgs;
use crate::command_args::home::HomeArgs;
use crate::command_args::import::ImportArgs;
use crate::command_args::info::InfoArgs;
use crate::command_args::install::InstallArgs;
use crate::command_args::list::ListArgs;
use crate::command_args::merge_bucket::MergeArgs;
use crate::command_args::prefix::PrefixArgs;
use crate::command_args::reset::ResetArgs;
use crate::command_args::search::SearchArgs;
use crate::command_args::self_update::SelfUpdateArgs;
use crate::command_args::shim::ShimArgs;
use crate::command_args::status::StatusArgs;
use crate::command_args::uninstall::UninstallArgs;
use crate::command_args::update::UpdateArgs;
use crate::command_args::which::WhichArgs;
pub(crate) use crate::command_args::{bucket_args::BucketArgs, cache::CacheArgs};
use crate::i18n::t;
use anyhow::{Context, bail};
use clap::{Args, Subcommand};
use command_util_lib::init_env::{get_app_dir_install_json, get_app_dir_install_json_global};
use command_util_lib::utils::utility::clap_args_to_lowercase;
use crossterm::style::Stylize;
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Subcommand)]
#[command(propagate_version = true)] // 自动传递版本信息
#[command(subcommand_negates_reqs = true)] // 禁止子命令的短选项冲突
#[command(infer_subcommands = true, infer_long_args = true)] // 自动推断子命令和长选项
#[command(
    arg_required_else_help = true,
    next_line_help = false,
    disable_help_subcommand = true
)]
pub(crate) enum Commands {
    Alias(AliasArgs),
    Bucket(BucketArgs),
    Cat(CatArgs),
    Cache(CacheArgs),
    Checkup(CheckupArgs),
    Cleanup(CleanupArgs),
    Config(ConfigArgs),
    Export(ExportArgs),
    Home(HomeArgs),
    Hold(HoldArgs),
    Import(ImportArgs),
    Info(InfoArgs),
    Install(InstallArgs),
    List(ListArgs),
    Prefix(PrefixArgs),
    Reset(ResetArgs),
    #[clap(alias = "s")]
    Search(SearchArgs),
    SelfUpdate(SelfUpdateArgs),
    Shim(ShimArgs),
    Status(StatusArgs),
    #[clap(alias = "un")]
    Uninstall(UninstallArgs),
    Update(UpdateArgs),
    Which(WhichArgs),
    Merge(MergeArgs),
    Credits(CreditsArgs),
}

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr("💖\t\tShow project credits", "💖\t\t显示 Credits 信息"),
    long_about = None
)]
#[command(arg_required_else_help = false, subcommand_negates_reqs = true)]
#[command(no_binary_name = true)]
pub struct CreditsArgs {}
pub async fn execute_credits_command() -> anyhow::Result<()> {
    if !auto_check_hp_update(None).await? {
        log::debug!("{}", t!("credits.up_to_date").dark_cyan().bold());
    };

    let author_line = t!("credits.author_line").to_string().dark_blue().bold();
    log::debug!("💖\t{author_line}");

    show_reward_img();
    Ok(())
}

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "💖\t\tLock app versions so global updates skip them",
        "💖\t\t锁定指定 APP 版本，后续更新与检测会跳过"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(no_binary_name = true)]
pub struct HoldArgs {
    #[arg(
        required = false,
        num_args = 1..,
        help = crate::i18n::tr(
            "Names of the apps to hold (exact match, supports multiple values)",
            "要锁定的 APP 名称，精确匹配，支持多个参数"
        ),
        value_parser = clap_args_to_lowercase
    )]
    pub app_names: Option<Vec<String>>,
    #[arg(
        short = 'u',
        long,
        required = false,
        help = crate::i18n::tr("Cancel hold for these apps", "取消锁定，支持多个 APP")
    )]
    pub cancel_hold: bool,
    #[arg(
        short = 'g',
        long,
        required = false,
        help = crate::i18n::tr("Hold or unhold globally installed apps", "锁定或解锁全局安装的应用")
    )]
    pub global: bool,
}

pub fn add_key_value_to_json(
    file_path: &str,
    new_key: &str,
    new_value: bool,
    name: &str,
) -> anyhow::Result<()> {
    let data = std::fs::read_to_string(file_path)
        .context(format!("Failed to read file: {} at line 114", file_path))?;

    let mut json_data: Value =
        serde_json::from_str(&data).context("Failed to parse JSON data at line 116")?;

    if let Value::Object(ref mut map) = json_data {
        if map.get(new_key).is_some() {
            bail!("{name} is already held.");
        }
        map.insert(new_key.to_string(), Value::Bool(new_value));
        println!("{}", t!("hold.locked", name = name).dark_green().bold());
    } else {
        bail!("Invalid JSON: Expected an object");
    }
    std::fs::write(file_path, serde_json::to_string_pretty(&json_data)?)
        .context(format!("Failed to write file: {}", file_path))?;
    Ok(())
}

pub fn execute_hold_command(hold_args: HoldArgs) -> anyhow::Result<()> {
    if hold_args.app_names.is_none() {
        return Ok(());
    }
    let app_names = hold_args.app_names.unwrap();

    let result = app_names
        .iter()
        .filter_map(|name| {
            let install_json = if hold_args.global {
                get_app_dir_install_json_global(name)
            } else {
                let user_json = get_app_dir_install_json(name);
                if Path::new(&user_json).exists() {
                    user_json
                } else {
                    let global_json = get_app_dir_install_json_global(name);
                    if Path::new(&global_json).exists() {
                        global_json
                    } else {
                        user_json
                    }
                }
            };
            if !Path::new(&install_json).exists() {
                eprintln!(
                    "{}",
                    format!("'{name}' is not installed.").dark_yellow().bold()
                );
                None
            } else {
                if hold_args.cancel_hold {
                    let result = unhold_locked_apps(name, &install_json);
                    if result.is_err() { Some(result) } else { None }
                } else {
                    let result = add_key_value_to_json(&install_json, "hold", true, name);
                    if result.is_err() { Some(result) } else { None }
                }
            }
        })
        .collect::<Vec<_>>();
    if result.is_empty() {
        return Ok(());
    }
    result.iter().for_each(|result| match result {
        Ok(_) => {}
        Err(e) => {
            let e = e.to_string();
            eprintln!("{}", e.dark_red().bold());
        }
    });
    Ok(())
}

pub fn unhold_locked_apps(app_name: &str, install_json_file: &str) -> anyhow::Result<()> {
    let data = std::fs::read_to_string(install_json_file)
        .context(format!("Failed to read file: {}", install_json_file))?;

    let mut json_data: Value =
        serde_json::from_str(&data).context("Failed to parse JSON data in unhold_locked_apps")?;

    if let Value::Object(ref mut map) = json_data {
        if map.get("hold").is_none() {
            bail!("'{app_name}' is not held.");
        }
        map.remove("hold");
        println!(
            "{}",
            t!("hold.unlocked", name = app_name).dark_green().bold()
        );
    } else {
        bail!("Invalid JSON: Expected an object");
    }
    std::fs::write(install_json_file, serde_json::to_string_pretty(&json_data)?)
        .context(format!("Failed to write file: {}", install_json_file))?;
    Ok(())
}

pub fn show_reward_img() {
    use qrcode::QrCode;
    use qrcode::render::unicode;

    let url = "https://img.picui.cn/free/2025/05/04/68170e249fdcd.png";

    let code = QrCode::new(url).unwrap();
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();

    println!("{}", image);
    log::debug!(
        "{}",
        t!("credits.support_message").as_ref().dark_cyan().bold()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_multi_app_parsing() {
        // Install
        let cli = crate::Cli::try_parse_from(["hp", "install", "git", "curl", "7zip"]).unwrap();
        match cli.command.unwrap() {
            Commands::Install(args) => assert_eq!(args.app_names, vec!["git", "curl", "7zip"]),
            _ => panic!("Expected Install"),
        }

        // Uninstall
        let cli = crate::Cli::try_parse_from(["hp", "uninstall", "git", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Uninstall(args) => assert_eq!(args.app_names, vec!["git", "curl"]),
            _ => panic!("Expected Uninstall"),
        }

        // Update
        let cli = crate::Cli::try_parse_from(["hp", "update", "git", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Update(args) => assert_eq!(args.app_names, vec!["git", "curl"]),
            _ => panic!("Expected Update"),
        }

        // Reset
        let cli = crate::Cli::try_parse_from(["hp", "reset", "python@3.9", "nodejs"]).unwrap();
        match cli.command.unwrap() {
            Commands::Reset(args) => assert_eq!(args.names, vec!["python@3.9", "nodejs"]),
            _ => panic!("Expected Reset"),
        }

        // Cache rm
        let cli = crate::Cli::try_parse_from(["hp", "cache", "rm", "git", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Cache(args) => match args.command.unwrap() {
                crate::command_args::cache::CacheSubcommand::Rm(rm) => {
                    assert_eq!(rm.rm_apps, vec!["git", "curl"]);
                }
                _ => panic!("Expected Cache Rm"),
            },
            _ => panic!("Expected Cache"),
        }

        // Info
        let cli = crate::Cli::try_parse_from(["hp", "info", "git", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Info(args) => assert_eq!(args.names, vec!["git", "curl"]),
            _ => panic!("Expected Info"),
        }

        // Cat
        let cli = crate::Cli::try_parse_from(["hp", "cat", "git", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Cat(args) => assert_eq!(args.app_names, vec!["git", "curl"]),
            _ => panic!("Expected Cat"),
        }

        // Home
        let cli = crate::Cli::try_parse_from(["hp", "home", "git", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Home(args) => assert_eq!(args.names, vec!["git", "curl"]),
            _ => panic!("Expected Home"),
        }

        // Prefix
        let cli = crate::Cli::try_parse_from(["hp", "prefix", "git", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Prefix(args) => assert_eq!(args.names, vec!["git", "curl"]),
            _ => panic!("Expected Prefix"),
        }

        // Which
        let cli = crate::Cli::try_parse_from(["hp", "which", "git", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Which(args) => assert_eq!(args.names, vec!["git", "curl"]),
            _ => panic!("Expected Which"),
        }
    }

    #[test]
    fn test_single_app_parsing() {
        let cli = crate::Cli::try_parse_from(["hp", "install", "git"]).unwrap();
        match cli.command.unwrap() {
            Commands::Install(args) => assert_eq!(args.app_names, vec!["git"]),
            _ => panic!("Expected Install"),
        }

        let cli = crate::Cli::try_parse_from(["hp", "uninstall", "git"]).unwrap();
        match cli.command.unwrap() {
            Commands::Uninstall(args) => assert_eq!(args.app_names, vec!["git"]),
            _ => panic!("Expected Uninstall"),
        }

        let cli = crate::Cli::try_parse_from(["hp", "update", "git"]).unwrap();
        match cli.command.unwrap() {
            Commands::Update(args) => assert_eq!(args.app_names, vec!["git"]),
            _ => panic!("Expected Update"),
        }

        let cli = crate::Cli::try_parse_from(["hp", "update", "-a"]).unwrap();
        match cli.command.unwrap() {
            Commands::Update(args) => {
                assert!(args.all);
                assert!(args.app_names.is_empty());
            }
            _ => panic!("Expected Update"),
        }

        // Self-update without force
        let cli = crate::Cli::try_parse_from(["hp", "self-update"]).unwrap();
        match cli.command.unwrap() {
            Commands::SelfUpdate(args) => {
                assert!(!args.force_update_override);
            }
            _ => panic!("Expected SelfUpdate"),
        }

        // Self-update with -f
        let cli = crate::Cli::try_parse_from(["hp", "self-update", "-f"]).unwrap();
        match cli.command.unwrap() {
            Commands::SelfUpdate(args) => {
                assert!(args.force_update_override);
            }
            _ => panic!("Expected SelfUpdate"),
        }

        // Self-update with --force
        let cli = crate::Cli::try_parse_from(["hp", "self-update", "--force"]).unwrap();
        match cli.command.unwrap() {
            Commands::SelfUpdate(args) => {
                assert!(args.force_update_override);
            }
            _ => panic!("Expected SelfUpdate"),
        }

        // Self-update with --force-update-override
        let cli =
            crate::Cli::try_parse_from(["hp", "self-update", "--force-update-override"]).unwrap();
        match cli.command.unwrap() {
            Commands::SelfUpdate(args) => {
                assert!(args.force_update_override);
            }
            _ => panic!("Expected SelfUpdate"),
        }
    }
}
