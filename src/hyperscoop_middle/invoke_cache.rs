use log::info;
use command_util_lib::cache::display_all_cache_info;
use crate::command::CacheArgs;
use crate::command_args::cache::CacheSubcommand::{Rm, Show};

pub  fn execute_cache_command(cache_args: CacheArgs) -> Result<(), anyhow::Error> { 
  match cache_args.command {
    None => { 
        log::info!("cache show command ") ;
        Ok(())
    }
    Some(args ) => { 
      match args {
        Show(_) => {
           display_all_cache_info() ; 
          Ok(())
        }
        Rm(_) => {
          info!("cache rm command ");
          Ok(())
        }
      }
    }
  }
 
}