
use clap::{Args, Subcommand};

#[derive(Debug, Clone, Args)]
#[command( about  = "设置配置选项, config set <name> <value>")]
pub struct SetArgs {
  pub name: String,
  pub value: String,
}

#[derive(Args, Debug)]
#[command( about  = "获取指定配置, config get <name>")]
pub struct GetArgs {
  pub name: String,
}
#[derive(Args, Debug)]
#[command( about  = "显示所有配置, config show")]

pub struct ShowArgs {}

#[derive(Args, Debug)]
#[command( about  = "删除指定配置, config rm <name> ")]

pub struct RmArgs {
  pub name: String,
}
#[derive(Debug , Subcommand )]
pub(crate) enum ConfigSubcommand {
  Show( ShowArgs),
  Set(SetArgs),
  Get(GetArgs),
  Rm (RmArgs),
}
#[derive(Args, Debug)]
#[clap(author, version, about="🐼\t\t获取或设置配置文件", arg_required_else_help = true) ]
pub struct ConfigArgs  {
  #[clap(subcommand)]
  pub(crate) command: Option<ConfigSubcommand>,

  #[clap(short, long ,help ="显示配置帮助信息")]
  pub config_help  : bool,

}

pub const STR : &str =  r#"
You Can  Set $SCOOP  to change the default directory for Scoop. 
The scoop configuration file is saved at ~/.config/scoop/config.json.

To get all configuration settings:

    hp config show

To get a configuration setting:

    hp config get <name>

To set a configuration setting:

    hp config set <name> <value>

To remove a configuration setting:

    hp config rm <name>

Settings
--------
scoop_repo: http://github.com/ScoopInstaller/Scoop
      Git repository containining scoop source code.
      This configuration is useful for custom forks.

scoop_branch: master|develop
      Allow to use different branch than master.
      Could be used for testing specific functionalities before released into all users.
      If you want to receive updates earlier to test new functionalities use develop (see: 'https://github.com/ScoopInstaller/Scoop/issues/2939')

proxy:  host:port   eg : 127.0.0.1:7890  or http://127.0.0.1:7890
      By default, Scoop will use the proxy settings from Internet Options, but with anonymous authentication.
      * To use the credentials for the current logged-in user, use 'currentuser' in place of username:password
      * To use the system proxy settings configured in Internet Options, use 'default' in place of host:port


root_path: $Env:UserProfile\\scoop
      Path to Scoop root directory.

global_path: $Env:ProgramData\\scoop
      Path to Scoop root directory for global apps.

cache_path:
      For downloads, defaults to 'cache' folder under Scoop root directory.
 ate limits and download from private repositories.
"# ;
