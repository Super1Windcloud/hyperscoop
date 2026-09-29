use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "📊\t\tDisplay disk space usage of installed apps, cache, and persist data",
        "📊\t\t分析已安装应用、缓存与持久化数据的磁盘占用"
    ),
    long_about = None,
    alias = "disk-usage"
)]
#[command(arg_required_else_help = false)]
pub struct SizeArgs {
    #[arg(
        help = crate::i18n::tr(
            "Name of a specific app to inspect (omitted to inspect all apps)",
            "指定要分析的 APP 名称（留空则分析所有已安装应用）"
        ),
        required = false,
        value_parser = clap_args_to_lowercase
    )]
    pub app_name: Option<String>,

    #[arg(from_global)]
    pub global: bool,
}
