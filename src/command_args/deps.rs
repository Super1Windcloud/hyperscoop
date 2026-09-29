use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🌲\t\tDisplay dependencies of the specified app",
        "🌲\t\t查看指定 APP 的依赖项"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct DepsArgs {
    #[arg(
        help = crate::i18n::tr(
            "Name of the app to query dependencies for",
            "目标 APP 名称，查询其依赖项"
        ),
        required = true,
        value_parser = clap_args_to_lowercase
    )]
    pub app_name: String,

    #[arg(
        short,
        long,
        help = crate::i18n::tr(
            "Show dependencies as a recursive tree",
            "以递归依赖树的形式展示"
        )
    )]
    pub tree: bool,

    #[arg(from_global)]
    pub global: bool,
}
