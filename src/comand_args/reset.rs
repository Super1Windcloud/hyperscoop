
use  clap::Args;


#[derive(Args, Debug)]
///🍻          切换指定的APP版本, 如果同app存在多版本
pub struct ResetArgs       {
  name: Option<String>,
}
