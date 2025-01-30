use  clap::Args;


#[derive(Args, Debug)]
///🎠          清理移除旧版本的APP
#[clap(author, version, about="🎠  清理移除旧版本的APP,别名clean ", long_about = None)]
#[clap(arg_required_else_help = true)]
#[clap(alias = "clean")]
pub struct CleanupArgs  {  
  #[arg(short='a', long, help = "清理所有版本的APP旧版本,别名*")] 
  #[clap(alias = "*")]
  pub(crate) all : bool, 
  pub(crate) name: Option<String>,
}
