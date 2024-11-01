
use  clap::Args;


#[derive(Args, Debug)]
///🎅          导出已安装的APP和bucket列表为json格式文件
pub struct ExportArgs  {
  name: Option<String>,
}
