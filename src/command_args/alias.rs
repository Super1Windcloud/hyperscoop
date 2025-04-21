use clap::Args;

#[derive(Args, Debug)]
#[clap(author, version, about="🎉\t\t创建Window终端命令的别名",  long_about = None)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(after_long_help = r#"
Available subcommands: add, rm, list.

Aliases are custom hp subcommands that can be created to make common tasks easier.

To add an alias:

    hp alias add <name> <command> [<description>]

e.g.,

    hp alias add rm 'hp uninstall $args[0]' 'Uninstall an app'
    hp alias add upgrade 'hp update *' 'Update all apps, just like "brew" or "apt"'

To remove an alias:

    hp alias rm <name>

To list all aliases:

    hp alias list [-v|--verbose]
"#)]
#[command(no_binary_name = true)]
pub struct AliasArgs  {
  #[arg( required = false,  num_args =1.., help = "要锁定的APP名称,精准匹配,支持多参数")]
  pub app_names: Option<Vec<String>>,
  #[arg(short = 'u', long, required = false, help = "取消锁定, 支持多参数")]
  pub cancel_hold: bool,
}

pub  fn execute_alias_command(args: AliasArgs)  ->anyhow::Result<()>{
   
  Ok(())
}