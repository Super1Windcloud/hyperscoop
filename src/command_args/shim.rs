use clap::{Args, Subcommand};
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🐼\t\tManage shim shortcuts",
        "🐼\t\t管理所有的 shim 快捷方式"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
#[command(after_help = crate::i18n::tr(
    r#"Add : hp shim add <shim_name> <command_path> [<args>...]
Rm  : hp shim rm <shim_name>
List: hp shim list [<regex_pattern>...]
Info: hp shim info <shim_name>
Options:
  -g, --global       Manipulate global shim(s)
Example (arguments optional):
    hp shim add myapp 'A:\path\myapp.exe' --arguments myapp_args"#,
    r#"添加: hp shim add <shim_name> <command_path> [<args>...]
删除: hp shim rm <shim_name>
列出: hp shim list [<regex_pattern>...]
详情: hp shim info <shim_name>
选项:
  -g, --global       操作全局 shim
示例（参数可选）:
    hp shim add myapp 'A:\path\myapp.exe' --arguments myapp_args"#
))]
pub struct ShimArgs {
    #[clap(subcommand)]
    pub(crate) command: Option<ShimSubCommand>,

    #[arg(from_global)]
    pub global: bool,
}

#[derive(Debug, Args)]
#[clap(
    author,
    version,
    about = crate::i18n::tr("Add a shim shortcut", "添加一个 shim 快捷方式"),
    long_about = None
)]
#[clap(arg_required_else_help = true)]
#[command(after_help = crate::i18n::tr(
    "Example: hp shim add myapp 'A:\\path\\myapp.exe' --arguments myapp_args",
    "示例: hp shim add myapp 'A:\\path\\myapp.exe' --arguments myapp_args"
))]
pub struct AddArgs {
    #[arg(
        help = crate::i18n::tr("Shim name", "shim 的名称"),
        required = false,
        value_parser = clap_args_to_lowercase
    )]
    pub(crate) name: Option<String>,
    #[arg(
        help = crate::i18n::tr("Path to shim target", "shim 的路径"),
        required = false
    )]
    pub(crate) path: Option<String>,

    #[arg(
        short,
        long,
        help = crate::i18n::tr("Command arguments for the shim (optional)", "shim 的命令参数，可选"),
        required = false
    )]
    pub(crate) arguments: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Debug, Args)]
#[clap(
    author,
    version,
    about = crate::i18n::tr("Delete a shim shortcut", "删除一个 shim 快捷方式"),
    long_about = None
)]
#[clap(arg_required_else_help = true)]
pub struct RmArgs {
    #[clap(
        help = crate::i18n::tr("Shim name", "shim 的名称"),
        value_parser = clap_args_to_lowercase
    )]
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
#[clap(
    author,
    version,
    about = crate::i18n::tr("Change shim targets", "更改 shim 的目标路径"),
    long_about = None
)]
pub struct AlterArgs {
    #[arg(
        help = crate::i18n::tr("Shim name", "shim 的名称"),
        value_parser = clap_args_to_lowercase
    )]
    name: String,
    #[arg(help = crate::i18n::tr("Shim target path", "shim 的路径"))]
    path: String,
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr("List shim shortcuts", "列出所有 shim 快捷方式"),
    long_about = None
)]
pub struct ListArgs {
    #[arg(
        short,
        long,
        help = crate::i18n::tr("Regex filter for shim names", "正则匹配 shim 名称"),
        value_parser = clap_args_to_lowercase,
        required = false
    )]
    pub regex: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr("Clean invalid shims", "清理无效的 shim 快捷方式"),
    long_about = None
)]
pub struct ClearArgs {
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr("Show shim details", "显示指定 shim 的信息"),
    long_about = None
)]
#[clap(arg_required_else_help = true)]
pub struct InfoArgs {
    #[clap(
        help = crate::i18n::tr("App name", "APP 的名称"),
        value_parser = clap_args_to_lowercase
    )]
    pub(crate) name: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}
