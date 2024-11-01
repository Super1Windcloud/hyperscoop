use  clap::Args;


#[derive(Args, Debug)]
///🔫          管理scoop 所有bucket
#[clap(author, version, about, long_about = None)]
#[clap(override_usage = "子命令 _add|list|known|rm repo_name_")]
pub struct BucketArgs  {
  name: Option<String>,
}
