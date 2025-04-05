
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
#[clap(author, version, about="🐼\t\t管理所有的shim快捷方式", long_about = None)]
#[command(arg_required_else_help = true)]
pub struct ShimArgs         {
   #[clap(subcommand)]
   pub(crate) command : Option<ShimSubCommand>
}



#[derive(Debug , Args)]
#[clap(author, version, about="添加一个shim快捷方式", long_about = None)]
pub struct AddArgs {
   #[arg(help="shim的名称")]
   pub(crate) name: String,
   #[arg(help="shim的路径")]
   pub(crate) path: String,
}
#[derive(Debug , Args)]
#[clap(author, version, about="删除一个shim快捷方式", long_about = None)]
pub struct RmArgs {
   #[clap(help="shim的名称")]
   name: String,
}
#[derive( Debug , Subcommand)]
pub enum ShimSubCommand {
   Add(AddArgs),
   Rm(RmArgs),
  List(ListArgs),
  Info(InfoArgs),
  Alter(AlterArgs),

  }


#[derive(Args, Debug)]
#[clap(author, version, about="更改shim的目标源", long_about = None)]
pub struct AlterArgs {
   #[arg(help="shim的名称")]
   name: String,
   #[arg(help="shim的路径")]
   path: String,
}

#[derive(Args, Debug)]
#[clap(author, version, about="列出所有的shim快捷方式", long_about = None)]
pub struct ListArgs {
    #[arg(short , long , help="正则匹配shim名称")]
      pub regex : Option<String> ,
}
#[derive(Args, Debug)]
#[clap(author, version,  about="显示指定Shim 的的信息", long_about = None)]
pub struct InfoArgs {
   #[clap(help="APP的名称")]
   pub(crate) name: String,
}
