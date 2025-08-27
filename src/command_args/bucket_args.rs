use clap::{Args, Subcommand};
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug, Clone)]
#[clap(author, version, about=None , long_about=None)]
#[clap(override_usage = "子命令  add|list|known|rm   ")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[clap(about = "🔫\t\t管理hp的所有bucket")]
pub struct BucketArgs {
    #[command(subcommand)]
    pub(crate) command: Option<BucketSubcommands>,

    #[arg(from_global)]
    pub global: bool,

    #[arg(short, long, help = "初始化自动添加官方所有bucket")]
    pub init_office_bucket: bool,
    #[arg(
        long,
        help = "初始化自动添加官方所有bucket,包括社区桶(scoopbucket,DEV-tools,ScoopMaster)"
    )]
    pub init_official_bucket_with_social: bool,
}

#[derive(Subcommand, Debug, Clone)]
#[clap(author, version, about=None , long_about=None)]
#[clap(override_usage = "子命令  add|list|known|rm repo_name ")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(disable_help_subcommand = true, next_line_help = false)]
#[command(infer_subcommands = true, infer_long_args = true)]
pub enum BucketSubcommands {
    Add(AddArgs),
    List(ListArgs),
    Known(KnownArgs),
    Rm(RmArgs),
    Update(UpdateArgs),
}

#[derive(Args, Debug, Clone)]
#[command(no_binary_name = true)]
#[clap(author, version, about = "列出所有已知bucket源  ")]
pub struct KnownArgs {
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
#[command(about = "删除一个bucket   \n---hp bucket rm <repo_name>")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct RmArgs {
    #[arg(required = true , help="删除的仓库名称",
    value_parser = clap_args_to_lowercase )]
    pub(crate) name: String,

    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
#[command(
    about = "添加一个指定bucket, 如何没有仓库名,使用URL最后一个层次名   \n---hp bucket add <name> [<repo_url>] \n---hp bucket add <repo_url>"
)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct AddArgs {
    #[arg(required = false, help = "仓库名称")]
    pub(crate) name: Option<String>,
    #[arg(required = false, help = "仓库源地址")]
    pub(crate) repo_url: Option<String>,

    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
#[command(about = "列出所有bucket ")]
pub struct ListArgs {
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
#[command(about = "更新所有bucket ")]
pub struct UpdateArgs {
    #[arg(from_global)]
    pub global: bool,
}
