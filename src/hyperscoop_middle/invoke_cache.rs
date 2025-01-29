use command_util_lib::cache::*;
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
        Rm(sub ) => {     
            if sub.all {
              log::info!("cache rm command ") ; 
              rm_all_cache(); 
              return Ok(());
            }
          if let Some(app_name) = sub.rm_app {
            display_specified_cache_info(app_name) ;
            Ok(())
          }
          else {  
            log::warn!("the following required arguments were not provided: <app>") ;  
            Ok(())
          }
        }
      }
    }
  }
 
}