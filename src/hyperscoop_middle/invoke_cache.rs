use crate::command::CacheArgs;
use crate::command_args::cache::CacheSubcommand::{Prune, Rm, Show};
use anyhow::bail;
use command_util_lib::cache::*;

pub fn execute_cache_command(cache_args: CacheArgs) -> Result<(), anyhow::Error> {
    match cache_args.command {
        None => {
            log::info!("cache show command ");
            Ok(())
        }
        Some(args) => match args {
            Show(args) => {
                display_all_cache_info(args.global)?;
                Ok(())
            }
            Rm(sub) => {
                if !sub.all && sub.rm_apps.is_empty() {
                    bail!("the following required arguments were not provided: <apps>...");
                }
                let apps = if sub.all || sub.rm_apps.iter().any(|name| name == "*") {
                    &[][..]
                } else {
                    &sub.rm_apps
                };
                clean_cache(sub.global, apps, None, sub.dry_run)
            }
            Prune(sub) => clean_cache(sub.global, &sub.apps, Some(sub.days), sub.dry_run),
        },
    }
}
