use clap::Args;


#[derive(Args, Debug)]
///🦄          搜索可用的指定名称APP
#[command(arg_required_else_help = true)]
pub struct SearchArgs {
  #[clap(help = "搜索app的名称")]
  #[clap(required = false)]
  pub(crate) name: Option<String>,
}
