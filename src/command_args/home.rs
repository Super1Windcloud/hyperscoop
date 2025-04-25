use clap::Args;
// 获取或设置配置文件
use command_util_lib::utils::utility::clap_args_to_lowercase;
#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = "🐳\t\t打开指定APP的主页")]
#[command(override_usage = "hp  home   [app_name]")]
pub struct HomeArgs { 
    #[arg(required = false , help = "指定APP的名称" , 
    value_parser = clap_args_to_lowercase )]
    pub name: Option<String>,
}
