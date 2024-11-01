
use  clap::Args;


#[derive(Args, Debug, Clone)]
///  列出已安装的所有app
pub struct ListArgs   {
  name: Option<String>,
}
