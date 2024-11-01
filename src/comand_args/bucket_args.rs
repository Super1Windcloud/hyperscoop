use  clap::Args;


#[derive(Args, Debug)]
///🔫          管理scoop 所有buckets
pub struct BucketArgs  {
  #[clap(short, long, help = "添加bucket")]
  #[clap( help = "         管理scoop 所有buckets")]
  name: Option<String>,
}
