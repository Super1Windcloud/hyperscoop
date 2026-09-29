use clap::{Args, Subcommand};

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "📦\t\tManage dependencies via Hpfile (Brewfile equivalent)",
        "📦\t\t通过 Hpfile 声明式管理环境依赖（类似 Brewfile）"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct BundleArgs {
    #[command(subcommand)]
    pub command: BundleSubcommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum BundleSubcommands {
    #[command(
        about = crate::i18n::tr(
            "Dump current buckets and installed apps to an Hpfile",
            "将当前已安装的 Bucket 和应用导出到 Hpfile"
        )
    )]
    Dump(BundleDumpArgs),

    #[command(
        about = crate::i18n::tr(
            "Install and satisfy all dependencies defined in Hpfile",
            "安装并满足 Hpfile 中定义的所有依赖项"
        )
    )]
    Install(BundleInstallArgs),

    #[command(
        about = crate::i18n::tr(
            "Check whether all dependencies in Hpfile are satisfied",
            "检查当前环境是否满足 Hpfile 中的依赖"
        )
    )]
    Check(BundleCheckArgs),
}

#[derive(Args, Debug, Clone)]
pub struct BundleDumpArgs {
    #[arg(
        long,
        default_value = "Hpfile",
        help = crate::i18n::tr(
            "Path to the Hpfile to write (defaults to ./Hpfile)",
            "导出的 Hpfile 目标路径（默认为 ./Hpfile）"
        )
    )]
    pub file: String,

    #[arg(
        short,
        long,
        help = crate::i18n::tr(
            "Force overwrite existing Hpfile",
            "强制覆盖已存在的 Hpfile"
        )
    )]
    pub force: bool,
}

#[derive(Args, Debug, Clone)]
pub struct BundleInstallArgs {
    #[arg(
        long,
        default_value = "Hpfile",
        help = crate::i18n::tr(
            "Path to the Hpfile to install from (defaults to ./Hpfile)",
            "要安装的 Hpfile 路径（默认为 ./Hpfile）"
        )
    )]
    pub file: String,
}

#[derive(Args, Debug, Clone)]
pub struct BundleCheckArgs {
    #[arg(
        long,
        default_value = "Hpfile",
        help = crate::i18n::tr(
            "Path to the Hpfile to check (defaults to ./Hpfile)",
            "要检查的 Hpfile 路径（默认为 ./Hpfile）"
        )
    )]
    pub file: String,
}
