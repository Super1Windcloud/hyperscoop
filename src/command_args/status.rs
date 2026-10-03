use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🍅\t\tCheck whether installed apps are up-to-date (alias: outdated)",
        "🍅\t\t检查已安装 APP 是否为最新版本，别名 outdated"
    ),
    long_about = None
)]
#[command(arg_required_else_help = false, subcommand_negates_reqs = true)]
#[command(no_binary_name = true)]
#[clap(alias = "outdated")]
pub struct StatusArgs {
    #[arg(
        help = crate::i18n::tr(
            "Optional app name(s) to check status for",
            "可选的待检查应用名称（支持多个）"
        ),
        required = false,
        num_args = 0..,
        value_parser = clap_args_to_lowercase
    )]
    pub app_names: Vec<String>,

    #[arg(
        short = 'q',
        long,
        help = crate::i18n::tr(
            "Only display names of outdated apps (useful for piping into upgrade)",
            "仅显示有更新的应用名称（适合管道传参给 upgrade）"
        ),
        required = false
    )]
    pub quiet: bool,

    #[arg(
        long,
        help = crate::i18n::tr("Output status in JSON format", "以 JSON 格式输出状态信息"),
        required = false
    )]
    pub json: bool,

    #[arg(from_global)]
    pub global: bool,
}
