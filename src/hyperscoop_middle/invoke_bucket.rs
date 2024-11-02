use crate::comand_args::bucket_args::BucketSubcommands;
use anyhow::anyhow;
use command_util_lib::buckets::Buckets;

//解析bucket 命令的参数，并执行相应的操作
pub async fn execute_bucket_command(args: &Option<BucketSubcommands>) -> Result<(), anyhow::Error> {
  let mut buckets = Buckets::new();
  let bucket_args = args.as_ref().expect("bucket_args cannot be none");
  match bucket_args {
    BucketSubcommands::Add(add_args) => {
      match (add_args.name.is_some(), add_args.repo_url.is_some()) {
        (true, true) => {
          buckets.add_buckets(&add_args.name, &add_args.repo_url).await?;
        }
        (true, false) => {
          buckets.add_buckets(&None, &add_args.name).await?;
        }

        _ => {
          return Err(anyhow!("repo_url is required when name is provided."));
        }
      }
    }

    BucketSubcommands::List(list_args) => {
      buckets.display_all_buckets()?;
    }
    BucketSubcommands::Known(known_args) => {
      buckets.display_known_buckets()?;
    }
    BucketSubcommands::Rm(rm_args) => {
      println!("{:?}", rm_args);
    }
  }
  Ok(())
}
