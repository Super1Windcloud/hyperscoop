use crate::init_env::{get_app_current_bin_path, get_shims_path};
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::{
    ArrayOrDoubleDimensionArray, StringOrArrayOrDoubleDimensionArray,
};
use crate::utils::system::get_system_default_arch;
use anyhow::bail;
use crossterm::style::Stylize;
use gix_object::Exists;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};
const DRIVER_SHIM_BYTES: &[u8] = include_bytes!("..\\bin\\shim.exe");

pub fn create_shim_or_shortcuts(manifest_json: String, app_name: &String) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(manifest_json)?;
    let serde_obj: InstallManifest = serde_json::from_str(&content)?;
    let bin = serde_obj.bin;
    let architecture = serde_obj.architecture;
    let shortcuts = serde_obj.shortcuts;
    if bin.is_some() {
        create_shims_file(bin.unwrap(), app_name)?;
    }
    if shortcuts.is_some() {
        create_start_menu_shortcuts(shortcuts.unwrap(), app_name.into())?;
    }
    if architecture.is_some() {
        let architecture = architecture.unwrap();
        let system_arch = get_system_default_arch()?;
        if system_arch == "64bit" {
            let x64 = architecture.x64bit;
            if x64.is_none() {
                return Ok(());
            }
            let x64 = x64.unwrap();
            let bin = x64.bin;
            if bin.is_none() {
                return Ok(());
            }
            let bin = bin.unwrap();
            create_shims_file(bin, app_name)?;
            let shortcuts = x64.shortcuts;
            if shortcuts.is_none() {
                return Ok(());
            }
            let shortcuts = shortcuts.unwrap();
            create_start_menu_shortcuts(shortcuts, app_name.into())?;
        } else if system_arch == "32bit" {
            let x86 = architecture.x86bit;
            if x86.is_none() {
                return Ok(());
            }
            let x86 = x86.unwrap();
            let bin = x86.bin;
            if bin.is_none() {
                return Ok(());
            }
            let bin = bin.unwrap();
            create_shims_file(bin, app_name)?;
            let shortcuts = x86.shortcuts;
            if shortcuts.is_none() {
                return Ok(());
            }
            let shortcuts = shortcuts.unwrap();
            create_start_menu_shortcuts(shortcuts, app_name.into())?;
        } else if system_arch == "arm64" {
            let arm64 = architecture.arm64;
            if arm64.is_none() {
                return Ok(());
            }
            let arm64 = arm64.unwrap();
            let bin = arm64.bin;
            if bin.is_none() {
                return Ok(());
            }
            let bin = bin.unwrap();
            create_shims_file(bin, app_name)?;
            let shortcuts = arm64.shortcuts;
            if shortcuts.is_none() {
                return Ok(());
            }
            let shortcuts = shortcuts.unwrap();
            create_start_menu_shortcuts(shortcuts, app_name.into())?;
        }
    }
    Ok(())
}

pub fn create_shims_file(
    bin: StringOrArrayOrDoubleDimensionArray,
    app_name: &String,
) -> anyhow::Result<()> {
    let shim_path = get_shims_path();
    match bin {
        StringOrArrayOrDoubleDimensionArray::String(s) => {
            create_default_shim_name_file(s, &shim_path, app_name)?;
        }
        StringOrArrayOrDoubleDimensionArray::StringArray(a) => {
            for item in a {
                create_default_shim_name_file(item, &shim_path, app_name)?;
            }
        }
        StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(a) => {
            for item in a {
                let len = item.len();
                if len == 1 {
                    create_default_shim_name_file((&item[0]).to_string(), &shim_path, app_name)?;
                }
                if len == 2 {
                    let exe_name = item[0].clone();
                    let alias_name = item[1].clone();
                    create_alias_shim_name_file(exe_name, alias_name, &shim_path, app_name, None )?;
                }
                if len == 3 {
                    let exe_name = item[0].clone();
                    let alias_name = item[1].clone();
                    let params = item[2].clone();
                  create_alias_shim_name_file(
                        exe_name, alias_name, &shim_path, app_name ,  Some(params) ,
                    )?;
                }
            }
        }
        StringOrArrayOrDoubleDimensionArray::NestedStringArray(a) => {
            for item in a {
                match item {
                    StringOrArrayOrDoubleDimensionArray::String(s) => {
                        create_default_shim_name_file(s, &shim_path, app_name)?;
                    }
                    StringOrArrayOrDoubleDimensionArray::StringArray(item) => {
                        let len = item.len();
                        if len == 1 {
                            create_default_shim_name_file(
                                (&item[0]).to_string(),
                                &shim_path,
                                app_name,
                            )?;
                        }
                        if len == 2 {
                            let exe_name = item[0].clone();
                            let alias_name = item[1].clone();
                            create_alias_shim_name_file(
                                exe_name, alias_name, &shim_path, app_name, None,
                            )?;
                        }
                        if len == 3 {
                            let exe_name = item[0].clone();
                            let alias_name = item[1].clone();
                            let params = item[2].clone();
                            create_alias_shim_name_file(
                                exe_name,
                                alias_name,
                                &shim_path,
                                app_name,
                                Some(params),
                            )?;

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

pub fn create_start_menu_shortcuts(
    shortcuts: ArrayOrDoubleDimensionArray,
    app_name: String,
) -> anyhow::Result<()> {
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
            let bin_name_with_extension = shortcut[0].clone();
            let shortcut_name = shortcut[1].clone() + ".lnk";
            if shortcut_name.is_empty() {
                return Ok(());
            }
            let scoop_link_home  = r"C:\Users\superuse\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Scoop Apps".to_string();
            let scoop_link_home = PathBuf::from(scoop_link_home);
            if scoop_link_home.exists() {
                let start_menu_link_path = scoop_link_home.join(&shortcut_name);
                if !start_menu_link_path.exists() {
                    let target_path =
                        get_app_current_bin_path(app_name, bin_name_with_extension.clone());
                    start_create_shortcut(
                        start_menu_link_path,
                        target_path,
                        &bin_name_with_extension,
                    )?;
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
                let shortcut_name = shortcut_item[1].clone() + ".lnk";
                if shortcut_name.is_empty() {
                    return Ok(());
                };
                let bin_name_with_extension = shortcut_item[0].clone();
                let scoop_link_home  = r"C:\Users\superuse\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Scoop Apps".to_string();
                let scoop_link_home = PathBuf::from(scoop_link_home);
                if scoop_link_home.exists() {
                    let start_menu_link_path = scoop_link_home.join(&shortcut_name);
                    if !start_menu_link_path.exists() {
                        let target_path = get_app_current_bin_path(
                            app_name.clone(),
                            bin_name_with_extension.clone(),
                        );
                        if !Path::new(&target_path).exists() {
                            bail!(format!("链接目标文件 {target_path} 不存在"))
                        };
                        start_create_shortcut(
                            start_menu_link_path,
                            target_path,
                            &bin_name_with_extension,
                        )?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn start_create_shortcut<P: AsRef<Path>>(
    start_menu_path: P,
    link_target_path: String,
    app_name: &String,
) -> anyhow::Result<()> {
    use mslnk::ShellLink;
    let link = start_menu_path.as_ref().to_str().unwrap();
    println!(
        "{} {} => {}",
        "Creating Shortcuts for".to_string().dark_blue().bold(),
        app_name.to_string().dark_cyan().bold(),
        link.to_string().dark_green().bold()
    );
    let shell_link = ShellLink::new(link_target_path)?;
    shell_link.create_lnk(start_menu_path)?;
    Ok(())
}

pub fn create_alias_shim_name_file(
    exe_name: String,
    alias_name: String,
    shim_dir: &String,
    app_name: &String,
    program_args: Option<String>,
) -> anyhow::Result<()> {
    let out_dir = PathBuf::from(shim_dir);
    let target_path = get_app_current_bin_path(app_name.into(), exe_name);
    if !out_dir.exists() {
        bail!(format!("shim 目录 {shim_dir} 不存在"));
    }
    if !Path::new(&target_path).exists() {
        bail!(format!("链接目标文件 {target_path} 不存在"))
    };
    if program_args.is_some() {
        let program_args = program_args.unwrap();
        create_exe_type_shim_file_and_shim_bin(
            target_path,
            out_dir,
            Some(alias_name),
            Some(program_args),
        )?;
    } else {
        create_exe_type_shim_file_and_shim_bin(target_path, out_dir, Some(alias_name), None)?;
    }
    Ok(())
}


pub fn create_default_shim_name_file(
    exe_name: String,
    shim_dir: &String,
    app_name: &String,
) -> anyhow::Result<()> {
    let out_dir = PathBuf::from(shim_dir); 
    let  temp =exe_name.clone();
    let   suffix = temp. split('.').last().unwrap();
   if suffix.is_empty() {
      bail!(format!("shim 文件名 {exe_name} 后缀为空 WTF?"))
   }
    let target_path = get_app_current_bin_path(app_name.into(), exe_name);
    if !out_dir.exists() {
        bail!(format!("shim 目录 {shim_dir} 不存在"));
    }
    if !Path::new(&target_path).exists() {
        bail!(format!("链接目标文件 {target_path} 不存在"))
    };
  if suffix == "exe"  || suffix == "com" {
    create_exe_type_shim_file_and_shim_bin(target_path, out_dir, None, None)?;
  } else  if suffix == "cmd"  ||"bat" == suffix {

  }else if suffix=="ps1" {
    
  }else if  suffix=="jar" {
    
  }else if suffix=="py" { 
    
  }else { 
    bail!(format!(" 后缀{suffix} 不支持 WTF?"))
  }
    Ok(())
}
pub fn create_exe_type_shim_file_and_shim_bin<P1: AsRef<Path>, P2: AsRef<Path>>(
    target_path: P1,
    output_dir: P2,
    alias_name: Option<String>,
    program_params: Option<String>,
) -> anyhow::Result<()> {
    let target_path = target_path.as_ref().to_str().unwrap();
    let output_dir = output_dir.as_ref().to_path_buf();

    let target_name = if alias_name.is_none() {
        Path::new(target_path)
            .file_stem()
            .and_then(|s| s.to_str().and_then(|s| Some(s.to_lowercase())))
            .ok_or("Invalid target executable name")
    } else {
        Ok(alias_name.unwrap().to_string())
    };

    if target_name.is_err() {
        bail!("Invalid target executable name {}", target_path)
    }
    let content = if program_params.is_none() {
        format!("path = \"{}\"", target_path)
    } else {
        let program_params = program_params.unwrap();
        format!("path = {target_path } \nargs = {program_params}")
    };
    let target_name = target_name.unwrap();
    // Determine the shim file name
    let shim_name = format!("{}.shim", target_name);
    let shim_path = output_dir.join(&shim_name);
    if !shim_path.exists() {
        fs::create_dir_all(&output_dir)?;
    }
    // Write the shim file
    let mut file = File::create(&shim_path)?;
    file.write_all(content.as_bytes())?;
    println!(
        "{} {}",
        "Created  shim  file => ".to_string().dark_blue().bold(),
        &shim_path.to_str().unwrap().dark_green().bold()
    );

    if DRIVER_SHIM_BYTES.is_empty() {
        bail!("origin driver shim.exe not found");
    }
    #[cfg(windows)]
    {
        let exe_name = format!("{}.exe", target_name);
        let output_shim_exe = output_dir.join(&exe_name);
        let parent_dir = output_shim_exe.parent().unwrap();
        if !parent_dir.exists() {
            fs::create_dir_all(&parent_dir)?; // 递归创建所有不存在的父目录
        }
        println!(
            "{} {}",
            "Created  shim  bin =>".dark_blue().bold(),
            &output_shim_exe.display().to_string().dark_green().bold()
        );
        fs::write(&output_shim_exe, DRIVER_SHIM_BYTES)?;
    }
    Ok(())
}

mod test_shim {
    #[allow(unused)]
    use super::*;
    #[test]
    fn test_create_shortcuts() {
        use crate::install::create_start_menu_shortcuts;
        use crate::manifest::install_manifest::InstallManifest;
        let file = r"A:\Scoop\buckets\ScoopMaster\bucket\zigmod.json";
        let content = std::fs::read_to_string(file).unwrap();
        let manifest: InstallManifest = serde_json::from_str(&content).unwrap();
        let shortcuts = manifest.shortcuts.unwrap();
        let app_name = "zigmod".to_string();
        create_start_menu_shortcuts(shortcuts, app_name).unwrap();
    }

    #[test]
    fn test_create_shims() {
        let cwd = std::env::current_dir().unwrap();
        println!("{}", cwd.display());
        let shim_exe = cwd.join("src\\bin\\shim.exe");
        if shim_exe.exists() {
            println!("{:?}", shim_exe);
        }
        let target_path = r#"A:\Scoop\apps\zig\current\zig.exe"#;
        let output_dir = cwd.join("src\\bin\\output");
        let args = "run -h";
        create_exe_type_shim_file_and_shim_bin(target_path, output_dir, None, Some(args.into()))
            .unwrap();
    }
}
