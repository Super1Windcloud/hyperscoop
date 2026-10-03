use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug, Clone)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🗑️\t\tRemove a bucket/tap (alias for bucket rm)",
        "🗑️\t\t移除指定的 bucket/tap（bucket rm 的别名）"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct UntapArgs {
    #[arg(
        help = crate::i18n::tr("Name of the bucket/tap to remove", "待移除的 bucket/tap 名称"),
        required = true,
        value_parser = clap_args_to_lowercase
    )]
    pub name: String,

    #[arg(from_global)]
    pub global: bool,
}
