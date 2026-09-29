use clap::Args;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🧹\t\tUninstall unused packages that were installed as dependencies",
        "🧹\t\t清理所有不再被任何应用依赖的孤儿依赖包"
    ),
    long_about = None
)]
#[command(arg_required_else_help = false)]
pub struct AutoremoveArgs {
    #[arg(
        short = 'n',
        long,
        help = crate::i18n::tr(
            "Show what would be removed without actually removing anything",
            "仅展示可被清理的孤儿依赖包，不实际执行卸载"
        )
    )]
    pub dry_run: bool,

    #[arg(
        short = 'y',
        long,
        help = crate::i18n::tr(
            "Automatically confirm removal without prompting",
            "自动确认清理，无需交互式提示"
        )
    )]
    pub yes: bool,

    #[arg(from_global)]
    pub global: bool,
}
