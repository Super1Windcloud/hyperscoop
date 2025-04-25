use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;


#[derive(Args, Debug, Clone)]
#[command(about = "🦀\t\t列出已安装的所有app")]
#[command(arg_required_else_help = false, subcommand_negates_reqs = true)]
#[command(  after_help = "支持使用模糊匹配,支持多参数查询 ,Example : hp list zig rust ")]
pub struct ListArgs {
  #[clap(required = false,  num_args =1.., help = "列出指定app,使用模糊匹配", 
  value_parser = clap_args_to_lowercase )]
  pub(crate) name: Option<Vec<String>>,
  
   #[arg(from_global)]
  pub  global: bool,
}
