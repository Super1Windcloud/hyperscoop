
use  clap::Args;


#[derive(Args, Debug)]
///👻          打印指定APP的安装目录
pub struct PrefixArgs      {
  name: Option<String>,
}
