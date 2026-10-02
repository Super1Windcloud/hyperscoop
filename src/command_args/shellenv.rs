use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug, Clone)]
#[clap(
    author = "superwindcloud",
    version,
    about = crate::i18n::tr(
        "🐚\t\tPrint shell environment setup commands (similar to brew shellenv)",
        "🐚\t\t输出终端环境初始化脚本（类似 brew shellenv）"
    ),
    long_about = None
)]
pub struct ShellenvArgs {
    #[arg(
        help = crate::i18n::tr(
            "Target shell: powershell, cmd, bash, zsh, nu (default: auto-detect)",
            "目标 Shell 类型: powershell, cmd, bash, zsh, nu（默认自动探测）"
        ),
        value_parser = clap_args_to_lowercase
    )]
    pub shell: Option<String>,
}
