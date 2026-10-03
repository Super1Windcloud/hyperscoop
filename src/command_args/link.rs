use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🔗\t\tCreate shims and shortcuts for an app (similar to brew link)",
        "🔗\t\t为已安装应用生成 shim 与快捷方式入口 (类似 brew link)"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct LinkArgs {
    #[arg(
        help = crate::i18n::tr("App name(s) to link", "要生成链接的 APP 名称"),
        required = true,
        num_args = 1..,
        value_parser = clap_args_to_lowercase
    )]
    pub app_names: Vec<String>,

    #[arg(from_global)]
    pub global: bool,
}
