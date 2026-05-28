use clap::Args;

#[derive(Args, Debug)]
#[command(
    name = "self-update",
    about = crate::i18n::tr("🍹\t\tUpdate hp itself", "🍹\t\t更新 hp 自身")
)]
#[command(arg_required_else_help = false)]
pub struct SelfUpdateArgs {
    #[arg(
        short = 's',
        long,
        help = crate::i18n::tr("Skip download hash verification", "跳过下载文件哈希验证")
    )]
    pub(crate) skip_hash_check: bool,

    #[arg(
        short = 'k',
        long,
        help = crate::i18n::tr(
            "Bypass cache and re-download from remote source",
            "跳过本地缓存，强制从远程源重新下载安装"
        )
    )]
    pub(crate) no_use_download_cache: bool,

    #[arg(
        short = 'i',
        required = false,
        long,
        help = crate::i18n::tr(
            "Do not auto-download manifest dependencies (likely to break apps)",
            "不自动下载 manifest 里的依赖，很大概率导致软件异常"
        )
    )]
    pub no_auto_download_dependencies: bool,

    #[arg(
        short = 'f',
        long,
        help = crate::i18n::tr(
            "Force update and clean if the current install is broken",
            "当前版本安装错误时强制更新并删除错误安装"
        )
    )]
    pub force_update_override: bool,

    #[arg(
        short = 'I',
        long,
        help = crate::i18n::tr("Interactive install mode", "交互式安装，默认关闭"),
        required = false
    )]
    pub interactive: bool,

    #[arg(from_global)]
    pub global: bool,
}
