use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(name = "prefix")]
#[clap(about = crate::i18n::tr(
    "👻\t\tPrint the install directory of an app",
    "👻\t\t打印指定 APP 的安装目录"
))]
#[clap(arg_required_else_help = true)]
pub struct PrefixArgs {
    #[arg(
        required = false,
        num_args = 1..,
        help = crate::i18n::tr("App name(s)", "指定 APP 的名称（支持查询多个）"),
        value_parser = clap_args_to_lowercase
    )]
    pub(crate) names: Vec<String>,

    #[arg(from_global)]
    pub(crate) global: bool,
}
