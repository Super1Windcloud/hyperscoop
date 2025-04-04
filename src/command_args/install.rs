
use  clap::Args;


#[derive(Args, Debug)]
#[command(name = "install/i", alias = "i",  about = "🐘          安装指定APP,别名i")]
#[clap(author="superwindcloud", version="114514" , long_about = None)]
#[command(arg_required_else_help = true)]
#[command(after_help = r#"
e.g. 安装应用程序的通常方法（使用您的本地buckets）： hp install git

指定特定buckets的清单中安装:   hp install  main/genact

安装应用程序的不同版本,如果存在多版本清单 :  hp install gh@2.7.0

从计算机上的指定路径清单中安装应用程序 :   hp install \path\to\app.json
     "#)]
pub struct InstallArgs  {
  #[arg(help = "下载APP的名称", required = false )]
  pub  app_name: Option<String>,


  #[arg(short='k' , long, help = "不启用下载缓存", required = false )]
   pub no_download_cache : bool,

  #[arg(short, long, help = "跳过下载哈希校验", required = false )]
  pub ship_hash_check : bool,
  #[arg(short='u' , long, help = "安装前更新hp和bucket,默认不更新", required = false )]
  pub update_hp_and_bucket : bool,

  #[arg(short='a', long, help = "指定安装架构, 如果支持的话",
    required = false ,default_value ="64bit" ,value_name="<32bit|64bit|arm64>")]
  pub arch : Option<String>,

  #[arg(from_global)]
  pub  global :bool
  
}
