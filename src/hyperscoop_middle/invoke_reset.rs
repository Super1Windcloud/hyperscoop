use crate::command_args::reset::ResetArgs;
use command_util_lib::reset:: * ;  
use regex::Regex;
pub fn  execute_reset_command (args: ResetArgs) -> Result< (),  anyhow::Error > {  
  if let Some(name) = args.name {   
    if   name .contains('@')  {  
      let count = name.matches('@').count(); 
      if count != 1 {    
        return Err(anyhow::anyhow!("Invalid app name: {}", name));
      }
      let pattern = Regex::new(r"^([a-zA-Z]+)@([a-zA-Z0-9_.-]+)$")?;
      if !pattern.is_match(&name) {
        return Err(anyhow::anyhow!("Invalid app name: {}", name)); 
      }
      let app_name = name.split('@').next().unwrap();
      let app_version = name.split('@').last().unwrap();  
      if app_name.is_empty() || app_version.is_empty() {
        return Err(anyhow::anyhow!("Invalid app name: {}", name));
      }
      reset_specific_version(app_name, app_version ,args.global)?  
    }  
    else {
      reset_latest_version(name ,args.global)?
    }
  }
  
  Ok(()) 
} 
