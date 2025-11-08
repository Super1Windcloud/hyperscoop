use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🍻\t\tSwitch or reset app versions",
        "🍻\t\t切换指定 APP 版本或重置为最新版本"
    ),
    long_about = "None"
)]
#[command(arg_required_else_help = true)]
pub struct ResetArgs {
    #[arg(
        help = crate::i18n::tr(
            "App name, e.g. reset python@3.9 or reset python",
            "APP 名称，例如 reset python@3.9 或 reset python"
        ),
        required = false,
        value_parser = clap_args_to_lowercase
    )]
    pub(crate) name: Option<String>,

    #[arg(
        required = false,
        short,
        long,
        help = crate::i18n::tr("Reset shims as well", "是否一并重置 shim")
    )]
    pub shim_reset: bool,
    #[arg(from_global)]
    pub global: bool,
}
