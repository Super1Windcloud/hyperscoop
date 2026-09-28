use clap::{Args, Subcommand};
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Clone, Subcommand, Debug)]
pub enum CacheSubcommand {
    Show(ShowArgs),
    Rm(RmArgs),
}

#[derive(Debug, Clone, Args)]
#[command(about = crate::i18n::tr("Show download cache entries", "显示所有缓存"))]
pub struct ShowArgs {
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Debug, Clone, Args)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct RmArgs {
    #[arg(
        required = false,
        num_args = 0..,
        help = crate::i18n::tr("Remove cache for the provided app(s)", "删除指定 App 缓存（支持多参数）"),
        value_parser = clap_args_to_lowercase
    )]
    pub rm_apps: Vec<String>,
    #[arg(
        long,
        short = 'a',
        help = crate::i18n::tr(
            "Clear all cache entries (e.g. rm -a / --all / *)",
            "清理所有缓存，例如 rm -a / --all / *"
        ),
        alias = "*"
    )]
    pub all: bool,
    #[arg(from_global)]
    pub global: bool,
    #[arg(
        short = 'l',
        long,
        help = crate::i18n::tr("Set log level", "启动日志等级"),
        default_value = "4",
        value_name = "1-4"
    )]
    pub log_level: u8,
}

#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = crate::i18n::tr(
    "🎨\t\tShow or clean download cache",
    "🎨\t\t显示或清理下载缓存"
))]
#[command(override_usage = crate::i18n::tr(
    "hp cache show|rm [app(s)]",
    "hp cache show|rm [app(s)]"
))]
pub struct CacheArgs {
    #[clap(subcommand)]
    pub(crate) command: Option<CacheSubcommand>,
    #[arg(
        long,
        short = 'a',
        help = crate::i18n::tr(
            "Clear all cache entries (e.g. rm -a / --all / *)",
            "清理所有缓存，例如 rm -a / --all / *"
        ),
        alias = "*"
    )]
    pub all: bool,

    #[arg(
        short = 'l',
        long,
        help = crate::i18n::tr("Set log level", "启动日志等级"),
        default_value = "4",
        value_name = "1-4"
    )]
    pub log_level: u8,

    #[arg(from_global)]
    pub global: bool,
}
