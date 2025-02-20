use command_util_lib::arch_specific_field;
use std::path::Path;
use anyhow::bail;
use command_util_lib::install::* ;
use crate::command_args::install::InstallArgs;
use command_util_lib::manifest::install_manifest::* ; 
pub  async fn  execute_install_command(args: InstallArgs) -> Result< () , anyhow::Error>{
  if args.app_name.is_none() { 
    return  Ok(());
  }
  let app_name = args.app_name.unwrap();
  if  Path::new(&app_name).exists()  && Path::new(&app_name).is_file()  {
     log::trace!("manifest file {}" , app_name);  
    if  args.arch.is_some() {
      install_app_from_local_manifest_file(   &app_name ,  Some(args.arch.unwrap()) ).await?;

    } else { 
      install_app_from_local_manifest_file(   &app_name,  None ).await?;
    }
    return Ok(()); 
  } 
  if app_name.contains("/"){ 
    if  app_name.contains('@') { 
      bail!("指定的App格式不正确") 
    }
    let split_arg = app_name.split('/').collect::<Vec<&str>>();
    if split_arg.iter().count() ==2 {
      let  bucket = split_arg[0];
      let app_name = split_arg[1];
      if  bucket.is_empty() || app_name.is_empty() {
        bail!("指定的App格式不正确")
      }
      install_from_specific_bucket(bucket , app_name ).await?;
      return Ok(());
    }else if split_arg.iter().count()>2 || split_arg.len() == 1  {
      bail!("指定的APP格式错误")
    }
  }
  if  app_name.contains('@') {
    let split_version =app_name.split('@').collect::<Vec<&str>>();
    if split_version.iter().count() ==2 {
      let app_name = split_version[0];
      let app_version = split_version[1];
      if app_name.is_empty() || app_version.is_empty() {
        bail!("指定的APP格式错误")
      }
      install_app_specific_version(app_name, app_version).await?;
      return Ok(());
    }else if    split_version.len()==1 || split_version.len()  >2  { 
      bail!("指定的APP格式错误")
    }
  }
 install_app (app_name.as_str()  ).await? ; 
  Ok(())
 }
