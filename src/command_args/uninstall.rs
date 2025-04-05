
use  clap::Args;

#[derive(Args, Debug)]
#[clap(author="superwindcloud", version, about="⛄\t\t卸载指定APP", long_about = None)]
#[command(arg_required_else_help = true )]
pub struct  UninstallArgs           {
  #[arg(help = "卸载指定APP的名称,精准匹配,仅单个卸载")]
  pub(crate) app_name : Option<String>,
  #[arg(short, long, help = "是否删除持久化数据,$SCOOP/persist/<app>" , long_help="  scoop uninstall <app> --purge ")]
  pub(crate) purge : bool ,
  #[arg(from_global)]
  pub  global :bool ,
  #[arg(short ,long , help = "强制删除,尽管没有正确安装" )]
  pub force : bool,
}
