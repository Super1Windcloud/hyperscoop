use crate::command_args::edit::EditArgs;
use crate::i18n::tr;
use anyhow::{Context, bail};
use command_util_lib::manifest::manifest::{
    get_best_manifest_from_local_bucket, get_best_manifest_from_local_bucket_global,
};
use crossterm::style::Stylize;
use std::process::Command;

pub fn execute_edit_command(args: EditArgs) -> anyhow::Result<()> {
    let manifest_path = if args.global {
        get_best_manifest_from_local_bucket_global(&args.app_name)?
    } else {
        get_best_manifest_from_local_bucket(&args.app_name)?
    };

    println!(
        "{} {}",
        tr("Opening manifest in editor:", "正在使用编辑器打开清单:")
            .dark_cyan()
            .bold(),
        manifest_path.display().to_string().dark_green()
    );

    let editor = args
        .editor
        .or_else(|| std::env::var("EDITOR").ok())
        .or_else(|| std::env::var("VISUAL").ok())
        .unwrap_or_else(|| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "nano".to_string()
            }
        });

    let status = Command::new(&editor)
        .arg(&manifest_path)
        .status()
        .context(format!("Failed to spawn editor '{editor}'"))?;

    if !status.success() {
        bail!("Editor '{editor}' exited with error: {:?}", status.code());
    }

    Ok(())
}
