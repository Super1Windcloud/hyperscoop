use crate::command_args::which::WhichArgs;
use anyhow::bail;
use command_util_lib::init_env::{
    get_app_current_dir, get_app_current_dir_global, get_shims_root_dir, get_shims_root_dir_global,
};
use command_util_lib::list::ArchType;
use command_util_lib::manifest::install_manifest::InstallManifest;
use command_util_lib::manifest::manifest_deserialize::StringOrArrayOrDoubleDimensionArray;
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use which::which;

#[derive(Debug, Deserialize, Serialize)]
pub struct InstallJSON {
    pub bucket: Option<String>,
    pub architecture: Option<ArchType>,
}

pub fn execute_which_command(command: WhichArgs) -> Result<(), anyhow::Error> {
    if let Some(name) = command.name {
        let current_dir = if command.global {
            get_app_current_dir_global(&name)
        } else {
            get_app_current_dir(&name)
        };

        let shim_root_dir = if command.global {
            get_shims_root_dir_global()
        } else {
            get_shims_root_dir()
        };

        let result = which(name.as_str()).ok();
        if let Some(path) = result {
            output_current_exe(path, shim_root_dir.as_str())?;
        } else {
            if !Path::new(&current_dir).exists() {
                bail!("{current_dir} is not exists")
            } else {
                let manifest_json = format!("{}\\manifest.json", current_dir);
                if !Path::new(&manifest_json).exists() {
                    bail!("{manifest_json} is not exists")
                } else {
                    let manifest_json_content = std::fs::read_to_string(&manifest_json)?;
                    let manifest: InstallManifest = serde_json::from_str(&manifest_json_content)?;

                    let install_json = format!("{}\\install.json", current_dir);
                    if !Path::new(&install_json).exists() {
                        bail!("{install_json} is not exists")
                    }
                    let install_json_content = std::fs::read_to_string(&install_json)?;
                    let install_json: InstallJSON = serde_json::from_str(&install_json_content)?;
                    let arch = install_json.architecture.unwrap();

                    let bin = manifest.bin;
                    let architecture = manifest.architecture;

                    if bin.is_some() {
                        let bin = bin.unwrap();
                        match_bin_parser(bin, shim_root_dir.to_string())?;
                    } else if architecture.is_some() {
                        let architecture = architecture.unwrap();
                        match arch {
                            ArchType::X86 => {
                                let x86 = architecture.x86bit;
                                if x86.is_some() {
                                    let x86 = x86.unwrap();
                                    let bin = x86.bin;
                                    if bin.is_some() {
                                        let bin = bin.unwrap();
                                        match_bin_parser(bin, shim_root_dir.to_string())?;
                                    }
                                }
                            }
                            ArchType::X64 => {
                                let x64 = architecture.x64bit;
                                if x64.is_some() {
                                    let x64 = x64.unwrap();
                                    let bin = x64.bin;
                                    if bin.is_some() {
                                        let bin = bin.unwrap();
                                        match_bin_parser(bin, shim_root_dir.to_string())?;
                                    }
                                }
                            }
                            ArchType::Arm64 => {
                                let arm64 = architecture.arm64;
                                if arm64.is_some() {
                                    let arm64 = arm64.unwrap();
                                    let bin = arm64.bin;
                                    if bin.is_some() {
                                        let bin = bin.unwrap();
                                        match_bin_parser(bin, shim_root_dir.to_string())?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn output_current_exe(path: PathBuf, shim_root_dir: &str) -> anyhow::Result<()> {
    let parent = path.parent().unwrap();
    if parent.to_str().unwrap() != shim_root_dir {
        println!("{}", path.display().to_string().dark_green().bold());
        return Ok(());
    }
  
    let path = path.to_str().unwrap();

    let splits = path.split(".").collect::<Vec<&str>>();
    if splits.len() != 2 {
        bail!("{path} is not a valid path")
    }
    let prefix = splits[0];
    let suffix = splits[1];
    if suffix == "exe" || suffix == "com" {
        let shim_file = format!("{}.shim", prefix);
        if !Path::new(&shim_file).exists() {
            bail!("{shim_file} is not exists")
        }
        let content = std::fs::read_to_string(shim_file)?;
        println!("{}", content.to_string().dark_green().bold());
    } else if suffix == "cmd" || suffix == "bat" || suffix == "ps1" {
        let cmd_file = format!("{}.cmd", prefix);
        if !Path::new(&cmd_file).exists() {
            bail!("{cmd_file} is not exists")
        }
        let content = extract_rem_comments(cmd_file.as_str());
        println!("{}", content.dark_green().bold());
    } else {
        eprintln!("Unknown suffix: {}", suffix);
        bail!("{path} is not a valid path")
    }

    Ok(())
}

pub fn match_bin_parser(
    bin: StringOrArrayOrDoubleDimensionArray,
    shim_root_dir: String,
) -> anyhow::Result<()> {
    match bin {
        StringOrArrayOrDoubleDimensionArray::String(s) => {
            let result = which(s.as_str()).ok();
            if let Some(path) = result {
                output_current_exe(path, shim_root_dir.as_str())?;
            }
        }
        StringOrArrayOrDoubleDimensionArray::StringArray(a) => {
            for item in a {
                let result = which(item.as_str()).ok();
                if let Some(path) = result {
                    output_current_exe(path, shim_root_dir.as_str())?;
                }
            }
        }

        StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(nested_arrs) => {
            for arr in nested_arrs {
                if arr.len() == 1 {
                    let result = which(arr[0].as_str()).ok();
                    if let Some(path) = result {
                        output_current_exe(path, shim_root_dir.as_str())?;
                    }
                } else if arr.len() == 2 {
                    let result = which(arr[1].as_str()).ok();
                    if let Some(path) = result {
                        output_current_exe(path, shim_root_dir.as_str())?;
                    }
                }
            }
        }
        StringOrArrayOrDoubleDimensionArray::Null => {
            println!("bin is  null");
        }
        StringOrArrayOrDoubleDimensionArray::NestedStringArray(nested_arr) => {
            for nest_arr in nested_arr {
                match nest_arr {
                    StringOrArrayOrDoubleDimensionArray::String(s) => {
                        let result = which(s.as_str()).ok();
                        if let Some(path) = result {
                            output_current_exe(path, shim_root_dir.as_str())?;
                        }
                    }
                    StringOrArrayOrDoubleDimensionArray::StringArray(a) => {
                        for item in a {
                            let result = which(item.as_str()).ok();
                            if let Some(path) = result {
                                output_current_exe(path, shim_root_dir.as_str())?;
                            }
                        }
                    }
                    _ => {
                        println!(" what the fuck bin?   {:?}", nest_arr);
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_rem_comments(file_path: &str) -> String {
    let content = std::fs::read_to_string(file_path).expect("Failed to read file");
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("@rem") {
                Some(trimmed[4..].trim_start().to_string()) // 提取 "@rem" 后的内容
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}



