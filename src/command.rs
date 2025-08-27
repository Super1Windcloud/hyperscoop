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
use crate::command_args::shim::ShimArgs;
use crate::command_args::status::StatusArgs;
use crate::command_args::uninstall::UninstallArgs;
use crate::command_args::update::UpdateArgs;
use crate::command_args::which::WhichArgs;
pub(crate) use crate::command_args::{bucket_args::BucketArgs, cache::CacheArgs};
use anyhow::{bail, Context};
use clap::{Args, Subcommand};
use command_util_lib::init_env::get_app_dir_install_json;
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
#[clap(author, version, about="💖\t\t显示Credit信息", long_about = None)]
#[command(arg_required_else_help = false, subcommand_negates_reqs = true)]
#[command(no_binary_name = true)]
pub struct CreditsArgs {}

pub async fn execute_credits_command() -> anyhow::Result<()> {
    if !auto_check_hp_update(None).await? {
        println!(
            "{}",
            "💖\tNow hp's  version is latest! Please enjoy it!"
                .dark_cyan()
                .bold()
        );
    };

    let str = "Hp  is created by superwindcloud(https://gitee.com/superwindcloud)(https://github.com/super1windcloud)"
        .to_string()
        .dark_blue()
        .bold();
    println!("💖\t{str}");

    show_reward_img();
    Ok(())
}

#[derive(Args, Debug)]
#[clap(author, version, about="💖\t\t锁定指定APP版本,锁定之后更新所有APP或者检测更新状态将自动跳过", long_about = None)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(no_binary_name = true)]
pub struct HoldArgs {
    #[arg( required = false,  num_args =1.., help = "要锁定的APP名称,精准匹配,支持多参数"
    ,value_parser = clap_args_to_lowercase)]
    pub app_names: Option<Vec<String>>,
    #[arg(short = 'u', long, required = false, help = "取消锁定, 支持多参数")]
    pub cancel_hold: bool,
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
        println!(
            "{}",
            (name.to_string() + " is now held and can not be updated anymore.")
                .dark_green()
                .bold()
        );
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
            let install_json = get_app_dir_install_json(name);
            if !Path::new(&install_json).exists() {
                eprintln!("{install_json} 不存在");
                None
            } else {
                if hold_args.cancel_hold {
                    let result = unhold_locked_apps(&name, &install_json);
                    if result.is_err() {
                        Some(result)
                    } else {
                        None
                    }
                } else {
                    let result = add_key_value_to_json(&install_json, "hold".as_ref(), true, name);
                    if result.is_err() {
                        Some(result)
                    } else {
                        None
                    }
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
            eprintln!("{}", e.dark_grey().bold());
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
            bail!("'{app_name}' is not  held.");
        }
        map.remove("hold");
        println!(
            "{}",
            (app_name.to_string() + " is no longer held and can be updated again.")
                .dark_green()
                .bold()
        );
    } else {
        bail!("Invalid JSON: Expected an object");
    }
    std::fs::write(install_json_file, serde_json::to_string_pretty(&json_data)?)
        .context(format!("Failed to write file: {}", install_json_file))?;
    Ok(())
}

pub fn show_reward_img() {
    use qrcode::render::unicode;
    use qrcode::QrCode;

    let url = "https://img.picui.cn/free/2025/05/04/68170e249fdcd.png";

    let code = QrCode::new(url).unwrap();
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();

    println!("{}", image);
    println!(
        "{}",
        "您的支持是我调试人生的 print!   │───┘".dark_cyan().bold()
    );
}
