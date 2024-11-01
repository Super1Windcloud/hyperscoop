use  clap::Args;


#[derive(Args, Debug)]
///🎨          显示或清理下载缓存
pub struct CacheArgs  {
  name: Option<String>,
}
