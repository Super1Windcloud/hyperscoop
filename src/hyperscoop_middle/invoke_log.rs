use crate::command_args::log::LogArgs;
use crate::i18n::tr;
use anyhow::{Context, bail};
use command_util_lib::manifest::manifest::{
    get_best_manifest_from_local_bucket, get_best_manifest_from_local_bucket_global,
};
use crossterm::style::Stylize;
use std::process::Command;

pub fn execute_log_command(args: LogArgs) -> anyhow::Result<()> {
    let manifest_path = if args.global {
        get_best_manifest_from_local_bucket_global(&args.app_name)?
    } else {
        get_best_manifest_from_local_bucket(&args.app_name)?
    };

    let bucket_dir = manifest_path
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("Failed to locate bucket repository"))?;

    let count = args.max_count.unwrap_or(10);
    println!(
        "{} {}\n",
        tr("Git commit history for", "Git 提交历史:")
            .dark_cyan()
            .bold(),
        args.app_name.dark_green().bold()
    );

    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(bucket_dir)
        .arg("log")
        .arg(format!("-n{}", count))
        .arg("--pretty=format:%C(yellow)%h%Creset %C(green)%ad%Creset %s %C(blue)<%an>%Creset")
        .arg("--date=short")
        .arg("--")
        .arg(&manifest_path);

    let status = cmd.status().context("Failed to run 'git log'")?;
    println!();
    if !status.success() {
        bail!("git log exited with error: {:?}", status.code());
    }

    Ok(())
}
