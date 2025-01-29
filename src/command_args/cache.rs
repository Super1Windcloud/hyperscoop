use clap::{Args, Subcommand};
#[derive(Clone, Subcommand, Debug)]
pub enum CacheSubcommand {
    Show(ShowArgs),
    Rm(RmArgs),
}

#[derive(Debug, Clone, Args)]
///显示所有缓存
pub struct ShowArgs {} 

#[derive(Debug, Clone, Args)]
///删除指定缓存 
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct RmArgs {
  
    pub    rm_app: Option<String> ,
  #[arg(
    long,
    short = 'a',
    help = "清理所有缓存\t 例如 rm -a /--all /* ",
    alias = "*"
  )]
  pub all: bool,

  #[arg (short = 'l' , long, help = "启动日志等级", default_value = "4" , value_name = "1-4" )]
  pub  log_level :  u8 ,
}

#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = "🎨          显示或清理下载缓存 ")]
#[command(override_usage = " hp  cache show|rm [app(s)]")]
pub struct CacheArgs {
    #[clap(subcommand)]
    pub(crate) command: Option<CacheSubcommand>,
  #[arg(
    long,
    short = 'a',
    help = "清理所有缓存\t 例如 rm -a /--all /* ",
    alias = "*"
  )]
  pub all: bool,
  
  #[arg (short = 'l' , long, help = "启动日志等级", default_value = "4" , value_name = "1-4" )]
  pub  log_level :  u8 , 
}
