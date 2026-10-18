use clap::Args;

#[derive(Args, Debug)]
#[clap(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🎅  Export installed apps and buckets to a JSON file",
        "🎅  导出已安装的 APP 和 bucket 列表为 JSON 文件"
    ),
    long_about = None
)]
pub struct ExportArgs {
    #[clap(
        help = crate::i18n::tr(
            "Target file name or path, e.g. export config.json (current dir) or export C:\\path\\export.json",
            "指定文件名或路径，例如 export config.json（当前目录）或 export C:\\path\\export.json"
        )
    )]
    pub(crate) file_name: Option<String>,
    #[clap(
        short,
        long,
        help = crate::i18n::tr("Export Scoop config as well", "一并导出 Scoop 配置文件")
    )]
    pub(crate) config: bool,
}
