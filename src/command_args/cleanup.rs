use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(author, version, about="🎠\t\t清理移除旧版本的APP,别名clean ", long_about = None)]
#[clap(arg_required_else_help = true)]
#[clap(alias = "clean")]
pub struct CleanupArgs {
    #[arg(short = 'a', long, help = "清理所有安装的APP旧版本")]
    pub(crate) all: bool,
    #[arg(required=false ,  num_args =1.., help = "清理app的名称,支持多参数",
    value_parser = clap_args_to_lowercase )]
    pub(crate) app_names: Option<Vec<String>>,

    #[arg(from_global)]
    pub global: bool,
}
