use clap::Args;

#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = "🍷          显示指定APP的信息")]
#[command(override_usage = "hp  info  [app_name]")]
pub struct InfoArgs {
    pub name: Option<String>,
}
