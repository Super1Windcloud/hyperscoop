use clap::Args;

#[derive(Args, Debug)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(about = "🐼          显示App的manifest清单文件内容")]
#[command(override_usage = "hp  cat [app_name]")]
pub struct CatArgs {
    app_name: String,
}
