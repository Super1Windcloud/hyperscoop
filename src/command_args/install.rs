use clap::ArgAction;
use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[command(
    name = "install",
    alias = "i",
    about = crate::i18n::tr(
        "🐘\t\tInstall an app (alias: i)",
        "🐘\t\t安装指定 APP，别名 i"
    )
)]
#[clap(author = "superwindcloud", version, long_about = None)]
#[command(arg_required_else_help = true)]
#[command(after_help = crate::i18n::tr(
    r#"Examples:
- Install from local buckets: hp install git
- Install from a specific bucket manifest: hp install main/genact
- Install another version when manifests exist: hp install gh@2.7.0
- Install from a local manifest path: hp install \path\to\app.json
- Install from a remote URL: hp install https://example.com/app.exe (supports .cmd, .bat, .ps1, .exe; installer-style exe packages are unsupported)
"#,
    r#"示例:
- 使用本地 bucket 安装: hp install git
- 指定 bucket 清单安装: hp install main/genact
- 安装清单中提供的其他版本: hp install gh@2.7.0
- 从本地清单路径安装: hp install \path\to\app.json
- 从远程 URL 安装: hp install https://example.com/app.exe（支持 .cmd/.bat/.ps1/.exe，安装包式 exe 无效）
"#
))]
pub struct InstallArgs {
    #[arg(
        help = crate::i18n::tr(
            "App name to install (exact match, single install)",
            "安装 APP 的名称，精准匹配，仅单个安装"
        ),
        required = false,
        value_parser = clap_args_to_lowercase
    )]
    pub app_name: Option<String>,

    #[arg(
        short,
        long,
        help = crate::i18n::tr("Skip download hash verification", "跳过下载文件的哈希校验"),
        required = false,
        action = ArgAction::SetTrue,
        help_heading = crate::i18n::tr("Install Options", "安装选项")
    )]
    pub skip_download_hash_check: bool,

    #[arg(
        short = 'k',
        long,
        help = crate::i18n::tr(
            "Bypass local cache and force download from remote source",
            "跳过本地缓存，强制从远程源重新下载安装"
        ),
        required = false,
        action = ArgAction::SetTrue,
        help_heading = crate::i18n::tr("Install Options", "安装选项")
    )]
    pub no_use_download_cache: bool,
    #[arg(
        short = 'i',
        long,
        help = crate::i18n::tr(
            "Do not auto-download manifest dependencies (likely to break apps)",
            "不自动下载 manifest 里的依赖，极易导致软件异常"
        ),
        required = false,
        action = ArgAction::SetTrue,
        help_heading = crate::i18n::tr("Install Options", "安装选项")
    )]
    pub no_auto_download_dependencies: bool,

    #[arg(
        short = 'o',
        long,
        help = crate::i18n::tr(
            "Download into cache and verify hash without installing",
            "下载文件到缓存并校验哈希，不执行安装"
        ),
        required = false,
        action = ArgAction::SetTrue,
        help_heading = crate::i18n::tr("Install Options", "安装选项")
    )]
    pub only_download_no_install: bool,

    #[arg(
        long,
        help = crate::i18n::tr(
            "Force download into cache, verify hash, skip install, overwrite cache",
            "强制下载文件到缓存并校验哈希，不执行安装，自动覆盖缓存"
        ),
        required = false,
        action = ArgAction::SetTrue,
        help_heading = crate::i18n::tr("Install Options", "安装选项")
    )]
    pub only_download_no_install_with_override_cache: bool,

    #[arg(
        short = 'u',
        long,
        help = crate::i18n::tr(
            "Update hp and buckets before installing",
            "安装前更新 hp 和 bucket，默认不更新"
        ),
        required = false,
        action = ArgAction::SetTrue,
        help_heading = crate::i18n::tr("Install Options", "安装选项")
    )]
    pub update_hp_and_buckets: bool,
    #[arg(
        short = 'c',
        long,
        help = crate::i18n::tr(
            "Check hp version before installing",
            "安装前检查 hp 版本是否最新，默认不检查"
        ),
        required = false,
        action = ArgAction::SetTrue,
        help_heading = crate::i18n::tr("Install Options", "安装选项")
    )]
    pub check_version_up_to_date: bool,

    #[arg(
        short = 'I',
        long,
        help = crate::i18n::tr("Interactive install mode", "交互式安装模式"),
        required = false,
        action = ArgAction::SetTrue,
        help_heading = crate::i18n::tr("Install Options", "安装选项")
    )]
    pub interactive: bool,

    #[arg(
        short,
        long,
        help_heading = crate::i18n::tr("Install Options", "安装选项"),
        required = false,
        help = crate::i18n::tr(
            "Force reinstall by deleting the existing directory first",
            "强制覆盖安装，先删除已安装目录"
        )
    )]
    pub force_install_override: bool,
    #[arg(
        short = 'a',
        long,
        help = crate::i18n::tr("Target architecture, when available", "指定安装架构（若支持）"),
        help_heading = crate::i18n::tr("Install Options", "安装选项"),
        required = false,
        default_value = "64bit",
        value_name = "<32bit|64bit|arm64>",
       value_parser = clap_args_to_lowercase
    )]
    pub arch: Option<String>,

    #[arg(
        short = 'A',
        long,
        help = crate::i18n::tr(
            "Alias to register for apps installed via URL (remote URL installs only)",
            "为从 URL 安装的 App 指定别名，仅适用于远程 URL 安装"
        ),
        help_heading = crate::i18n::tr("Install Options", "安装选项"),
        required = false,
        value_parser = clap_args_to_lowercase
    )]
    pub app_alias_from_url_install: Option<String>,

    #[arg(from_global)]
    pub global: bool,
}
