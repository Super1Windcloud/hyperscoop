
use  clap::Args;


#[derive(Args, Debug)]
///🍷          显示指定APP的信息
pub struct InfoArgs  {
  name: Option<String>,
}
