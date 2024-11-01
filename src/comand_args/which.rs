
use  clap::Args;


#[derive(Args, Debug)]
///🐸          打印指定APP的可执行文件路径
pub struct  WhichArgs             {
  name: Option<String>,
}
