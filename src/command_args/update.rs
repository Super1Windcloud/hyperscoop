use clap::Args;


#[derive(Args, Debug)]
#[command(name = "bucket", alias = "u", about = "🍹          更新指定APP或者hp与buckets,别名u")]
#[command(arg_required_else_help = true, after_help = "只对官方维护的bucket进行更新, hp bucket known ")]
pub struct UpdateArgs {
  #[arg(required = false)]
  #[clap(help = "指定要更新的APP名称")]
  pub(crate) app_name   : Option<String>,
  #[arg(required = false ,short , long , help = "更新hp和buckets")]
  pub    update_self  : bool,
  #[clap(short = 'k', long, help = "不使用下载缓存")]
  pub(crate) no_cache: bool,

  #[clap(short = 's', long, help = "跳过哈希验证")]
  pub(crate) skip_hash_check: bool,

  #[arg(short , long, help = "更新所有APP")]
  pub  all : bool,
}
