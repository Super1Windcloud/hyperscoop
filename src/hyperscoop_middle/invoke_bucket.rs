use crate::command_args::bucket_args::BucketSubcommands;
use anyhow::anyhow;
use command_util_lib::buckets::Buckets;
use crossterm::style::Stylize;

pub async fn execute_bucket_command(args: &Option<BucketSubcommands>) -> Result<(), anyhow::Error> {
    let buckets = Buckets::new();
    let bucket_args = args.as_ref().expect("bucket_args cannot be none");
    match bucket_args {
        BucketSubcommands::Add(add_args) => {
            match (add_args.name.is_some(), add_args.repo_url.is_some()) {
                (true, true) => {
                    buckets
                        .add_buckets(&add_args.name, &add_args.repo_url)
                        .await?;
                }
                (true, false) => {
                    let first = add_args.name.clone().unwrap_or(String::new());
                    if buckets.is_valid_url(&first) {
                        let url =first ; 
                        buckets.add_buckets(&None,  &Some(url )).await?;
                    } else {
                        buckets.add_buckets(&add_args.name, &None).await?;
                    }
                }

                _ => {
                    return Err(anyhow!("repo_url is required when name is provided."));
                }
            }
        }

        BucketSubcommands::List(_list_args) => {
            buckets.display_all_buckets()?;
        }
        BucketSubcommands::Known(_known_args) => {
            buckets.display_known_buckets()?;
        }
        BucketSubcommands::Rm(rm_args) => {
            println!(
                "{} {} ",
                "准备删除桶:".to_string().blue(),
                &rm_args.name.clone().dark_green().bold() 
            );
            buckets.rm_buckets(&rm_args.name).await?;
        }

        BucketSubcommands::Update(_) => {
            crate::hyperscoop_middle::invoke_update::update_buckets().await?;
        }
    }
    Ok(())
}
