use clap::Args;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🍃\t\tList installed apps that are not dependencies of any other app",
        "🍃\t\t列出所有顶层已安装应用（不被其他应用依赖的叶子应用）"
    ),
    long_about = None
)]
#[command(arg_required_else_help = false)]
pub struct LeavesArgs {
    #[arg(from_global)]
    pub global: bool,
}
