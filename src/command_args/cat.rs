use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = "🐉\t\t显示App的manifest清单文件内容")]
#[command(override_usage = "hp  cat [app_name]")]
pub struct CatArgs {
    #[arg(help = "App的名称", required = false,
        value_parser = clap_args_to_lowercase
    )]
    pub app_name: String,

    #[arg(from_global)]
    pub global: bool,
}
