use clap::Args;

#[derive(Args, Debug)]
#[clap(author, version, about="🐧\t\t移除buckets中冗余和错误的manifest文件", long_about = None)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(no_binary_name = true)]
#[command(after_help = "只会操作社区的bucket, 忽略scoop官方的bucket, hp bucket known ")]
pub struct MergeArgs {
    #[arg(short = 'e', long, help = "移除buckets中格式错误的manifest文件")]
    pub rm_err_manifest: bool,

    #[arg(
        short = 'r',
        long,
        help = "移除buckets中冗余的manifest文件",
        help_heading = "仅超大型社区桶"
    )]
    pub rm_redundant_manifest: bool,
}
