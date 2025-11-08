use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    name = "which",
    about = crate::i18n::tr(
        "🐸\t\tPrint the executable path of an app",
        "🐸\t\t打印指定 APP 的可执行文件路径"
    )
)]
#[clap(arg_required_else_help = true)]
pub struct WhichArgs {
    #[arg(
        required = false,
        help = crate::i18n::tr("App name", "指定 APP 名称"),
        value_parser = clap_args_to_lowercase
    )]
    pub(crate) name: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}
