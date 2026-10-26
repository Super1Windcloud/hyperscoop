use clap::{Args, Subcommand};
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Clone, Subcommand, Debug)]
pub enum CacheSubcommand {
    Show(ShowArgs),
    Rm(RmArgs),
    #[command(about = crate::i18n::tr("Remove expired download cache (default: 120 days)", "清理过期下载缓存（默认 120 天）"))]
    Prune(PruneArgs),
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
    #[arg(short = 'n', long, help = crate::i18n::tr("Preview cache removal", "预览缓存清理，不删除文件"))]
    pub dry_run: bool,
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

#[derive(Debug, Clone, Args)]
pub struct PruneArgs {
    #[arg(value_parser = clap_args_to_lowercase)]
    pub apps: Vec<String>,
    #[arg(long, default_value_t = 120, help = crate::i18n::tr("Remove packages older than this many days", "清理超过指定天数的安装包"))]
    pub days: u64,
    #[arg(short = 'n', long, help = crate::i18n::tr("Preview cache removal", "预览缓存清理，不删除文件"))]
    pub dry_run: bool,
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = crate::i18n::tr(
    "🎨  Show or clean download cache",
    "🎨  显示或清理下载缓存"
))]
#[command(override_usage = crate::i18n::tr(
    "hp cache show|rm|prune [app(s)]",
    "hp cache show|rm|prune [app(s)]"
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
