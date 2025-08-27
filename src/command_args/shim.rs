use clap::{Args, Subcommand};
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(author, version, about="🐼\t\t管理所有的shim快捷方式", long_about = None)]
#[command(arg_required_else_help = true)]
#[command(after_help = " 
Add : hp shim add <shim_name> <command_path> [<args>...]
Rm  : hp shim rm <shim_name> 
List: hp shim list [<regex_pattern>...]
Info: hp shim info <shim_name>
Options:
  -g, --global       Manipulate global shim(s)
示例用法:  参数可选
    hp shim add myapp 'A:\\path\\myapp.exe'  --arguments  myapp_args")]
pub struct ShimArgs {
    #[clap(subcommand)]
    pub(crate) command: Option<ShimSubCommand>,

    #[arg(from_global)]
    pub global: bool,
}

#[derive(Debug, Args)]
#[clap(author, version, about="添加一个shim快捷方式", long_about = None)]
#[clap(arg_required_else_help = true)]
#[command(after_help = "Eg : hp shim add myapp 'A:\\path\\myapp.exe' --arguments  myapp_args")]
pub struct AddArgs {
    #[arg(help = "shim的名称" , required = false ,value_parser = clap_args_to_lowercase)]
    pub(crate) name: Option<String>,
    #[arg(help = "shim的路径", required = false)]
    pub(crate) path: Option<String>,

    #[arg(short, long, help = "shim的命令参数,参数可选", required = false)]
    pub(crate) arguments: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Debug, Args)]
#[clap(author, version, about="删除一个shim快捷方式", long_about = None)]
#[clap(arg_required_else_help = true)]
pub struct RmArgs {
    #[clap(help = "shim的名称", value_parser = clap_args_to_lowercase)]
    pub(crate) name: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Debug, Subcommand)]
pub enum ShimSubCommand {
    Add(AddArgs),
    Rm(RmArgs),
    List(ListArgs),
    Info(InfoArgs),
    Clear(ClearArgs),
}

#[derive(Args, Debug)]
#[clap(author, version, about="更改shim的目标源", long_about = None)]
pub struct AlterArgs {
    #[arg(help = "shim的名称",value_parser = clap_args_to_lowercase)]
    name: String,
    #[arg(help = "shim的路径")]
    path: String,
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug)]
#[clap(author, version, about="列出所有的shim快捷方式", long_about = None)]
pub struct ListArgs {
    #[arg(short, long, help = "正则匹配shim名称" , value_parser = clap_args_to_lowercase
     ,required =  false )]
    pub regex: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug)]
#[clap(author, version, about="清理无效的shim快捷方式", long_about = None)]
pub struct ClearArgs {
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug)]
#[clap(author, version,  about="显示指定Shim 的的信息", long_about = None)]
#[clap(arg_required_else_help = true)]
pub struct InfoArgs {
    #[clap(help = "APP的名称", value_parser = clap_args_to_lowercase)]
    pub(crate) name: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}
