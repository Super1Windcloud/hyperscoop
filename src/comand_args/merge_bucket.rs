use clap::Args;


#[derive(Args, Debug)]
///🐧          移除buckets中冗余和错误的manifest文件
#[command(arg_required_else_help = false, subcommand_negates_reqs = true)]
#[command(no_binary_name = true)]
pub struct MergeArgs {}
