
use  clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;


#[derive(Args, Debug)]
#[clap(author, version, about="🍻\t\t切换指定的APP版本或重置最新版本", long_about = "None")]
#[command(arg_required_else_help = true)]
pub struct ResetArgs       { 
  #[arg(help = " APP名称, 示例: reset  python@3.9 or reset  python",required = false , 
  value_parser = clap_args_to_lowercase)]
  pub(crate) name: Option<String>,
  #[arg(from_global)]
  pub global: bool,
}
