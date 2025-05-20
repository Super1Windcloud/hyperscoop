use crate::command::BucketArgs;
use crate::command_args::bucket_args::BucketSubcommands;
use crate::hyperscoop_middle::invoke_update::update_buckets_parallel;
use anyhow::anyhow;
use command_util_lib::buckets::Buckets;
use command_util_lib::utils::system::{is_admin, request_admin};
use command_util_lib::utils::utility::{
    get_official_bucket_urls, get_official_with_social_bucket_urls,
};
use crossterm::style::Stylize;
use std::env;

pub fn execute_bucket_command(args: BucketArgs) -> Result<(), anyhow::Error> {
    let buckets = Buckets::new()?;
    let global = args.global;

    if global && !is_admin()? {
        let args = env::args().skip(1).collect::<Vec<String>>();
        let args_str = args.join(" ");
        log::warn!(
            "Global command arguments: {}",
            args_str.clone().dark_yellow()
        );
        request_admin(args_str.as_str())?;
        return Ok(());
    }
    execute_init_bucket_command(&args, &buckets, global)?;

    if args.command.is_none() {
        return Ok(());
    }
    let command = args.command;

    let bucket_args = command.as_ref().expect("bucket_args cannot be none");
    match bucket_args {
        BucketSubcommands::Add(add_args) => {
            match (add_args.name.is_some(), add_args.repo_url.is_some()) {
                (true, true) => {
                    if add_args.global {
                        buckets.add_buckets(
                            add_args.name.clone(),
                            add_args.repo_url.clone(),
                            true,
                        )?
                    } else {
                        buckets.add_buckets(
                            add_args.name.clone(),
                            add_args.repo_url.clone(),
                            false,
                        )?
                    }
                }
                (true, false) => {
                    let first = add_args.name.clone().unwrap_or(String::new());
                    if buckets.is_valid_url(&first) {
                        let url = first;
                        if add_args.global {
                            buckets.add_buckets(None, Some(url), true)?
                        } else {
                            buckets.add_buckets(None, Some(url), false)?
                        }
                    } else {
                        if add_args.global {
                            buckets.add_buckets(add_args.name.clone(), None, true)?
                        } else {
                            buckets.add_buckets(add_args.name.clone(), None, false)?
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

fn execute_init_bucket_command(
    args: &BucketArgs,
    bucket: &Buckets,
    global: bool,
) -> anyhow::Result<()> {
    if args.init_office_bucket {
        if args.init_official_bucket_with_social {
            let official_with_social_bucket_urls = get_official_with_social_bucket_urls();
            let result = official_with_social_bucket_urls
                .into_iter()
                .try_for_each(|url| {
                    if global {
                        bucket.add_buckets(None, Some(url.into()), true)?;
                    } else {
                        bucket.add_buckets(None, Some(url.into()), false)?;
                    }
                    Ok(())
                }) as anyhow::Result<()>;
            if result.is_err() {
                return Err(anyhow!("Failed to init official bucket at line 103")
                    .context(result.unwrap_err()));
            }
        } else {
            let official_bucket_urls = get_official_bucket_urls();
            let result = official_bucket_urls.into_iter().try_for_each(|url| {
                if global {
                    bucket.add_buckets(None, Some(url.into()), true)?;
                } else {
                    bucket.add_buckets(None, Some(url.into()), false)?;
                }
                Ok(())
            }) as anyhow::Result<()>;
            if result.is_err() {
                return Err(anyhow!("Failed to init official bucket at line 125")
                    .context(result.unwrap_err()));
            }
        }
    } else if args.init_official_bucket_with_social {
        let official_with_social_bucket_urls = get_official_with_social_bucket_urls();
        let result = official_with_social_bucket_urls
            .into_iter()
            .try_for_each(|url| {
                if global {
                    bucket.add_buckets(None, Some(url.into()), true)?;
                } else {
                    bucket.add_buckets(None, Some(url.into()), false)?;
                }
                Ok(())
            }) as anyhow::Result<()>;
        if result.is_err() {
            return Err(
                anyhow!("Failed to init official bucket at line 141").context(result.unwrap_err())
            );
        }
    }

    Ok(())
}
