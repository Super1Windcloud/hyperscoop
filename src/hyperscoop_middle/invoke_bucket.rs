use std::env;
use crate::command::BucketArgs;
use crate::command_args::bucket_args::BucketSubcommands;
use crate::hyperscoop_middle::invoke_update::update_buckets_parallel;
use anyhow::anyhow;
use command_util_lib::buckets::Buckets;
use crossterm::style::Stylize;
use command_util_lib::utils::system::{is_admin, request_admin};

pub fn execute_bucket_command(args: BucketArgs) -> Result<(), anyhow::Error> {
    let buckets = Buckets::new()?;
    let global = args.global;
    let args = args.command; 
    if  global && !is_admin()? {
      let args =env::args().skip(1). collect::<Vec<String>>();
      let  args_str= args.join(" ");
      log::warn!("Global command arguments: {}", args_str.clone().dark_yellow());
      request_admin( args_str.as_str())?;
      return Ok(());
    }
    let bucket_args = args.as_ref().expect("bucket_args cannot be none");
    match bucket_args {
        BucketSubcommands::Add(add_args) => {
            match (add_args.name.is_some(), add_args.repo_url.is_some()) {
                (true, true) => {
                    if add_args.global {
                        buckets.add_buckets(&add_args.name, &add_args.repo_url, true)?
                    } else {
                        buckets.add_buckets(&add_args.name, &add_args.repo_url, false)?
                    }
                }
                (true, false) => {
                    let first = add_args.name.clone().unwrap_or(String::new());
                    if buckets.is_valid_url(&first) {
                        let url = first;
                        if add_args.global {
                            buckets.add_buckets(&None, &Some(url), true)?
                        } else {
                            buckets.add_buckets(&None, &Some(url), false)?
                        }
                    } else {
                        if add_args.global {
                            buckets.add_buckets(&add_args.name, &None, true)?
                        } else {
                            buckets.add_buckets(&add_args.name, &None, false)?
                        }
                    }
                }
                _ => {
                    return Err(anyhow!("repo_url is required when name is provided."));
                }
            }
        }

        BucketSubcommands::List(_list_args) => {
            buckets.display_all_buckets_extra(_list_args.global)?;
        }
        BucketSubcommands::Known(_known_args) => {
            buckets.display_known_buckets(_known_args.global)?;
        }
        BucketSubcommands::Rm(rm_args) => {
            println!(
                "{} {} ",
                "准备删除桶:".to_string().blue(),
                &rm_args.name.clone().dark_green().bold()
            );

            buckets.rm_buckets(&rm_args.name, rm_args.global)?;
        }
        BucketSubcommands::Update(_) => {
            update_buckets_parallel()?;
        }
    }
    Ok(())
}
