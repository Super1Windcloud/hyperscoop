use clap::Args;

#[derive(Args, Debug)]
#[clap(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[clap(author, version, about="🎅\t\t导出已安装的APP和bucket列表为json格式文件", long_about = None)]
pub struct ExportArgs {
    #[clap(
        help = "指定文件名或路径,例如: export config.json(当前目录) |  export  C:\\path\\export.json"
    )]
    pub(crate) file_name: Option<String>,
    #[clap(short, long, help = "一并导出Scoop配置文件")]
    pub(crate) config: bool,
}
