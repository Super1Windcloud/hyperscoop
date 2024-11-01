// 获取或设置配置文件

use  clap::Args;


#[derive(Args, Debug)]
///🐼          获取或设置配置文件
pub struct ConfigArgs  {
  name: Option<String>,
}
