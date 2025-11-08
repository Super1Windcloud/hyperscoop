use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug, Clone)]
#[command(about = crate::i18n::tr(
    "🦀\t\tList all installed apps",
    "🦀\t\t列出已安装的所有 app"
))]
#[command(arg_required_else_help = false, subcommand_negates_reqs = true)]
#[command(after_help = crate::i18n::tr(
    "Supports fuzzy match and multiple values, e.g. hp list zig rust",
    "支持模糊匹配和多参数查询，例如 hp list zig rust"
))]
pub struct ListArgs {
    #[clap(
        required = false,
        num_args = 1..,
        help = crate::i18n::tr("Filter apps by fuzzy names", "列出指定 app，使用模糊匹配"),
        value_parser = clap_args_to_lowercase
    )]
    pub(crate) name: Option<Vec<String>>,

    #[arg(from_global)]
    pub global: bool,
}
