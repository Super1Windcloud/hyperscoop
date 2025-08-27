use crate::command::CacheArgs;
use crate::command_args::cache::CacheSubcommand::{Rm, Show};
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
                if sub.all {
                    log::info!("cache rm command ");
                    rm_all_cache(sub.global)?;
                    return Ok(());
                }
                if let Some(app_name) = sub.rm_app {
                    display_specified_cache_info(app_name.as_str(), sub.global)?;
                    Ok(())
                } else {
                    bail!("the following required arguments were not provided: <app>");
                }
            }
        },
    }
}
