use clap::ArgAction;
use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[command(name = "install", alias = "i", about = "🐘\t\t安装指定APP,别名i")]
#[clap(author="superwindcloud", version , long_about = None)]
#[command(arg_required_else_help = true)]
#[command(after_help = r#"
e.g. 安装应用程序的通常方法（使用您的本地buckets）： hp install git

指定特定buckets的清单中安装:   hp install  main/genact

安装应用程序的不同版本,如果存在多版本清单 :  hp install gh@2.7.0

从计算机上的指定路径清单中安装应用程序 :   hp install \path\to\app.json
     "#)]
pub struct InstallArgs {
    #[arg(help = "安装APP的名称,精准匹配,仅单个安装", required = false, 
    value_parser = clap_args_to_lowercase)]
    pub app_name: Option<String>,

    #[arg(short, long, help = "跳过下载文件的哈希校验", required = false, action = ArgAction::SetTrue,help_heading = "Install Options"  )]
    pub skip_download_hash_check: bool,

    #[arg(short='k' , long, help = "跳过本地缓存，强制从远程源重新下载安装", required = false , action = ArgAction::SetTrue, help_heading = "Install Options" )]
    pub no_use_download_cache: bool,
    #[arg(short='i' , long, help = "不自动下载manifest里的依赖,很大概率导致软件异常", required = false , action = ArgAction::SetTrue,help_heading = "Install Options" )]
    pub no_auto_download_dependencies: bool,

    #[arg(short='o' , long, help = "下载文件到缓存并且校验哈希,不执行安装", required = false, action = ArgAction::SetTrue,help_heading = "Install Options"  )]
    pub only_download_no_install: bool,

    #[arg( long, help = "强制下载文件到缓存并且校验哈希,不执行安装,自动覆盖缓存", required = false, action = ArgAction::SetTrue,help_heading = "Install Options"  )]
    pub only_download_no_install_with_override_cache: bool,

    #[arg(short='u' , long, help = "安装前更新hp和bucket,默认不更新", required = false , action = ArgAction::SetTrue,help_heading = "Install Options" )]
    pub update_hp_and_buckets: bool,
    #[arg(short='c' , long, help = "安装前检查hp版本是否最新,默认不检查", required = false , action = ArgAction::SetTrue,help_heading = "Install Options" )]
    pub check_version_up_to_date: bool,
  
    #[arg(short='I' , long , help = "交互式安装,默认不开启" , required = false, action = ArgAction::SetTrue,help_heading = "Install Options"  )]
    pub interactive: bool,
  
    #[arg(
        short,
        long,
        help_heading = "Install Options",
        required = false,
        help = "强制覆盖安装,先删除已安装目录"
    )]
    pub force_install_override: bool,
    #[arg(
        short = 'a',
        long,
        help = "指定安装架构, 如果支持的话",
        help_heading = "Install Options",
        required = false,
        default_value = "64bit",
        value_name = "<32bit|64bit|arm64>",
       value_parser = clap_args_to_lowercase
    )]
    pub arch: Option<String>,

    #[arg(from_global)]
    pub global: bool,
}
