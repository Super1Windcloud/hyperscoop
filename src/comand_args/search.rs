use clap::{command, Args};

#[derive(Args, Debug)]
#[command(name = "search", about = "🦄          搜索可用的指定名称APP(别名为 s)")]
#[command(arg_required_else_help = true)]
pub struct SearchArgs {
    #[clap(help = "搜索app的名称,可以指定bucket,例如: main/rust")]
    #[clap(required = false)]
    pub(crate) name: String,
    #[clap(required = false)]
    #[clap(short, long, help = "默认模糊匹配 ,开启选项则精确匹配")]
    pub(crate) exact_match_option: bool,
}

//

struct Struct {
    name: String,
}
