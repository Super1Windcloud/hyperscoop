use clap::Args;


#[derive(Args, Debug)]
#[command(name = "bucket", alias = "u", about = "🍹\t\t更新指定APP或者hp与buckets,别名u")]
#[command(arg_required_else_help = true, after_help = "只对官方维护的bucket进行更新, hp bucket known ")]
pub struct UpdateArgs {
  #[arg(required = false)]
  #[clap(help = "指定要更新的APP名称,仅单个更新")]
  pub(crate) app_name   : Option<String>,

  #[arg(short , long, help = "更新所有APP")]
  pub  all : bool,
  
  
  #[clap(short = 's', long, help = "跳过下载文件哈希验证")]
  pub(crate) skip_hash_check: bool,

  
  #[arg(required = false ,short , long , help = "更新hp自身和所有buckets")]
  pub    update_self  : bool,
  #[clap(short = 'k', long, help = "跳过本地缓存，强制从远程源重新下载安装")]
  pub(crate) no_use_download_cache: bool,

 
  #[arg (short='r' , long, help = "保留旧的App的安装目录,默认会删除旧版本目录\n")]
  pub    retain_out_app : bool,

  #[arg(from_global)]
  pub  global :bool

}
