use clap::Args;


#[derive(Args, Debug, Clone)]
#[command(about = "🏳️‍🌈          列出已安装的所有app")]
#[command(arg_required_else_help = false, subcommand_negates_reqs = true)]
#[command(  after_help = "支持使用模糊匹配,支持多参数查询 ,Example : hp list zig rust ")]
pub struct ListArgs {
  #[clap(required = false,  num_args =1.., help = "列出指定app,使用模糊匹配")]
  pub(crate) name: Option<Vec<String>>,
}
