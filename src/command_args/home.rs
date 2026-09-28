use clap::Args;
// 获取或设置配置文件
use command_util_lib::utils::utility::clap_args_to_lowercase;
#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = crate::i18n::tr(
    "🐳\t\tOpen the homepage of an app",
    "🐳\t\t打开指定 APP 的主页"
))]
#[command(override_usage = crate::i18n::tr(
    "hp home [app_name]",
    "hp home [app_name]"
))]
pub struct HomeArgs {
    #[arg(
        required = false,
        num_args = 1..,
        help = crate::i18n::tr("App name(s)", "指定 APP 的名称（支持打开多个）"),
        value_parser = clap_args_to_lowercase
    )]
    pub names: Vec<String>,
}
