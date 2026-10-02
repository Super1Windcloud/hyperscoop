use clap::Args;
use command_util_lib::utils::utility::clap_args_to_lowercase;

#[derive(Args, Debug, Clone)]
#[clap(
    author = "superwindcloud",
    version,
    about = crate::i18n::tr(
        "📝\t\tOpen an app manifest in your default editor",
        "📝\t\t在默认编辑器中打开指定 APP 的 Manifest 文件"
    ),
    long_about = None
)]
#[command(arg_required_else_help = true)]
pub struct EditArgs {
    #[arg(
        help = crate::i18n::tr("App name to edit manifest for", "要编辑 Manifest 的 APP 名称"),
        required = true,
        value_parser = clap_args_to_lowercase
    )]
    pub app_name: String,

    #[arg(
        short,
        long,
        help = crate::i18n::tr(
            "Editor executable to use (defaults to $EDITOR, $VISUAL, or notepad)",
            "指定的编辑器程序（默认优先读取 $EDITOR / $VISUAL，缺省使用 notepad）"
        )
    )]
    pub editor: Option<String>,

    #[arg(from_global)]
    pub global: bool,
}
