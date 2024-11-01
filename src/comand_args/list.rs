
use  clap::Args;


#[derive(Args, Debug)]
///🏳️‍🌈       列出已安装的所有app
pub struct ListArgs   {
  name: Option<String>,
}
