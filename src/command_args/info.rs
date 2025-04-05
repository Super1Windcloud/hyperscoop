use clap::Args;

#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = "🍺\t\t显示指定APP的信息")]
#[command(override_usage = "hp  info  [app_name]")]
pub struct InfoArgs {
  #[clap(help ="精准匹配, 可以指定bucket, 例如 main/zig")]
    pub name: Option<String>,
}
