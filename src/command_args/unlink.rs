use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "⛓️‍💥\t\tRemove shims and shortcuts without uninstalling (similar to brew unlink)",
        "⛓️‍💥\t\t移除应用的 shim 与快捷方式，但不删除应用文件 (类似 brew unlink)"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct UnlinkArgs {
    #[arg(
        help = crate::i18n::tr("App name(s) to unlink", "要断开链接的 APP 名称"),
        required = true,
        num_args = 1..,
        value_parser = clap_args_to_lowercase
    )]
    pub app_names: Vec<String>,

    #[arg(from_global)]
    pub global: bool,
}
