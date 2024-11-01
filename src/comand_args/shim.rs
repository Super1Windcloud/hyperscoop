
use  clap::Args;


#[derive(Args, Debug)]
///🐼          管理所有的shim快捷方式
pub struct ShimArgs         {
  name: Option<String>,
}
