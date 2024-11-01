use  clap::Args;


#[derive(Args, Debug)]
///🎠          清理移除旧版本的APP
pub struct CleanupArgs  {
  name: Option<String>,
}
