use clap::Args;


#[derive(Args, Debug)]
#[command(name = "bucket", alias = "u", about = "🍹\t\t更新指定APP或者hp与buckets,别名u")]
#[command(arg_required_else_help = true, after_help = " 只对官方维护的bucket和非大型Bucket进行更新\n (DEV-tools,scoopbucket,\
 scoop-cn,scoop-proxy-cn,scoop-apps,scoopbucket-third,ScoopMaster)")]
pub struct UpdateArgs {
  #[arg(required = false)]
  #[arg (help = "指定要更新的APP名称,仅单个更新")]
  pub(crate) app_name   : Option<String>,

  #[arg(short , long, help = "更新所有已安装APP")]
  pub  all : bool,

  #[arg(short = 's', long, help = "跳过下载文件哈希验证")]
  pub(crate) skip_hash_check: bool,

  #[arg(required = false ,short , long , help = "更新hp自身和所有buckets")]
  pub    update_self_and_buckets   : bool,

  #[arg(short = 'k', long, help = "跳过本地缓存，强制从远程源重新下载安装")]
  pub(crate) no_use_download_cache: bool,

  #[arg(short='i' ,   required = false , long, help="不自动下载manifest里的依赖,很大概率导致软件异常")]
  pub no_auto_download_dependencies: bool,

  #[arg (short='r' , long, help = "删除旧的App的安装目录,默认会保留旧版本目录\n")]
  pub     remove_old_app : bool,

  #[arg(from_global)]
  pub  global :bool

}
