use clap::{Args, Subcommand};

#[derive(Debug, Clone , Args)]
#[clap(author, version, about="🎉\t\t创建Window终端命令的别名",  long_about = None)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(after_help = r#"
Available subcommands: add, rm, list.
Aliases are custom hp subcommands that can be created to make common tasks easier.
To add an alias:       hp alias add <name> <command> [<description>]
To rm an alias:        hp alias rm <name>
To list all aliases:   hp alias list 
示例:   hp alias add rm 'hp uninstall $args[0]' 'Uninstall an app' [描述内容可选]
\t   alias_name创建之后, 运行hp-<alias> ,例如运行hp-rm 就可以替代 hp uninstall命令进行操作
"#)]
pub struct   AliasArgs  {
  #[command(subcommand)]
  pub(crate) command: Option<AliasSubcommands> ,

  #[arg(from_global)]
  pub  global : bool,
 
}

#[derive(Subcommand, Debug, Clone)]
#[command(no_binary_name = true)]
#[command(infer_subcommands = true, infer_long_args = true)]
#[command(disable_help_subcommand = true, next_line_help = false )]
pub enum AliasSubcommands {
  Add(AddArgs),
  List(ListArgs),
  Rm(RmArgs),
}

#[derive(Args, Debug, Clone)]
#[command(about = "删除一个alias shim")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct RmArgs {
  #[arg(required = true , help="删除的仓库名称")]
  pub(crate) name: String,

  #[arg(from_global)]
  pub  global : bool,
}


#[derive(Args, Debug, Clone)]
#[command(about = "添加一个alias shim")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct AddArgs {
  #[arg(required = false ,help = "仓库名称")]
  pub(crate) name: Option<String>,
  #[arg(required = false ,help ="仓库源地址")]
  pub(crate) repo_url: Option<String>,

  #[arg(from_global)]
  pub  global : bool,
}

#[derive(Args, Debug, Clone)]
#[command(about = "列出所有alias的ps1脚本 ")]
pub struct ListArgs {
  #[arg(from_global)]
  pub  global : bool,

}


pub  fn execute_alias_command(args: AliasArgs)  ->anyhow::Result<()>{
   
  Ok(())
}