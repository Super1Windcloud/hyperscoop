
use  clap::Args;


#[derive(Args, Debug)]
///☃️          卸载指定APP
pub struct  UninstallArgs           {
  name: Option<String>,
}
