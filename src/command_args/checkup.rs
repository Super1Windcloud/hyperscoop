use clap::Args;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🐇\t\tCheck every potential issue (alias: check)",
        "🐇\t\t检查所有潜在问题，别名 check"
    ),
    long_about = None
)]
#[clap(alias = "check")]
pub struct CheckupArgs {
    #[arg(from_global)]
    pub global: bool,
}
