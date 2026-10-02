use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug, Clone)]
#[clap(
    author = "superwindcloud",
    version,
    about = crate::i18n::tr(
        "♻️\t\tReinstall an app (equivalent to: hp install --force)",
        "♻️\t\t重新安装指定的 APP（等价于 hp install --force）"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct ReinstallArgs {
    #[arg(
        help = crate::i18n::tr(
            "App name(s) to reinstall (supports multiple apps)",
            "重新安装 APP 的名称，精准匹配，支持同时重装多个 APP"
        ),
        required = true,
        num_args = 1..,
        value_parser = clap_args_to_lowercase
    )]
    pub app_names: Vec<String>,

    #[arg(
        short,
        long,
        help = crate::i18n::tr("Skip download hash verification", "跳过下载文件的哈希校验")
    )]
    pub skip_download_hash_check: bool,

    #[arg(
        short = 'k',
        long,
        help = crate::i18n::tr(
            "Bypass local cache and force download from remote source",
            "跳过本地缓存，强制从远程源重新下载安装"
        )
    )]
    pub no_use_download_cache: bool,

    #[arg(
        short = 'i',
        long,
        help = crate::i18n::tr(
            "Do not auto-download manifest dependencies",
            "不自动下载 manifest 里的依赖"
        )
    )]
    pub no_auto_download_dependencies: bool,

    #[arg(
        short = 'u',
        long,
        help = crate::i18n::tr(
            "Update buckets before reinstalling",
            "安装前更新 bucket，默认不更新"
        )
    )]
    pub update_hp_and_buckets: bool,

    #[arg(
        short = 'a',
        long,
        help = crate::i18n::tr("Target architecture, when available", "指定安装架构（若支持）"),
        default_value = "64bit",
        value_name = "<32bit|64bit|arm64>",
        value_parser = clap_args_to_lowercase
    )]
    pub arch: Option<String>,

    #[arg(from_global)]
    pub global: bool,
}
