use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🧹  Remove leftover error directories inside app folders",
        "🧹  自动清理 APP 残留错误目录"
    ),
    long_about = crate::i18n::tr(
        "Scan app folders and remove directories that are not installed versions.\nKeeps current, directory links, and any version that still has install.json.\nOld installed versions stay for 'hp cleanup'. Empty app folders are removed.\nThe scoop app itself is never touched.",
        "扫描 APP 目录，删除不属于已安装版本的残留文件夹。\n保留 current、目录链接，以及仍带有 install.json 的版本目录。\n已安装的旧版本交给 hp cleanup 处理。空的 APP 目录会被删除。\n不会改动 scoop 自身。"
    )
)]
#[command(arg_required_else_help = false)]
pub struct PurgeArgs {
    #[arg(
        required = false,
        num_args = 1..,
        help = crate::i18n::tr(
            "App names to scan (default: every installed app)",
            "要扫描的 APP 名称；省略时扫描全部已安装应用"
        ),
        value_parser = clap_args_to_lowercase
    )]
    pub(crate) app_names: Option<Vec<String>>,

    #[arg(from_global)]
    pub global: bool,

    #[arg(
        short = 'n',
        long,
        help = crate::i18n::tr(
            "Show leftover directories that would be removed without deleting them",
            "演练模式：仅展示将要清理的残留目录，不实际删除"
        )
    )]
    pub dry_run: bool,
}
