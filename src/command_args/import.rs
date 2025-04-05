use clap::Args;


#[derive(Args, Debug)]
#[clap(name = "import", about = "⚽\t\t导入通过export导出的json配置文件")]
#[command(arg_required_else_help = true)]
pub struct ImportArgs {
  #[arg(help = "导入的json配置文件路径")]
  pub(crate) path  : Option<String>,
}


