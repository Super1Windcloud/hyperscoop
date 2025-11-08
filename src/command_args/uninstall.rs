use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(
    author = "superwindcloud",
    version,
    about = crate::i18n::tr("⛄\t\tUninstall an app", "⛄\t\t卸载指定 APP"),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct UninstallArgs {
    #[arg(
        help = crate::i18n::tr(
            "Name of the app to uninstall (exact match, single)",
            "卸载指定 APP 的名称，精准匹配，仅单个卸载"
        ),
        required = false,
        value_parser = clap_args_to_lowercase
    )]
    pub(crate) app_name: Option<String>,
    #[arg(
        short,
        long,
        help = crate::i18n::tr(
            "Also delete persisted data at $SCOOP/persist/<app>",
            "是否删除持久化数据，路径 $SCOOP/persist/<app>"
        ),
        long_help = crate::i18n::tr(
            "Equivalent to: scoop uninstall <app> --purge",
            "等价于：scoop uninstall <app> --purge"
        )
    )]
    pub(crate) purge: bool,
    #[arg(from_global)]
    pub global: bool,
    #[arg(
        short,
        long,
        help = crate::i18n::tr(
            "Force removal and terminate running processes",
            "强制删除，自动杀掉运行中进程"
        )
    )]
    pub force: bool,
}
