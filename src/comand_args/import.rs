
use  clap::Args;


#[derive(Args, Debug)]
///⚽️         导入json文件下载列表中的APP
pub struct ImportArgs  {
  name: Option<String>,
}
