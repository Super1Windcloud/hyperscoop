use clap::{Args, Subcommand};
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug, Clone)]
#[clap(author, version, about=None , long_about=None)]
#[clap(override_usage = crate::i18n::tr(
    "Subcommands: add|list|known|rm",
    "子命令  add|list|known|rm"
))]
#[command(arg_required_else_help = false, subcommand_negates_reqs = true)]
#[clap(about = crate::i18n::tr(
    "🔫\t\tManage all hp buckets/taps (alias: tap)",
    "🔫\t\t管理 hp 的所有 bucket/tap，别名 tap"
))]
pub struct BucketArgs {
    #[command(subcommand)]
    pub(crate) command: Option<BucketSubcommands>,

    #[arg(from_global)]
    pub global: bool,

    #[arg(
        short,
        long,
        help = crate::i18n::tr(
            "Initialize and add all official buckets",
            "初始化自动添加官方所有 bucket"
        )
    )]
    pub init_office_bucket: bool,
    #[arg(
        long,
        help = crate::i18n::tr(
            "Initialize and add official buckets plus community buckets (scoopbucket, DEV-tools, ScoopMaster)",
            "初始化自动添加官方所有 bucket, 包括社区桶 (scoopbucket, DEV-tools, ScoopMaster)"
        )
    )]
    pub init_official_bucket_with_social: bool,
}

#[derive(Subcommand, Debug, Clone)]
#[clap(author, version, about=None , long_about=None)]
#[clap(override_usage = crate::i18n::tr(
    "Subcommands: add|list|known|rm <repo_name>",
    "子命令  add|list|known|rm <repo_name>"
))]
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
#[clap(author, version, about = crate::i18n::tr(
    "List all known bucket sources",
    "列出所有已知 bucket 源"
))]
pub struct KnownArgs {
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
#[command(about = crate::i18n::tr(
    "Remove a bucket\n---hp bucket rm <repo_name>",
    "删除一个 bucket\n---hp bucket rm <repo_name>"
))]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct RmArgs {
    #[arg(
        required = true,
        help = crate::i18n::tr("Bucket name to remove", "删除的仓库名称"),
        value_parser = clap_args_to_lowercase
    )]
    pub(crate) name: String,

    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
#[command(about = crate::i18n::tr(
    "Add a bucket; when the name is omitted, the last path segment of the URL will be used\n---hp bucket add <name> [<repo_url>]\n---hp bucket add <repo_url>",
    "添加一个指定 bucket，如没有仓库名则使用 URL 最后一级名称\n---hp bucket add <name> [<repo_url>]\n---hp bucket add <repo_url>"
))]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct AddArgs {
    #[arg(
        required = false,
        help = crate::i18n::tr("Bucket name", "仓库名称")
    )]
    pub(crate) name: Option<String>,
    #[arg(
        required = false,
        help = crate::i18n::tr("Bucket source URL", "仓库源地址")
    )]
    pub(crate) repo_url: Option<String>,

    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
#[command(about = crate::i18n::tr("List all buckets", "列出所有 bucket"))]
pub struct ListArgs {
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
#[command(about = crate::i18n::tr(
    "Update all buckets",
    "更新所有 bucket"
))]
pub struct UpdateArgs {
    #[arg(from_global)]
    pub global: bool,
}
