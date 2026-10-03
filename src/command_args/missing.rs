use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🔍\t\tCheck for missing dependencies among installed apps (similar to brew missing)",
        "🔍\t\t检查已安装应用中缺失的依赖项 (类似 brew missing)"
    ),
    long_about = None
)]
#[command(arg_required_else_help = false)]
pub struct MissingArgs {
    #[arg(
        help = crate::i18n::tr("Optional app name to check", "要检查的具体应用名称（不传则检查所有已安装应用）"),
        required = false,
        value_parser = clap_args_to_lowercase
    )]
    pub app_name: Option<String>,

    #[arg(from_global)]
    pub global: bool,
}
