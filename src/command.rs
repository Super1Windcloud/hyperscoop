use crate::check_self_update::auto_check_hp_update;
use crate::command_args::alias::AliasArgs;
use crate::command_args::autoremove::AutoremoveArgs;
use crate::command_args::bundle::BundleArgs;
use crate::command_args::cat::CatArgs;
use crate::command_args::checkup::CheckupArgs;
use crate::command_args::cleanup::CleanupArgs;
use crate::command_args::completion::CompletionArgs;
use crate::command_args::config::ConfigArgs;
use crate::command_args::deps::DepsArgs;
use crate::command_args::desc::DescArgs;
use crate::command_args::edit::EditArgs;
use crate::command_args::export::ExportArgs;
use crate::command_args::fetch::FetchArgs;
use crate::command_args::home::HomeArgs;
use crate::command_args::import::ImportArgs;
use crate::command_args::info::InfoArgs;
use crate::command_args::install::InstallArgs;
use crate::command_args::leaves::LeavesArgs;
use crate::command_args::link::LinkArgs;
use crate::command_args::list::ListArgs;
use crate::command_args::log::LogArgs;
use crate::command_args::merge_bucket::MergeArgs;
use crate::command_args::missing::MissingArgs;
use crate::command_args::prefix::PrefixArgs;
use crate::command_args::reinstall::ReinstallArgs;
use crate::command_args::reset::ResetArgs;
use crate::command_args::search::SearchArgs;
use crate::command_args::self_update::SelfUpdateArgs;
use crate::command_args::service::ServiceArgs;
use crate::command_args::shellenv::ShellenvArgs;
use crate::command_args::shim::ShimArgs;
use crate::command_args::size::SizeArgs;
use crate::command_args::status::StatusArgs;
use crate::command_args::switch::SwitchArgs;
use crate::command_args::uninstall::UninstallArgs;
use crate::command_args::unlink::UnlinkArgs;
use crate::command_args::update::UpdateArgs;
use crate::command_args::uses::UsesArgs;
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
    Autoremove(AutoremoveArgs),
    Bucket(BucketArgs),
    Bundle(BundleArgs),
    Cat(CatArgs),
    Cache(CacheArgs),
    Checkup(CheckupArgs),
    Cleanup(CleanupArgs),
    Completion(CompletionArgs),
    Config(ConfigArgs),
    Deps(DepsArgs),
    Desc(DescArgs),
    Edit(EditArgs),
    Export(ExportArgs),
    #[clap(alias = "download")]
    Fetch(FetchArgs),
    Home(HomeArgs),
    #[clap(alias = "pin")]
    Hold(HoldArgs),
    Import(ImportArgs),
    Info(InfoArgs),
    Install(InstallArgs),
    Leaves(LeavesArgs),
    Link(LinkArgs),
    List(ListArgs),
    Log(LogArgs),
    Missing(MissingArgs),
    Prefix(PrefixArgs),
    #[clap(alias = "re")]
    Reinstall(ReinstallArgs),
    Reset(ResetArgs),
    #[clap(alias = "s")]
    Search(SearchArgs),
    SelfUpdate(SelfUpdateArgs),
    #[clap(alias = "services")]
    Service(ServiceArgs),
    Shellenv(ShellenvArgs),
    Shim(ShimArgs),
    Size(SizeArgs),
    Status(StatusArgs),
    Switch(SwitchArgs),
    #[clap(alias = "un")]
    Uninstall(UninstallArgs),
    #[clap(alias = "unhold")]
    Unpin(UnpinArgs),
    Unlink(UnlinkArgs),
    #[clap(alias = "upgrade")]
    Update(UpdateArgs),
    Uses(UsesArgs),
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

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🔓\t\tUnhold (unpin) apps to allow version updates",
        "🔓\t\t解除锁定（Unpin）指定 APP，允许后续版本更新"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct UnpinArgs {
    #[arg(
        required = true,
        num_args = 1..,
        help = crate::i18n::tr(
            "Names of the apps to unpin/unhold",
            "要解除锁定的 APP 名称"
        ),
        value_parser = clap_args_to_lowercase
    )]
    pub app_names: Vec<String>,
    #[arg(
        short = 'g',
        long,
        help = crate::i18n::tr("Unhold globally installed apps", "解除锁定全局安装的应用")
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
        .context(format!("Failed to read file: {}", file_path))?;

    let mut json_data: Value = serde_json::from_str(&data).context("Failed to parse JSON data")?;

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

        // Test doctor alias for checkup
        let cli = crate::Cli::try_parse_from(["hp", "doctor"]).unwrap();
        match cli.command.unwrap() {
            Commands::Checkup(_) => {}
            _ => panic!("Expected Checkup for 'doctor'"),
        }

        // Test outdated alias for status
        let cli = crate::Cli::try_parse_from(["hp", "outdated"]).unwrap();
        match cli.command.unwrap() {
            Commands::Status(_) => {}
            _ => panic!("Expected Status for 'outdated'"),
        }

        // Test completion parsing
        let cli = crate::Cli::try_parse_from(["hp", "completion", "powershell"]).unwrap();
        match cli.command.unwrap() {
            Commands::Completion(args) => {
                assert_eq!(args.shell, clap_complete::Shell::PowerShell);
            }
            _ => panic!("Expected Completion"),
        }

        // Test uses parsing
        let cli = crate::Cli::try_parse_from(["hp", "uses", "python"]).unwrap();
        match cli.command.unwrap() {
            Commands::Uses(args) => {
                assert_eq!(args.app_name, "python");
                assert!(!args.all);
            }
            _ => panic!("Expected Uses"),
        }

        let cli = crate::Cli::try_parse_from(["hp", "uses", "git", "--all"]).unwrap();
        match cli.command.unwrap() {
            Commands::Uses(args) => {
                assert_eq!(args.app_name, "git");
                assert!(args.all);
            }
            _ => panic!("Expected Uses with --all"),
        }

        // Test deps parsing
        let cli = crate::Cli::try_parse_from(["hp", "deps", "neovim", "--tree"]).unwrap();
        match cli.command.unwrap() {
            Commands::Deps(args) => {
                assert_eq!(args.app_name, "neovim");
                assert!(args.tree);
            }
            _ => panic!("Expected Deps with --tree"),
        }

        // Test bundle dump parsing
        let cli = crate::Cli::try_parse_from([
            "hp",
            "bundle",
            "dump",
            "--file",
            "CustomHpfile",
            "--force",
        ])
        .unwrap();
        match cli.command.unwrap() {
            Commands::Bundle(args) => match args.command {
                crate::command_args::bundle::BundleSubcommands::Dump(dump_args) => {
                    assert_eq!(dump_args.file, "CustomHpfile");
                    assert!(dump_args.force);
                }
                _ => panic!("Expected BundleSubcommands::Dump"),
            },
            _ => panic!("Expected Bundle"),
        }

        // Test bundle check parsing
        let cli = crate::Cli::try_parse_from(["hp", "bundle", "check"]).unwrap();
        match cli.command.unwrap() {
            Commands::Bundle(args) => match args.command {
                crate::command_args::bundle::BundleSubcommands::Check(check_args) => {
                    assert_eq!(check_args.file, "Hpfile");
                }
                _ => panic!("Expected BundleSubcommands::Check"),
            },
            _ => panic!("Expected Bundle"),
        }

        // Test pin alias for hold
        let cli = crate::Cli::try_parse_from(["hp", "pin", "python"]).unwrap();
        match cli.command.unwrap() {
            Commands::Hold(args) => {
                assert_eq!(args.app_names.unwrap(), vec!["python"]);
                assert!(!args.cancel_hold);
            }
            _ => panic!("Expected Hold for 'pin'"),
        }

        // Test unpin command
        let cli = crate::Cli::try_parse_from(["hp", "unpin", "python"]).unwrap();
        match cli.command.unwrap() {
            Commands::Unpin(args) => {
                assert_eq!(args.app_names, vec!["python"]);
            }
            _ => panic!("Expected Unpin"),
        }

        // Test unhold alias for unpin
        let cli = crate::Cli::try_parse_from(["hp", "unhold", "python"]).unwrap();
        match cli.command.unwrap() {
            Commands::Unpin(args) => {
                assert_eq!(args.app_names, vec!["python"]);
            }
            _ => panic!("Expected Unpin for 'unhold'"),
        }

        // Test leaves parsing
        let cli = crate::Cli::try_parse_from(["hp", "leaves"]).unwrap();
        match cli.command.unwrap() {
            Commands::Leaves(_) => {}
            _ => panic!("Expected Leaves"),
        }

        // Test size parsing and disk-usage alias
        let cli = crate::Cli::try_parse_from(["hp", "size", "git"]).unwrap();
        match cli.command.unwrap() {
            Commands::Size(args) => {
                assert_eq!(args.app_name.as_deref(), Some("git"));
            }
            _ => panic!("Expected Size"),
        }

        let cli = crate::Cli::try_parse_from(["hp", "disk-usage"]).unwrap();
        match cli.command.unwrap() {
            Commands::Size(args) => {
                assert_eq!(args.app_name, None);
            }
            _ => panic!("Expected Size for 'disk-usage'"),
        }

        // Test autoremove parsing
        let cli = crate::Cli::try_parse_from(["hp", "autoremove", "-n", "-y"]).unwrap();
        match cli.command.unwrap() {
            Commands::Autoremove(args) => {
                assert!(args.dry_run);
                assert!(args.yes);
            }
            _ => panic!("Expected Autoremove"),
        }

        // Test reinstall parsing
        let cli = crate::Cli::try_parse_from(["hp", "reinstall", "git", "-k"]).unwrap();
        match cli.command.unwrap() {
            Commands::Reinstall(args) => {
                assert_eq!(args.app_names, vec!["git"]);
                assert!(args.no_use_download_cache);
            }
            _ => panic!("Expected Reinstall"),
        }

        // Test re alias for reinstall
        let cli = crate::Cli::try_parse_from(["hp", "re", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Reinstall(args) => {
                assert_eq!(args.app_names, vec!["curl"]);
            }
            _ => panic!("Expected Reinstall for 're' alias"),
        }

        // Test cleanup --dry-run parsing
        let cli = crate::Cli::try_parse_from(["hp", "cleanup", "-a", "-n"]).unwrap();
        match cli.command.unwrap() {
            Commands::Cleanup(args) => {
                assert!(args.all);
                assert!(args.dry_run);
            }
            _ => panic!("Expected Cleanup"),
        }

        // Test uninstall --dry-run parsing
        let cli = crate::Cli::try_parse_from(["hp", "uninstall", "git", "-n"]).unwrap();
        match cli.command.unwrap() {
            Commands::Uninstall(args) => {
                assert_eq!(args.app_names, vec!["git"]);
                assert!(args.dry_run);
            }
            _ => panic!("Expected Uninstall"),
        }

        // Test bundle cleanup parsing
        let cli = crate::Cli::try_parse_from(["hp", "bundle", "cleanup", "--force"]).unwrap();
        match cli.command.unwrap() {
            Commands::Bundle(args) => match args.command {
                crate::command_args::bundle::BundleSubcommands::Cleanup(c_args) => {
                    assert!(c_args.force);
                }
                _ => panic!("Expected BundleSubcommands::Cleanup"),
            },
            _ => panic!("Expected Bundle"),
        }

        // Test edit parsing
        let cli = crate::Cli::try_parse_from(["hp", "edit", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Edit(args) => {
                assert_eq!(args.app_name, "curl");
            }
            _ => panic!("Expected Edit"),
        }

        // Test log parsing
        let cli = crate::Cli::try_parse_from(["hp", "log", "git", "-n", "5"]).unwrap();
        match cli.command.unwrap() {
            Commands::Log(args) => {
                assert_eq!(args.app_name, "git");
                assert_eq!(args.max_count, Some(5));
            }
            _ => panic!("Expected Log"),
        }

        // Test shellenv parsing
        let cli = crate::Cli::try_parse_from(["hp", "shellenv", "zsh"]).unwrap();
        match cli.command.unwrap() {
            Commands::Shellenv(args) => {
                assert_eq!(args.shell, Some("zsh".to_string()));
            }
            _ => panic!("Expected Shellenv"),
        }

        // Test service parsing and alias services
        let cli = crate::Cli::try_parse_from(["hp", "service", "start", "redis"]).unwrap();
        match cli.command.unwrap() {
            Commands::Service(args) => match args.command {
                Some(crate::command_args::service::ServiceSubcommands::Start { name }) => {
                    assert_eq!(name, "redis");
                }
                _ => panic!("Expected ServiceSubcommands::Start"),
            },
            _ => panic!("Expected Service"),
        }

        let cli = crate::Cli::try_parse_from(["hp", "services", "list"]).unwrap();
        match cli.command.unwrap() {
            Commands::Service(args) => match args.command {
                Some(crate::command_args::service::ServiceSubcommands::List) => {}
                _ => panic!("Expected ServiceSubcommands::List"),
            },
            _ => panic!("Expected Service for 'services' alias"),
        }

        // Test upgrade alias and -c/--cleanup
        let cli = crate::Cli::try_parse_from(["hp", "upgrade", "git", "-c"]).unwrap();
        match cli.command.unwrap() {
            Commands::Update(args) => {
                assert_eq!(args.app_names, vec!["git"]);
                assert!(args.remove_old_app);
            }
            _ => panic!("Expected Update for 'upgrade' alias"),
        }

        // Test switch parsing
        let cli = crate::Cli::try_parse_from(["hp", "switch", "node", "20.9.0"]).unwrap();
        match cli.command.unwrap() {
            Commands::Switch(args) => {
                assert_eq!(args.app_name, "node");
                assert_eq!(args.version, Some("20.9.0".to_string()));
            }
            _ => panic!("Expected Switch"),
        }

        // Test link parsing
        let cli = crate::Cli::try_parse_from(["hp", "link", "git"]).unwrap();
        match cli.command.unwrap() {
            Commands::Link(args) => {
                assert_eq!(args.app_names, vec!["git"]);
            }
            _ => panic!("Expected Link"),
        }

        // Test unlink parsing
        let cli = crate::Cli::try_parse_from(["hp", "unlink", "git"]).unwrap();
        match cli.command.unwrap() {
            Commands::Unlink(args) => {
                assert_eq!(args.app_names, vec!["git"]);
            }
            _ => panic!("Expected Unlink"),
        }

        // Test fetch parsing and download alias
        let cli = crate::Cli::try_parse_from(["hp", "fetch", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Fetch(args) => {
                assert_eq!(args.app_names, vec!["curl"]);
            }
            _ => panic!("Expected Fetch"),
        }

        let cli = crate::Cli::try_parse_from(["hp", "download", "curl"]).unwrap();
        match cli.command.unwrap() {
            Commands::Fetch(args) => {
                assert_eq!(args.app_names, vec!["curl"]);
            }
            _ => panic!("Expected Fetch for 'download' alias"),
        }

        // Test missing parsing
        let cli = crate::Cli::try_parse_from(["hp", "missing"]).unwrap();
        match cli.command.unwrap() {
            Commands::Missing(args) => {
                assert_eq!(args.app_name, None);
            }
            _ => panic!("Expected Missing"),
        }

        // Test desc parsing
        let cli = crate::Cli::try_parse_from(["hp", "desc", "git"]).unwrap();
        match cli.command.unwrap() {
            Commands::Desc(args) => {
                assert_eq!(args.query, "git");
                assert!(!args.search);
            }
            _ => panic!("Expected Desc"),
        }

        // Test install flags: --dry-run (-n), --force (-f), --no-deps, --ignore-dependencies
        let cli = crate::Cli::try_parse_from(["hp", "install", "git", "-n"]).unwrap();
        match cli.command.unwrap() {
            Commands::Install(args) => {
                assert_eq!(args.app_names, vec!["git"]);
                assert!(args.dry_run);
            }
            _ => panic!("Expected Install"),
        }

        let cli = crate::Cli::try_parse_from(["hp", "install", "git", "--force"]).unwrap();
        match cli.command.unwrap() {
            Commands::Install(args) => {
                assert_eq!(args.app_names, vec!["git"]);
                assert!(args.force_install_override);
            }
            _ => panic!("Expected Install"),
        }

        let cli = crate::Cli::try_parse_from(["hp", "install", "git", "--no-deps"]).unwrap();
        match cli.command.unwrap() {
            Commands::Install(args) => {
                assert_eq!(args.app_names, vec!["git"]);
                assert!(args.no_auto_download_dependencies);
            }
            _ => panic!("Expected Install"),
        }

        let cli =
            crate::Cli::try_parse_from(["hp", "install", "git", "--ignore-dependencies"]).unwrap();
        match cli.command.unwrap() {
            Commands::Install(args) => {
                assert_eq!(args.app_names, vec!["git"]);
                assert!(args.no_auto_download_dependencies);
            }
            _ => panic!("Expected Install"),
        }

        let cli = crate::Cli::try_parse_from(["hp", "install", "git", "--no-auto-update"]).unwrap();
        match cli.command.unwrap() {
            Commands::Install(args) => {
                assert_eq!(args.app_names, vec!["git"]);
                assert!(args.no_auto_update);
            }
            _ => panic!("Expected Install"),
        }

        let cli = crate::Cli::try_parse_from(["hp", "install", "git", "--no-upgrade"]).unwrap();
        match cli.command.unwrap() {
            Commands::Install(args) => {
                assert_eq!(args.app_names, vec!["git"]);
                assert!(args.no_upgrade);
            }
            _ => panic!("Expected Install"),
        }
    }
}
