use clap::Args;


#[derive(Args, Debug, Clone)]
#[command(about = "🏳️‍🌈       列出已安装的所有app")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct ListArgs {
  name: Option<String>,
}
