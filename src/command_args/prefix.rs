
use  clap::Args;


#[derive(Args, Debug)]
#[clap(name = "prefix")]
#[clap(about = "👻\t\t打印指定APP的安装目录")]
#[clap(arg_required_else_help = true)]
pub struct PrefixArgs      {
  pub(crate) name: Option<String>,
  
  #[arg(from_global)]
  pub (crate) global: bool,
}
