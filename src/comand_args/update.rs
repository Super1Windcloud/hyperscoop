use clap::Args;


#[derive(Args, Debug)]
///🍹          更新指定APP或者hyperscoop和buckets
pub struct UpdateArgs {
  #[arg(required = false)]
  #[clap(help = "指定要更新的APP名称,如果没有name参数,则更新scoop和buckets,\
  只对官方维护的bucket进行更新")]
  pub(crate) name: Option<String>,

  #[clap(short = 'k', long, help = "不使用下载缓存")]
  pub(crate) no_cache: bool,

  #[clap(short = 's', long, help = "跳过哈希验证")]
  pub(crate) skip_hash_check: bool,
}
