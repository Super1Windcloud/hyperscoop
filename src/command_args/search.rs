use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug)]
#[command(name = "search", about = "🦄\t\t搜索可用的指定名称APP(别名为 s)")]
#[command(arg_required_else_help = true)]
pub struct SearchArgs {
    #[clap(help = "搜索app的名称,可以指定bucket,例如: main/rust")]
    #[clap(required = false, value_parser = clap_args_to_lowercase)]
    pub(crate) name: String,
    #[clap(required = false)]
    #[clap(short, long, help = "默认模糊匹配 ,开启选项则精确匹配")]
    pub(crate) exact_match_option: bool,

    #[arg(from_global)]
    pub global: bool,
}
