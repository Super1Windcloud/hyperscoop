use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(name = "which", about = "🐸\t\t打印指定APP的可执行文件路径")]
#[clap(arg_required_else_help = true)]
pub struct WhichArgs {
    #[arg(required = false ,help = "指定APP名称",
    value_parser = clap_args_to_lowercase)]
    pub(crate) name: Option<String>,
    #[arg(from_global)]
    pub global: bool,
}
