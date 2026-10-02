use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug, Clone)]
#[clap(
    author = "superwindcloud",
    version,
    about = crate::i18n::tr(
        "📜\t\tShow git commit history of an app manifest",
        "📜\t\t显示指定 APP Manifest 的 Git 提交变更历史"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct LogArgs {
    #[arg(
        help = crate::i18n::tr("App name to show log for", "要查询提交历史的 APP 名称"),
        required = true,
        value_parser = clap_args_to_lowercase
    )]
    pub app_name: String,

    #[arg(
        short = 'n',
        long,
        help = crate::i18n::tr("Number of commits to show (default: 10)", "展示的最大提交条数（默认为 10）")
    )]
    pub max_count: Option<usize>,

    #[arg(from_global)]
    pub global: bool,
}
