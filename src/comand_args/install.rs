
use  clap::Args;


#[derive(Args, Debug)]
///🐘          安装指定APP
pub struct InstallArgs  {
  name: Option<String>,
}
