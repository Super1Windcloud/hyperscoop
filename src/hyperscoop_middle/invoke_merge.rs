﻿use crate::command_args::merge_bucket::MergeArgs;
use anyhow;
use command_util_lib::merge::*;
use crossterm::style::Stylize;

pub fn execute_merge_command(args: MergeArgs) -> Result<(), anyhow::Error> {
    if args.rm_err_manifest {
        println!(
            "{ }",
            "开始移除格式错误的manifest文件.......".dark_green().bold()
        );
        log::debug!("rm_err_manifest");
        let result = rm_err_manifest();
        if result.is_err() {
            let result = result.unwrap_err().to_string();
            println!("{} {}", "Error: ".red().bold(), result.red().bold());
        }
        return Ok(());
    }

    if args.rm_redundant_manifest {
        log::debug!("rm_redundant_manifest");
        let result = merge_all_buckets();
        if result.is_err() {
            let result = result.unwrap_err().to_string();
            println!("{} {}", "Error: ".red().bold(), result.red().bold());
        }
        return Ok(());
    }

    Ok(())
}
