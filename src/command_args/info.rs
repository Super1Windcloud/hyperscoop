use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = crate::i18n::tr(
    "🍺\t\tShow information about an app",
    "🍺\t\t显示指定 APP 的信息"
))]
#[command(override_usage = crate::i18n::tr(
    "hp info [app_name]",
    "hp info [app_name]"
))]
pub struct InfoArgs {
    #[clap(
        help = crate::i18n::tr(
            "Exact match; bucket can be specified, e.g. main/zig",
            "精准匹配，可指定 bucket，例如 main/zig"
        ),
        value_parser = clap_args_to_lowercase
    )]
    pub name: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}
