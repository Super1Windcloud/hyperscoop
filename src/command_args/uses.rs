use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🔍\t\tShow which applications depend on the specified app",
        "🔍\t\t查询哪些应用程序依赖了指定的 APP（反向依赖）"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct UsesArgs {
    #[arg(
        help = crate::i18n::tr(
            "Name of the target app to check reverse dependencies for",
            "目标 APP 名称，查询依赖此 APP 的应用"
        ),
        required = true,
        value_parser = clap_args_to_lowercase
    )]
    pub app_name: String,

    #[arg(
        short,
        long,
        help = crate::i18n::tr(
            "Scan all bucket manifests instead of only installed apps",
            "扫描所有 Bucket 中的 Manifest 清单，而不局限于已安装应用"
        )
    )]
    pub all: bool,

    #[arg(from_global)]
    pub global: bool,
}
