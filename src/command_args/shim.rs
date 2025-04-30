use clap::{Args, Subcommand};
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(author, version, about="🐼\t\t管理所有的shim快捷方式", long_about = None)]
#[command(arg_required_else_help = true)]
#[command(after_help = "Available subcommands: add, rm, list, info, alter.
To add a custom shim, use the 'add' subcommand:
    hp shim add <shim_name> <command_path> [<args>...]
To remove shims, use the 'rm' subcommand: (CAUTION: this could remove shims added by an app manifest)
    hp shim rm <shim_name> [<shim_name>...]
To list all shims or matching shims, use the 'list' subcommand:
    hp shim list [<regex_pattern>...]
To show a shim's information, use the 'info' subcommand:
    hp shim info <shim_name>
Options:
  -g, --global       Manipulate global shim(s)

Example Usage:
    hp shim add myapp 'A:\\path\\myapp.exe'  --arguments  myapp_args")]
pub struct ShimArgs {
    #[clap(subcommand)]
    pub(crate) command: Option<ShimSubCommand>,
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

    #[arg(short, long, help = "shim的命令参数", required = false)]
    pub(crate) arguments: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}


#[derive(Debug, Args)]
#[clap(author, version, about="删除一个shim快捷方式", long_about = None)]
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
#[clap(author, version,  about="显示指定Shim 的的信息", long_about = None)]
pub struct InfoArgs {
    #[clap(help = "APP的名称", value_parser = clap_args_to_lowercase)]
    pub(crate) name: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}
