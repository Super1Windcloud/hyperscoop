use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
#[clap(author, version, about=None , long_about=None)]
#[clap(override_usage = "子命令  add|list|known|rm repo_name ")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[clap(about = "🔫          管理hyperscoop所有bucket")]
pub struct BucketArgs {
  #[command(subcommand)]
  pub(crate) command: Option<BucketSubcommands>,
}
#[derive(Subcommand, Debug, Clone)]
#[clap(author, version, about=None , long_about=None)]
#[clap(override_usage = "子命令  add|list|known|rm repo_name ")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(disable_help_subcommand = true, next_line_help = true)]
pub enum BucketSubcommands {
  Add(AddArgs),
  List(ListArgs),
  Known(KnownArgs),
  #[clap(alias = "remove")]
  Rm(RmArgs),
}

#[derive(Args, Debug, Clone)]
#[command(no_binary_name = true)]
#[clap(
  author,
  version,
  about = "\t列出所有已知bucket源  \t---hyperscoop bucket known"
)]
pub struct KnownArgs {}


#[derive(Args, Debug, Clone)]
#[command(about = "\t删除一个bucket  \t---hyperscoop bucket rm <name>")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct RmArgs {
  #[arg(required = true)]
  pub(crate) name: String,
}


#[derive(Args, Debug, Clone)]
#[command(about = "\t添加一个指定bucket  \t---hyperscoop bucket add <name> [<repo>]")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct AddArgs {
  #[arg(required = false)]
  pub(crate) name: Option<String>,
  #[arg(required = false)]
  pub(crate) repo_url: Option<String>,
}


#[derive(Args, Debug, Clone)]
#[command(about = "\t列出所有bucket  \t---hyperscoop bucket list ")]
pub struct ListArgs {}
