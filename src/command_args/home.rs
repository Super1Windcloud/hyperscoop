use clap::Args;

#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = "🐬          打开指定APP的主页")]
#[command(override_usage = "hp  home   [app_name]")]
pub struct HomeArgs {
    pub name: Option<String>,
}
