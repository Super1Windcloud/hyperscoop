use std::env;
use crossterm::style::Stylize;
use crate::command_args::shim::ShimSubCommand;
use command_util_lib::shim::*;
use command_util_lib::utils::system::{is_admin, request_admin};

pub fn execute_shim_command(
    args: crate::command_args::shim::ShimArgs,
) -> Result<(), anyhow::Error> { 
   if  args.global  && !is_admin()? {
     let args =env::args().skip(1). collect::<Vec<String>>();
     let  args_str= args.join(" ");
     log::warn!("Global command arguments: {}", args_str.clone().dark_yellow());
     request_admin( args_str.as_str())?;
     return Ok(());
   }
  
    if let Some(command) = args.command {
        match command {
            ShimSubCommand::Add(args) => {
                execute_add_shim(args.name, args.path, args.arguments, args.global)?; 
            }
            ShimSubCommand::Rm(args) => {
                remove_shim(args.name, args.global)?;
            }
            ShimSubCommand::List(args) => {
                if args.regex.is_some() {
                    list_shims_by_regex(args.regex.unwrap(), args.global)? ;
                } else {
                    list_all_shims(args.global)?;
                }
            }
            ShimSubCommand::Info(args) => {
                list_shim_info(args.name, args.global)?;
            }
        }
        Ok(())
    } else {
        Ok(())
    }
}
