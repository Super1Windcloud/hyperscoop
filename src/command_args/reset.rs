
use  clap::Args;


#[derive(Args, Debug)]
#[clap(author, version, about="🍻          切换指定的APP版本或重置最新版本", long_about = "None")]
#[command(arg_required_else_help = true)]
pub struct ResetArgs       { 
  #[arg(help = " APP名称, 示例: reset  python@3.9 or reset  python")]
  pub(crate) name: Option<String>,
}
