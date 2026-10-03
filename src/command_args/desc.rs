use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "📝\t\tShow short description of an app or search by description (similar to brew desc)",
        "📝\t\t查看应用的简短描述，或通过关键词搜索应用描述 (类似 brew desc)"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct DescArgs {
    #[arg(
        help = crate::i18n::tr("App name or search keyword", "应用名称或搜索关键词"),
        value_parser = clap_args_to_lowercase
    )]
    pub query: String,

    #[arg(
        short,
        long,
        help = crate::i18n::tr("Search across descriptions of all bucket apps", "在所有 Bucket 应用描述中搜索匹配")
    )]
    pub search: bool,
}
