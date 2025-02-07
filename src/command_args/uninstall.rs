
use  clap::Args;


#[derive(Args, Debug)]
#[clap(author="superwindcloud", version, about="☃️          卸载指定APP", long_about = None)]
#[command(arg_required_else_help = true )]
pub struct  UninstallArgs           {
  #[arg(help = "eg: hp  uninstall  git")]
  pub(crate) app_name : Option<String>,
  #[arg(short, long, help = "是否删除持久化数据,$SCOOP/persist/<app>" , long_help="  scoop uninstall <app> --purge ")]
  pub(crate) purge : bool
}
