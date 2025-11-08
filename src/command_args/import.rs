use clap::Args;

#[derive(Args, Debug)]
#[clap(
    name = "import",
    about = crate::i18n::tr(
        "⚽\t\tImport the JSON config exported by hp export",
        "⚽\t\t导入通过 export 导出的 JSON 配置文件"
    )
)]
#[command(arg_required_else_help = true)]
pub struct ImportArgs {
    #[arg(
        help = crate::i18n::tr("Path to the JSON config file", "导入的 JSON 配置文件路径")
    )]
    pub(crate) path: Option<String>,
}
