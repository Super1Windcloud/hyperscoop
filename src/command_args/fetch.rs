use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "📥\t\tDownload app package(s) into cache without installing (similar to brew fetch)",
        "📥\t\t下载安装包到本地缓存并校验哈希，不执行解压与安装 (类似 brew fetch)"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct FetchArgs {
    #[arg(
        help = crate::i18n::tr("App name(s) to fetch", "要下载缓存的 APP 名称，支持多个"),
        required = true,
        num_args = 1..,
        value_parser = clap_args_to_lowercase
    )]
    pub app_names: Vec<String>,

    #[arg(
        short = 's',
        long,
        help = crate::i18n::tr("Skip download hash verification", "跳过哈希校验")
    )]
    pub skip_download_hash_check: bool,

    #[arg(
        short = 'k',
        long,
        help = crate::i18n::tr("Force re-download and bypass existing cache", "忽略已有缓存，强制重新下载")
    )]
    pub no_use_download_cache: bool,

    #[arg(
        short = 'u',
        long,
        help = crate::i18n::tr("Update buckets before fetching", "下载前更新 bucket")
    )]
    pub update_hp_and_buckets: bool,

    #[arg(
        short = 'a',
        long,
        help = crate::i18n::tr("Architecture (64bit/32bit/arm64)", "指定架构 (64bit/32bit/arm64)")
    )]
    pub arch: Option<String>,

    #[arg(from_global)]
    pub global: bool,
}
