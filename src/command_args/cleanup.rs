use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🎠\t\tClean old versions of installed apps (alias: clean)",
        "🎠\t\t清理移除旧版本的 APP，别名 clean"
    ),
    long_about = None
)]
#[clap(arg_required_else_help = true)]
#[clap(alias = "clean")]
pub struct CleanupArgs {
    #[arg(
        short = 'a',
        long,
        help = crate::i18n::tr("Clean all installed app versions", "清理所有安装的 APP 旧版本")
    )]
    pub(crate) all: bool,
    #[arg(
        required = false,
        num_args = 1..,
        help = crate::i18n::tr("App names to clean (multiple allowed)", "要清理的 APP 名称，支持多参数"),
        value_parser = clap_args_to_lowercase
    )]
    pub(crate) app_names: Option<Vec<String>>,

    #[arg(from_global)]
    pub global: bool,
}
