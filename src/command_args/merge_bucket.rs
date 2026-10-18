use clap::Args;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🐧  Remove redundant or invalid manifests in buckets",
        "🐧  移除 buckets 中冗余和错误的 manifest 文件"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(no_binary_name = true)]
#[command(after_help = crate::i18n::tr(
    "Only touches community buckets; official scoop buckets are ignored (see hp bucket known)",
    "只会操作社区 bucket，忽略 scoop 官方 bucket，可通过 hp bucket known 查看"
))]
pub struct MergeArgs {
    #[arg(
        short = 'e',
        long,
        help = crate::i18n::tr("Remove malformed manifests", "移除 buckets 中格式错误的 manifest 文件")
    )]
    pub rm_err_manifest: bool,

    #[arg(
        short = 'r',
        long,
        help = crate::i18n::tr("Remove redundant manifests", "移除 buckets 中冗余的 manifest 文件"),
        help_heading = crate::i18n::tr("Large community buckets only", "仅超大型社区桶")
    )]
    pub rm_redundant_manifest: bool,
}
