use clap::Args;

#[derive(Args, Debug)]
#[clap(author, version, about="🍅\t\t检查已安装APP是否是最新版本", long_about = None)]
#[command(arg_required_else_help = false, subcommand_negates_reqs = true)]
#[command(no_binary_name = true)]
pub struct StatusArgs {
    #[arg(from_global)]
    pub global: bool,
}
