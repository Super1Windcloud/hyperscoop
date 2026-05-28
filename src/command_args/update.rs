use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[command(
    name = "bucket",
    alias = "u",
    about = crate::i18n::tr(
        "🍹\t\tUpdate apps or hp/buckets (alias: u)",
        "🍹\t\t更新指定 APP 或 hp 与 buckets，别名 u"
    )
)]
#[command(
    arg_required_else_help = true,
    after_help = crate::i18n::tr(
        "Only updates officially maintained or non-large buckets\n(DEV-tools, scoopbucket, scoop-cn, scoop-proxy-cn, scoop-apps, scoopbucket-third, ScoopMaster)",
        "只对官方维护的 bucket 和非大型 bucket 进行更新\n(DEV-tools、scoopbucket、scoop-cn、scoop-proxy-cn、scoop-apps、scoopbucket-third、ScoopMaster)"
    )
)]
pub struct UpdateArgs {
    #[arg(required = false)]
    #[arg(
        help = crate::i18n::tr(
            "Name of the app to update (single)",
            "指定要更新的 APP 名称，仅单个更新"
        ),
        value_parser = clap_args_to_lowercase
    )]
    pub(crate) app_name: Option<String>,

    #[arg(
        short,
        long,
        help = crate::i18n::tr("Update all installed apps", "更新所有已安装 APP")
    )]
    pub all: bool,

    #[arg(
        short = 's',
        long,
        help = crate::i18n::tr("Skip download hash verification", "跳过下载文件哈希验证")
    )]
    pub(crate) skip_hash_check: bool,

    #[arg(
        required = false,
        short,
        long,
        help = crate::i18n::tr("Update all buckets", "更新所有 buckets")
    )]
    pub update_self_and_buckets: bool,

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
        short = 'r',
        long,
        help = crate::i18n::tr(
            "Remove previous installed versions (keep by default)",
            "删除旧的 App 安装版本，默认保留旧目录"
        )
    )]
    pub remove_old_app: bool,

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
        short = 'S',
        long,
        help = crate::i18n::tr("Enable serial updates (default is parallel)", "启用串行更新，默认并行"),
        required = false
    )]
    pub serial_update: bool,

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
