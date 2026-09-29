use clap::Args;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🍅\t\tCheck whether installed apps are up-to-date",
        "🍅\t\t检查已安装 APP 是否为最新版本"
    ),
    long_about = None
)]
#[command(arg_required_else_help = false, subcommand_negates_reqs = true)]
#[command(no_binary_name = true)]
#[clap(alias = "outdated")]
pub struct StatusArgs {
    #[arg(from_global)]
    pub global: bool,
}
