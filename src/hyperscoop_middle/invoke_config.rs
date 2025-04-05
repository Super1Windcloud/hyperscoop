use command_util_lib::config::* ;
use crate::command_args::config::{ConfigArgs, STR};
use crate::command_args::config::ConfigSubcommand;
pub fn  execute_config_command (args: ConfigArgs) -> Result<(), anyhow::Error> { 
  if let Some(command) = args.command {
    match command {
      ConfigSubcommand::Show(_) => {  
        display_all_config() ;
      }
      ConfigSubcommand::Set( args ) => { 
        set_config_value(&args.name, &args.value) ;
      }
      ConfigSubcommand::Get(arg ) => { 
        get_config_value(&arg.name) ;
      }
      ConfigSubcommand::Rm( args ) => { 
        remove_config_value(&args.name) ;
      }
    }
    Ok(()) 
  } else {  
    if args.config_help{
      println!("{}", STR);
    }
    Ok(())
  }
} 