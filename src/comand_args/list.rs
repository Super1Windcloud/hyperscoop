use clap::Args;


#[derive(Args, Debug, Clone)]
#[command(about = "🏳️‍🌈       列出已安装的所有app")]
#[command(arg_required_else_help = false, subcommand_negates_reqs = true)]
pub struct ListArgs {
  #[clap(required = false, help = "搜索app的名称")]
  pub(crate) name: Option<String>,
}
