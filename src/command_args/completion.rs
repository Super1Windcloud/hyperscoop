use clap::Args;
use clap_complete::Shell;

#[derive(Args, Debug)]
#[clap(
    author,
    version,
    about = crate::i18n::tr(
        "🐚\t\tGenerate shell completion script (PowerShell, Bash, Zsh, Fish, Elvish)",
        "🐚\t\t生成 Shell 自动补全脚本 (PowerShell, Bash, Zsh, Fish, Elvish)"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct CompletionArgs {
    #[arg(
        value_enum,
        help = crate::i18n::tr("The shell type to generate completion for", "需要生成补全脚本的 Shell 类型")
    )]
    pub shell: Shell,
}
