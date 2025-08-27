use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[clap(name = "prefix")]
#[clap(about = "👻\t\t打印指定APP的安装目录")]
#[clap(arg_required_else_help = true)]
pub struct PrefixArgs {
    #[arg(required = false ,help = "指定APP的名称",
  value_parser = clap_args_to_lowercase)]
    pub(crate) name: Option<String>,

    #[arg(from_global)]
    pub(crate) global: bool,
}
