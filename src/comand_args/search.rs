
use  clap::Args;


#[derive(Args, Debug)]
///🦄          搜索可用的指定名称APP
pub struct SearchArgs        {
  name: Option<String>,
}
