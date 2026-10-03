use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🔀\t\tSwitch active version of an installed app (similar to brew switch)",
        "🔀\t\t在已安装的多个版本之间切换活动版本 (类似 brew switch)"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true, disable_version_flag = true)]
pub struct SwitchArgs {
    #[arg(
        help = crate::i18n::tr("App name to switch, e.g. nodejs, python", "要切换版本的 APP 名称，如 nodejs, python"),
        value_parser = clap_args_to_lowercase
    )]
    pub app_name: String,

    #[arg(
        help = crate::i18n::tr("Target version to switch to (omit to list installed versions)", "要切换的目标版本号（不传则列出所有已安装版本）"),
        required = false
    )]
    pub version: Option<String>,

    #[arg(from_global)]
    pub global: bool,
}
