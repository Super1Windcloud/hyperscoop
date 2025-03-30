use std::path::{Path, PathBuf};
use anyhow::bail;
use crossterm::style::Stylize;
use crate::init_env::{get_app_current_bin_path , get_shims_path};
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::{ArrayOrDoubleDimensionArray, StringOrArrayOrDoubleDimensionArray};
use crate::utils::system::get_system_default_arch;

pub fn  create_shim_or_shortcuts(manifest_json : String, app_name : &String) ->anyhow::Result<()>{
   let   content  = std::fs::read_to_string(manifest_json)?;
   let   serde_obj: InstallManifest = serde_json::from_str(&content)?;
    let   bin = serde_obj.bin;
    let  architecture = serde_obj.architecture;
    let  shortcuts = serde_obj.shortcuts;
   if bin.is_some(){
     create_shims_file( bin.unwrap())? ;
   }
  if  shortcuts.is_some(){
     create_start_menu_shortcuts(shortcuts .unwrap(), app_name.into() )? ;
   }
  if architecture.is_some(){
     let  architecture = architecture.unwrap();
    let  system_arch =  get_system_default_arch()?;
    if  system_arch =="64bit" {
       let   x64 = architecture.x64bit;
       if  x64.is_none(){ return Ok(())}
      let x64 = x64.unwrap();
      let bin = x64.bin;
      if bin.is_none() { return Ok(())}
      let bin = bin.unwrap();
      create_shims_file(bin)? ;
      let   shortcuts = x64.shortcuts;
      if  shortcuts.is_none() { return Ok(())}
      let shortcuts = shortcuts.unwrap();
      create_start_menu_shortcuts(shortcuts , app_name .into() )? ;
    }else if  system_arch =="32bit" {
      let   x86 = architecture.x86bit;
      if x86.is_none(){ return Ok(())}
      let x86 = x86.unwrap();
      let bin = x86.bin;
      if bin.is_none() { return Ok(())}
      let bin = bin.unwrap();
      create_shims_file(bin)? ;
      let   shortcuts = x86.shortcuts;
      if  shortcuts.is_none() { return Ok(())}
      let shortcuts = shortcuts.unwrap();
      create_start_menu_shortcuts(shortcuts ,app_name.into() )? ;
    }else if  system_arch =="arm64" {
      let arm64  = architecture.arm64;
      if arm64.is_none(){ return Ok(())}
      let arm64 = arm64.unwrap();
      let bin = arm64.bin;
      if bin.is_none() { return Ok(())}
      let bin = bin.unwrap();
      create_shims_file(bin)? ;
      let   shortcuts = arm64.shortcuts;
      if  shortcuts.is_none() { return Ok(())}
      let shortcuts = shortcuts.unwrap();
      create_start_menu_shortcuts(shortcuts ,app_name.into())? ;
    }
  }
  Ok(())
}

pub  fn  create_shims_file(bin  : StringOrArrayOrDoubleDimensionArray) ->anyhow::Result<()>{
   let shim_path = get_shims_path();
  match bin {
    StringOrArrayOrDoubleDimensionArray::String(s) => {
       create_default_shim_name_file(s, &shim_path)?;
    }
    StringOrArrayOrDoubleDimensionArray::StringArray(a) => {
      for item in a {
        create_default_shim_name_file(item, &shim_path)?;
      }
    }
    StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(a) => {
      for item in a {
        let len = item.len();
        if len == 1 {
          create_default_shim_name_file((&item[0]).to_string(), &shim_path)?;
        }
        if len == 2 || len == 3 {
          let exe_name = item[0].clone();
          let alias_name = item[1].clone();
          create_alias_shim_name_file(exe_name, alias_name, &shim_path)?;
        }
      }
    }
    StringOrArrayOrDoubleDimensionArray::NestedStringArray(a) => {
      for item in a {
        match item {
          StringOrArrayOrDoubleDimensionArray::String(s) => {
            create_default_shim_name_file(s, &shim_path)?;
          }
          StringOrArrayOrDoubleDimensionArray::StringArray(item) => {
            let len = item.len();
            if len == 1 {
              create_default_shim_name_file((&item[0]).to_string(), &shim_path)?;
            }
            if len == 2 || len == 3 {
              let exe_name = item[0].clone();
              let alias_name = item[1].clone();
              create_alias_shim_name_file(exe_name, alias_name, &shim_path)?;
            }
          }
          _ => {
            println!(" what the fuck bin?   {:?}", item);
          }
        }
      }
    }
    _ => {
      bail!("WTF? can't parser this bin object type ")
    }
  }
  Ok(())
}

fn create_alias_shim_name_file(exe_name : String, alias_name: String, shim_dir: &String  ) -> anyhow::Result<()>{


  Ok(())
}

fn create_default_shim_name_file(exe_name  : String,   shim_dir  : &String) ->  anyhow::Result<()>{


  Ok(())
}

pub  fn  create_start_menu_shortcuts(shortcuts : ArrayOrDoubleDimensionArray , app_name :String ) ->anyhow::Result<()>{

  match shortcuts {
    ArrayOrDoubleDimensionArray::Null => return Ok(()),
    ArrayOrDoubleDimensionArray::StringArray(shortcut) => {
      let arg_len = shortcut.len();
      if arg_len < 2 {
        eprintln!(
          "{} ",
          "Failed to find shortcut, maybe manifest json file format error"
            .dark_yellow()
            .bold()
        );
      }
      let bin_name_with_extension  = shortcut[0].clone();
      let shortcut_name = shortcut[1].clone()+".lnk";
      if shortcut_name.is_empty() {
        return Ok(());
      }
      let scoop_link_home  = r"C:\Users\superuse\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Scoop Apps".to_string();
       let  scoop_link_home = PathBuf::from(scoop_link_home);
        if scoop_link_home.exists()   {
          let start_menu_link_path = scoop_link_home.join(&shortcut_name);
          if !start_menu_link_path.exists()    {
            let  target_path  =  get_app_current_bin_path( app_name ,bin_name_with_extension.clone() ) ;
             start_create_shortcut(start_menu_link_path  ,target_path , &bin_name_with_extension) ? ;
          }
      }

    }
    ArrayOrDoubleDimensionArray::DoubleDimensionArray(shortcut) => {
      let arg_len = shortcut.len();
      if arg_len < 1 {
        eprintln!(
          "{} ",
          "Failed to find shortcut, maybe manifest json file format error"
            .dark_yellow()
            .bold()
        );
      }
      for shortcut_item in shortcut {
        let arg_len = shortcut_item.len();
        if arg_len < 2 {
          eprintln!(
            "{} ",
            "Failed to find shortcut, maybe manifest json file format error"
              .dark_yellow()
              .bold()
          );
        }
        let shortcut_name = shortcut_item[1].clone()+".lnk";
        if shortcut_name.is_empty() {
          return Ok(());
        } ;
        let bin_name_with_extension  = shortcut_item[0].clone();
        let scoop_link_home  = r"C:\Users\superuse\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Scoop Apps".to_string();
        let  scoop_link_home = PathBuf::from(scoop_link_home);
        if scoop_link_home.exists()   {
          let start_menu_link_path = scoop_link_home.join(&shortcut_name);
          if !start_menu_link_path.exists()    {
            println!(
              "Creating start menu shortcut '{}'",
              start_menu_link_path.display().to_string().dark_green().bold()
            );
            let  target_path  =  get_app_current_bin_path( app_name.clone()  , bin_name_with_extension.clone()) ;
            if  !Path::new(&target_path).exists() {
              bail!(format!("链接目标文件 {target_path} 不存在"))
            };
            start_create_shortcut(start_menu_link_path  ,target_path ,&bin_name_with_extension ) ? ;
          }
        }
      }
    }
  }

  Ok(())
}


fn start_create_shortcut<P: AsRef<Path>>(start_menu_path : P, link_target_path :String ,app_name :&String    ) -> anyhow::Result<()> {
  use mslnk::ShellLink;
   let  link = start_menu_path.as_ref().to_str().unwrap() ;
  println!("{} {} => {}","Created Shortcuts for".to_string().dark_blue().bold(),
           app_name.to_string().dark_cyan(), link.to_string().dark_green().bold());
   let   shell_link =ShellLink::new(link_target_path)?;
   shell_link.create_lnk(start_menu_path)?;
   Ok(())
}


mod test{
  use crate::install::create_start_menu_shortcuts;
  use crate::manifest::install_manifest::InstallManifest;

  #[test]
  fn test_create_shortcuts(){
    let  file = r"A:\Scoop\buckets\ScoopMaster\bucket\zigmod.json" ;
    let  content  = std::fs::read_to_string(file).unwrap();
    let manifest  : InstallManifest = serde_json::from_str(&content).unwrap();
    let  shortcuts = manifest.shortcuts.unwrap();
    let app_name =  "zigmod".to_string() ;
     create_start_menu_shortcuts(shortcuts, app_name ).unwrap();
  }

  #[test]
  fn  test_create_shims(){

  }
}
