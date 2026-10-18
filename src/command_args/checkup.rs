use clap::Args;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🐇  Check every potential issue (alias: check, doctor)",
        "🐇  检查所有潜在问题，别名 check, doctor"
    ),
    long_about = None
)]
#[clap(alias = "check", alias = "doctor")]
pub struct CheckupArgs {
    #[arg(from_global)]
    pub global: bool,

    #[arg(
        short,
        long,
        help = crate::i18n::tr(
            "Automatically attempt to fix detected issues",
            "自动尝试修复检测到的问题"
        )
    )]
    pub fix: bool,
}
