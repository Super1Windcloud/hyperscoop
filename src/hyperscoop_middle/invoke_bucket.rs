use crate::command_args::bucket_args::BucketSubcommands;
use anyhow::anyhow;
use command_util_lib::buckets::Buckets;
use crossterm::style::Stylize;
use crate::hyperscoop_middle::invoke_update::update_buckets_parallel;

pub   fn execute_bucket_command(args: &Option<BucketSubcommands>) -> Result<(), anyhow::Error> {
    let buckets = Buckets::new()?;
   
    let bucket_args = args.as_ref().expect("bucket_args cannot be none");
    match bucket_args {
        BucketSubcommands::Add(add_args) => {
            match (add_args.name.is_some(), add_args.repo_url.is_some()) {
                (true, true) => {
                    if add_args.global {
                        buckets
                            .add_buckets(&add_args.name, &add_args.repo_url, true)?
                            
                    } else {
                        buckets
                            .add_buckets(&add_args.name, &add_args.repo_url, false)?
                            
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
                buckets.display_all_buckets_extra(_list_args.global )?;
        }
        BucketSubcommands::Known(_known_args) => { 
              buckets.display_known_buckets(_known_args.global )?;
        }
        BucketSubcommands::Rm(rm_args) => {
            println!(
                "{} {} ",
                "准备删除桶:".to_string().blue(),
                &rm_args.name.clone().dark_green().bold()
            ); 
            
            buckets.rm_buckets(&rm_args.name ,rm_args.global)?;
        }
        BucketSubcommands::Update(_) => {
             update_buckets_parallel()?;
        }
    }
    Ok(())
}
