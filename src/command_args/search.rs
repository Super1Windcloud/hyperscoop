use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[command(
    name = "search",
    about = crate::i18n::tr(
        "🦄  Search apps by name (alias: s)",
        "🦄  搜索可用的指定名称 APP（别名 s）"
    )
)]
#[command(arg_required_else_help = true)]
pub struct SearchArgs {
    #[clap(
        help = crate::i18n::tr(
            "App name to search; bucket can be specified, e.g. main/rust",
            "搜索 app 的名称，可指定 bucket，例如 main/rust"
        )
    )]
    #[clap(required = false, value_parser = clap_args_to_lowercase)]
    pub(crate) name: String,
    #[clap(required = false)]
    #[clap(
        short,
        long,
        help = crate::i18n::tr(
            "Use exact match instead of fuzzy match",
            "默认模糊匹配，开启后改为精确匹配"
        )
    )]
    pub(crate) exact_match_option: bool,

    #[arg(from_global)]
    pub global: bool,
}
