use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = crate::i18n::tr(
    "🐉\t\tShow the manifest content of an app",
    "🐉\t\t显示 App 的 manifest 清单文件内容"
))]
#[command(override_usage = crate::i18n::tr(
    "hp cat [app_name]",
    "hp cat [app_name]"
))]
pub struct CatArgs {
    #[arg(
        help = crate::i18n::tr("App name(s)", "App 的名称（支持查看多个）"),
        required = false,
        num_args = 1..,
        value_parser = clap_args_to_lowercase
    )]
    pub app_names: Vec<String>,

    #[arg(from_global)]
    pub global: bool,
}
