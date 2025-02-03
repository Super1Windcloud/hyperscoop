use crate::command_args::shim::ShimSubCommand;
use   command_util_lib::shim::* ; 
pub fn execute_shim_command (args: crate::command_args::shim::ShimArgs) -> Result< (), anyhow::Error>   {
  
   if let Some(command) = args.command {
     match command {
       ShimSubCommand::Add( args ) => { execute_add_shim(args.name , args.path)  }
       ShimSubCommand::Rm(_) => {}
       ShimSubCommand::List(args ) => {   
         if args.regex.is_some() {
           list_shims_by_regex(args.regex.unwrap()) ;
         } else { 
           list_all_shims() ; 
         }
       }
       ShimSubCommand::Info(args) => { 
         list_shim_info(args.name) ;
       }
       ShimSubCommand::Alter(_) => {}  
     }
     Ok(())
   } else {
     Ok(())
   }
}