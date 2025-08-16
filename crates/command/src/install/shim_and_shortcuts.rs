use crate::init_env::{get_app_current_bin_path, get_shims_root_dir, get_shims_root_dir_global};
use crate::install::InstallOptions;
use crate::install::InstallOptions::InteractiveInstall;
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::{
    ArrayOrDoubleDimensionArray, StringOrArrayOrDoubleDimensionArray,
};
use crate::utils::system::get_system_default_arch;
use crate::utils::utility::{
    assume_yes_to_cover_shortcuts, exclude_scoop_self_scripts, strip_extended_prefix,
    target_version_dir_to_current_dir, write_utf8_file,
};
use anyhow::{bail, Context};
use crossterm::style::Stylize;
use shortcuts_rs::ShellLink;
use std::fs;
use std::path::{Path, PathBuf};
use textwrap::LineEnding;

const DRIVER_SHIM_BYTES: &[u8] = include_bytes!("..\\bin\\shim.exe");

pub fn create_shim_or_shortcuts(
    manifest_json: &str,
    app_name: &str,
    options: &Box<[InstallOptions]>,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(manifest_json).context(format!(
        "Failed to read manifest file: {} at line 25",
        manifest_json
    ))?;
    let serde_obj: InstallManifest = serde_json::from_str(&content).context(format!(
        "Failed to parse manifest file: {} at line 27",
        manifest_json
    ))?;
    let bin = serde_obj.bin;
    let architecture = serde_obj.architecture;
    let shortcuts = serde_obj.shortcuts;

    if bin.is_some() {
        create_shims_file(bin.unwrap(), app_name, options)?;
    }
    if shortcuts.is_some() {
        create_start_menu_shortcuts(shortcuts.unwrap(), app_name.into(), options)?;
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
            create_shims_file(bin, app_name, options)?;
            let shortcuts = x64.shortcuts;
            if shortcuts.is_none() {
                return Ok(());
            }
            let shortcuts = shortcuts.unwrap();
            create_start_menu_shortcuts(shortcuts, app_name.into(), options)?;
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
            create_shims_file(bin, app_name, options)?;
            let shortcuts = x86.shortcuts;
            if shortcuts.is_none() {
                return Ok(());
            }
            let shortcuts = shortcuts.unwrap();
            create_start_menu_shortcuts(shortcuts, app_name.into(), options)?;
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
            create_shims_file(bin, app_name, options)?;
            let shortcuts = arm64.shortcuts;
            if shortcuts.is_none() {
                return Ok(());
            }
            let shortcuts = shortcuts.unwrap();
            create_start_menu_shortcuts(shortcuts, app_name.into(), options)?;
        }
    }
    Ok(())
}

pub fn create_shims_file(
    bin: StringOrArrayOrDoubleDimensionArray,
    app_name: &str,
    options: &Box<[InstallOptions]>,
) -> anyhow::Result<()> {
    let shim_path = if options.contains(&InstallOptions::Global) {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    if !Path::new(&shim_path).exists() {
        fs::create_dir_all(&shim_path).context(format!(
            "Failed to create shims root dir: {} at line 116",
            shim_path
        ))?;
    }
    match bin {
        StringOrArrayOrDoubleDimensionArray::String(s) => {
            create_default_shim_name_file(s, &shim_path, app_name, options)?;
        }
        StringOrArrayOrDoubleDimensionArray::StringArray(a) => {
            for item in a {
                create_default_shim_name_file(item, &shim_path, app_name, options)?;
            }
        }
        StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(a) => {
            for item in a {
                let len = item.len();
                if len == 1 {
                    create_default_shim_name_file(
                        (&item[0]).to_string(),
                        &shim_path,
                        app_name,
                        options,
                    )?;
                } else if len == 2 {
                    let exe_name = item[0].clone();
                    let alias_name = item[1].clone();
                    create_alias_shim_name_file(
                        exe_name, alias_name, &shim_path, app_name, None, options,
                    )?;
                } else if len == 3 {
                    let exe_name = item[0].clone();
                    let alias_name = item[1].clone();
                    let params = item[2].clone();
                    create_alias_shim_name_file(
                        exe_name,
                        alias_name,
                        &shim_path,
                        app_name,
                        Some(params),
                        options,
                    )?;
                } else {
                    eprintln!(" what the fuck bin?   {:?}", item)
                }
            }
        }
        StringOrArrayOrDoubleDimensionArray::NestedStringArray(a) => {
            for item in a {
                match item {
                    StringOrArrayOrDoubleDimensionArray::String(s) => {
                        create_default_shim_name_file(s, &shim_path, app_name, options)?;
                    }
                    StringOrArrayOrDoubleDimensionArray::StringArray(item) => {
                        let len = item.len();
                        if len == 1 {
                            create_default_shim_name_file(
                                (&item[0]).to_string(),
                                &shim_path,
                                app_name,
                                options,
                            )?;
                        } else if len == 2 {
                            let exe_name = item[0].clone();
                            let alias_name = item[1].clone();
                            create_alias_shim_name_file(
                                exe_name, alias_name, &shim_path, app_name, None, options,
                            )?;
                        } else if len == 3 {
                            let exe_name = item[0].clone();
                            let alias_name = item[1].clone();
                            let params = item[2].clone();
                            create_alias_shim_name_file(
                                exe_name,
                                alias_name,
                                &shim_path,
                                app_name,
                                Some(params),
                                options,
                            )?;
                        } else {
                            eprintln!("what the fuck bin?   {:?}", item)
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
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let global = if options.contains(&InstallOptions::Global) {
        true
    } else {
        false
    };
    let user_name = std::env::var("USERNAME").unwrap_or_else(|_| "Default".to_string());
    let scoop_link_home = if global {
        r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Scoop Apps".into()
    } else {
        format!(
            r"C:\Users\{user_name}\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Scoop Apps"
        )
    };
    if !Path::new(&scoop_link_home).exists() {
        fs::create_dir_all(&scoop_link_home).context(format!(
            "Failed to create scoop link home dir: {} at line 232",
            scoop_link_home
        ))?;
    }
    match shortcuts {
        ArrayOrDoubleDimensionArray::Null => return Ok(()),
        ArrayOrDoubleDimensionArray::StringArray(shortcut) => {
            let arg_len = shortcut.len();
            if arg_len < 2 {
                bail!("Error :  manifest shortcuts config format error, arg_len <2");
            }
            let bin_name_with_extension = shortcut[0].clone();
            if bin_name_with_extension.is_empty() {
                bail!("Error : shortcuts target link  cannot be empty")
            }
            let shortcut_name = shortcut[1].clone() + ".lnk";
            if shortcut_name.is_empty() {
                bail!("Error : shortcut name cannot be empty")
            }
            let start_parameters = if arg_len == 3 || arg_len == 4 {
                shortcut[2].trim().to_string()
            } else {
                "".to_string()
            };
            let scoop_link_home = PathBuf::from(scoop_link_home);
            if scoop_link_home.exists() {
                let start_menu_link_path = scoop_link_home.join(&shortcut_name);
                let target_path =
                    get_app_current_bin_path(app_name.as_str(), &bin_name_with_extension, options);
                start_create_shortcut(
                    start_menu_link_path,
                    target_path,
                    &bin_name_with_extension,
                    start_parameters,
                    options,
                )?;
            }
        }
        ArrayOrDoubleDimensionArray::DoubleDimensionArray(shortcut) => {
            let arg_len = shortcut.len();
            if arg_len < 1 {
                bail!("Failed to find shortcut field, manifest json file format error");
            }
            for shortcut_item in shortcut {
                let arg_len = shortcut_item.len();
                if arg_len < 2 {
                    bail!("Error :  manifest shortcuts config format error, arg_len <2");
                }
                let shortcut_name = shortcut_item[1].clone() + ".lnk";
                if shortcut_name.is_empty() {
                    bail!("Error : shortcut name cannot be empty")
                };
                let bin_name_with_extension = shortcut_item[0].clone();
                if bin_name_with_extension.is_empty() {
                    bail!("Error : shortcuts target link  cannot be empty")
                }
                let start_parameters = if arg_len == 3 || arg_len == 4 {
                    shortcut_item[2].trim().to_string()
                } else {
                    "".to_string()
                };
                let scoop_link_home = PathBuf::from(&scoop_link_home);
                if scoop_link_home.exists() {
                    let start_menu_link_path = scoop_link_home.join(&shortcut_name);
                    let target_path = get_app_current_bin_path(
                        app_name.as_str(),
                        &bin_name_with_extension,
                        options,
                    );
                    if !Path::new(&target_path).exists() {
                        bail!(format!("链接目标文件 {target_path} 不存在"))
                    };
                    start_create_shortcut(
                        start_menu_link_path,
                        target_path,
                        &bin_name_with_extension,
                        start_parameters,
                        options,
                    )?;
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
    start_parameters: String,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let args = if start_parameters.is_empty() {
        None
    } else {
        Some(start_parameters)
    };
    let link_path = start_menu_path.as_ref().to_path_buf();
    let link_alias_name = link_path.file_stem().unwrap().to_str().unwrap();
    if link_path.exists() && options.contains(&InteractiveInstall) {
        let result = assume_yes_to_cover_shortcuts(link_alias_name)?;
        if result {
            fs::remove_file(start_menu_path.as_ref())
                .context("Failed to remove  old  start_menu_path at line 334".to_string())?;
        } else {
            return Ok(());
        }
    }

    println!(
        "{} '{}' => '{}'",
        "Creating  Shortcuts for".to_string().dark_blue().bold(),
        app_name.to_string().dark_cyan().bold(),
        link_alias_name.to_string().dark_green().bold()
    );
    if link_path.exists() {
        return Ok(());
    }
    let shell_link = ShellLink::new(link_target_path, args, None, None)?;
    let parent = start_menu_path.as_ref().parent().unwrap();
    if !parent.exists() {
        fs::create_dir_all(parent).context("Failed to create link parent directory at line 353")?;
    };
    shell_link
        .create_lnk(start_menu_path)
        .context("Create shell_link shortcuts failed  at line 357")?;
    Ok(())
}

///   *-------------------------------------------*
pub fn create_alias_shim_name_file(
    exe_name: String,
    alias_name: String,
    shim_dir: &str,
    app_name: &str,
    program_args: Option<String>,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let out_dir = PathBuf::from(shim_dir);
    let suffix = if exe_name.contains(".") {
        exe_name.split('.').last().unwrap().to_lowercase()
    } else {
        "".to_string()
    };

    let target_path = get_app_current_bin_path(app_name.into(), &exe_name, options);

    let target_path = fs::canonicalize(&target_path).context(format!(
        "Failed to get canonicalize target_path {target_path} at line 377"
    ))?;
    let target_path = strip_extended_prefix(target_path.as_path());
    let target_path = target_version_dir_to_current_dir(&target_path, options)?;

    if !out_dir.exists() {
        bail!(format!("shim 目录 {shim_dir} 不存在"));
    }
    if !Path::new(&target_path).exists() {
        bail!(format!("链接目标文件 {target_path} 不存在"))
    };
    log::info!("origin name {}, alias name {}", exe_name, alias_name);

    if suffix == "exe" || suffix == "com" {
        create_exe_type_shim_file_and_shim_bin(
            target_path,
            out_dir,
            Some(alias_name),
            program_args,
            options,
        )?;
    } else if suffix == "cmd" || "bat" == suffix {
        let result = exclude_scoop_self_scripts(&exe_name, Some(alias_name.as_str()))?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{exe_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        create_cmd_or_bat_shim_scripts(
            target_path.as_str(),
            out_dir,
            Some(alias_name),
            program_args,
            options,
        )?;
    } else if suffix == "ps1" {
        let result = exclude_scoop_self_scripts(&exe_name, Some(alias_name.as_str()))?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{exe_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        create_ps1_shim_scripts(
            &target_path,
            out_dir,
            Some(alias_name),
            program_args,
            options,
        )?;
    } else if suffix == "jar" {
        create_jar_shim_scripts(
            &target_path,
            out_dir,
            Some(alias_name),
            program_args,
            options,
        )?;
    } else if suffix == "py" {
        create_py_shim_scripts(
            &target_path,
            out_dir,
            Some(alias_name),
            program_args,
            options,
        )?;
    } else if suffix.is_empty() {
        // shell script file name , no extension
        let result = exclude_scoop_self_scripts(&exe_name, Some(alias_name.as_str()))?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{exe_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        create_shell_shim_scripts(
            target_path.as_str(),
            out_dir,
            Some(alias_name),
            program_args,
            options,
        )?;
    } else {
        bail!(format!(" 后缀{suffix}类型文件不支持, WTF?"))
    }

    Ok(())
}

///   *-------------------------------------------*
pub fn create_default_shim_name_file(
    exe_name: String,
    shim_dir: &str,
    app_name: &str,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let out_dir = PathBuf::from(shim_dir);
    let suffix = if !exe_name.contains(".") {
        "".into()
    } else {
        exe_name.split('.').last().unwrap().to_lowercase()
    };
    if app_name == "hp" {
        return Ok(());
    }
    let target_path = get_app_current_bin_path(app_name.into(), &exe_name, options);
    let target_path = fs::canonicalize(&target_path).context(format!(
        "Failed to get canonicalize target_path {target_path} at line 459"
    ))?;
    let target_path = strip_extended_prefix(target_path.as_path());
    let target_path = target_version_dir_to_current_dir(&target_path, options)?;

    if !out_dir.exists() {
        bail!(format!("shim 目录 {shim_dir} 不存在"));
    }
    if !Path::new(&target_path).exists() && exe_name != "hp.exe" {
        bail!(format!("链接目标文件 {target_path} 不存在"))
    };
    if suffix == "exe" || suffix == "com" {
        create_exe_type_shim_file_and_shim_bin(target_path, out_dir, None, None, options)?;
    } else if suffix == "cmd" || "bat" == suffix {
        let result = exclude_scoop_self_scripts(&exe_name, None)?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{exe_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        create_cmd_or_bat_shim_scripts(target_path.as_str(), out_dir, None, None, options)?;
    } else if suffix == "ps1" {
        let result = exclude_scoop_self_scripts(&exe_name, None)?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{exe_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        create_ps1_shim_scripts(&target_path, out_dir, None, None, options)?;
    } else if suffix == "jar" {
        create_jar_shim_scripts(&target_path, out_dir, None, None, options)?;
    } else if suffix == "py" {
        create_py_shim_scripts(&target_path, out_dir, None, None, options)?;
    } else if suffix.is_empty() {
        // shell script file name , no extension
        let result = exclude_scoop_self_scripts(&exe_name, None)?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{exe_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        create_shell_shim_scripts(target_path.as_str(), out_dir, None, None, options)?;
    } else {
        bail!(format!(" 后缀{suffix}类型文件不支持, WTF?"))
    }
    Ok(())
}

pub fn create_py_shim_scripts(
    target_path: &str,
    out_shim_dir: PathBuf,
    alias_name: Option<String>,
    program_args: Option<String>,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let target_name = if alias_name.is_none() {
        Path::new(&target_path)
            .file_stem()
            .and_then(|s| s.to_str().and_then(|s| Some(s.to_lowercase())))
            .ok_or("Invalid target executable name")
    } else {
        let alias_name = alias_name.unwrap();
        Ok(if alias_name.contains('.') {
            eprintln!("alias_name {} 包含 . 字符, 自动去除扩展名", alias_name);
            alias_name.split('.').next().unwrap().to_string()
        } else {
            alias_name
        })
    };
    if target_name.is_err() {
        let target_name = target_name.unwrap();
        bail!("Invalid target executable name {target_path} \n Error TargetName :{target_name}")
    }
    let target_name = target_name.unwrap();
    let out_shim_dir: Result<&str, anyhow::Error> = if out_shim_dir.exists() {
        Ok(out_shim_dir.to_str().unwrap())
    } else {
        bail!("shim 根目录 {} 不存在", out_shim_dir.display());
    };
    let out_shim_dir = out_shim_dir?;
    let shim_cmd_script = format!("{out_shim_dir}\\{target_name}.cmd");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_cmd_script.to_string().dark_green().bold()
    );
    let shim_shell_script = format!("{out_shim_dir}\\{target_name}");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_shell_script.to_string().dark_green().bold()
    );

    let cmd_content = if program_args.is_none() {
        format!(
            r#"@rem {}
@python "{}"  %*"#,
            target_path, target_path
        )
    } else {
        let arg = program_args.clone().unwrap();
        format!(
            r#"@rem {}
@python "{}" {} %*"#,
            target_path, target_path, arg
        )
    };
    write_utf8_file(&shim_cmd_script, &cmd_content, options)?;

    let sh_content = if program_args.is_some() {
        let arg = program_args.unwrap();
        format!(
            r#"#!/bin/sh
# {}
python.exe "{}" {} "$@""#,
            target_path, target_path, arg
        )
    } else {
        format!(
            r#"#!/bin/sh
# {}
python.exe "{}"  "$@""#,
            target_path, target_path
        )
    };
    write_utf8_file(&shim_shell_script, &sh_content, options)?;
    Ok(())
}

pub fn create_jar_shim_scripts(
    target_path: &str,
    out_shim_dir: PathBuf,
    alias_name: Option<String>,
    program_args: Option<String>,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let target_name = if alias_name.is_none() {
        Path::new(&target_path)
            .file_stem()
            .and_then(|s| s.to_str().and_then(|s| Some(s.to_lowercase())))
            .ok_or("Invalid target executable name")
    } else {
        let alias_name = alias_name.unwrap();
        Ok(if alias_name.contains('.') {
            eprintln!("alias_name {} 包含 . 字符, 自动去除扩展名", alias_name);
            alias_name.split('.').next().unwrap().to_string()
        } else {
            alias_name
        })
    };
    if target_name.is_err() {
        let target_name = target_name.unwrap();
        bail!("Invalid target executable name {target_path} \n Error TargetName :{target_name}")
    }
    let target_name = target_name.unwrap();
    let out_shim_dir: Result<&str, anyhow::Error> = if out_shim_dir.exists() {
        Ok(out_shim_dir.to_str().unwrap())
    } else {
        bail!("shim 根目录 {} 不存在", out_shim_dir.display());
    };
    let out_shim_dir = out_shim_dir?;
    let shim_cmd_script = format!("{out_shim_dir}\\{target_name}.cmd");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_cmd_script.to_string().dark_green().bold()
    );
    let parent_dir = Path::new(&target_path).parent().unwrap().to_str().unwrap();

    let cmd_content = if program_args.is_none() {
        format!(
            r#"@rem {}
@pushd "{}"
@java -jar "{}"  %*
@popd"#,
            target_path, parent_dir, target_path
        )
    } else {
        let arg = program_args.clone().unwrap();
        format!(
            r#"@rem {}
@pushd "{}"
@java -jar "{}" {} %*
@popd"#,
            target_path, parent_dir, target_path, arg
        )
    };
    write_utf8_file(&shim_cmd_script, &cmd_content, options)?;

    let shim_shell_script = format!("{out_shim_dir}\\{target_name}");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_shell_script.to_string().dark_green().bold()
    );

    // 生成 Unix shell 脚本
    let sh_content = if program_args.is_none() {
        format!(
            r#"#!/bin/sh
# {}
if [ $WSL_INTEROP ]
then
  cd $(wslpath -u '{}')
else
  cd $(cygpath -u '{}')
fi
java.exe -jar "{}"  "$@""#,
            target_path, parent_dir, parent_dir, target_path
        )
    } else {
        let jar_args = program_args.unwrap_or_default();
        format!(
            r#"#!/bin/sh
# {}
if [ $WSL_INTEROP ]
then
  cd $(wslpath -u '{}')
else
  cd $(cygpath -u '{}')
fi
java.exe -jar "{}" {} "$@""#,
            target_path, parent_dir, parent_dir, target_path, jar_args
        )
    };
    write_utf8_file(&shim_shell_script, &sh_content, options)?;

    Ok(())
}

pub fn create_ps1_shim_scripts(
    target_path: &str,
    out_shim_dir: PathBuf,
    alias_name: Option<String>,
    program_params: Option<String>,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let target_name = if alias_name.is_none() {
        Path::new(&target_path)
            .file_stem()
            .and_then(|s| s.to_str().and_then(|s| Some(s.to_lowercase())))
            .ok_or("Invalid target executable name")
    } else {
        let alias_name = alias_name.unwrap();
        Ok(if alias_name.contains('.') {
            eprintln!("alias_name {} 包含 . 字符, 自动去除扩展名", alias_name);
            alias_name.split('.').next().unwrap().to_string()
        } else {
            alias_name
        })
    };
    if target_name.is_err() {
        let target_name = target_name.unwrap();
        bail!("Invalid target executable name {target_path} \n Error TargetName :{target_name}")
    }
    let target_name = target_name.unwrap();
    let resolved_path = target_path;
    let out_shim_dir: Result<&str, anyhow::Error> = if out_shim_dir.exists() {
        Ok(out_shim_dir.to_str().unwrap())
    } else {
        bail!("shim 根目录 {} 不存在", out_shim_dir.display());
    };
    let out_shim_dir = out_shim_dir?;
    let shim_ps1_path = format!("{out_shim_dir}\\{target_name}.ps1");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_ps1_path.to_string().dark_green().bold()
    );

    // 生成 .ps1 脚本
    let ps1_content = if program_params.is_none() {
        format!(
            r#"# {resolved_path}
$path = "{resolved_path}"
if ($MyInvocation.ExpectingInput) {{ $input | & {resolved_path}  @args }} else {{ & {resolved_path} @args }}
exit $LASTEXITCODE
"#
        )
    } else {
        let args = program_params.unwrap();
        format!(
            r#"# {resolved_path}
$path = "{resolved_path}"
if ($MyInvocation.ExpectingInput) {{ $input | & {resolved_path}  {args} @args }} else {{ & {resolved_path} {args} @args }}
exit $LASTEXITCODE
"#
        )
    };
    write_utf8_file(&shim_ps1_path, &ps1_content, options)?;
    // 生成 .cmd 脚本
    let shim_cmd_path = format!("{out_shim_dir}\\{target_name}.cmd");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_cmd_path.to_string().dark_green().bold()
    );

    let cmd_content = format!(
        r#"@rem {resolved_path}
@echo off
where /q pwsh.exe
if %errorlevel% equ 0 (
    pwsh -noprofile -ex unrestricted -file "{resolved_path}" %*
) else (
    powershell -noprofile -ex unrestricted -file "{resolved_path}" %*
)
"#,
        resolved_path = resolved_path
    );
    write_utf8_file(&shim_cmd_path, &cmd_content, options)?;

    // 生成 .sh 脚本
    let shim_shell_path = format!("{out_shim_dir}\\{target_name}");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_shell_path.to_string().dark_green().bold()
    );
    let sh_content = format!(
        r#"#!/bin/sh
# {resolved_path}
if command -v pwsh.exe > /dev/null 2>&1; then
    pwsh.exe -noprofile -ex unrestricted -file "{resolved_path}" "$@"
else
    powershell.exe -noprofile -ex unrestricted -file "{resolved_path}" "$@"
fi
"#,
        resolved_path = resolved_path
    );
    write_utf8_file(&shim_shell_path, &sh_content, options)?;
    Ok(())
}

pub fn create_cmd_or_bat_shim_scripts(
    target_path: &str,
    out_shim_dir: PathBuf,
    alias_name: Option<String>,
    program_args: Option<String>,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let target_name = if alias_name.is_none() {
        Path::new(&target_path)
            .file_stem()
            .and_then(|s| s.to_str().and_then(|s| Some(s.to_lowercase())))
            .ok_or("Invalid target executable name")
    } else {
        let alias_name = alias_name.unwrap();
        Ok(if alias_name.contains('.') {
            eprintln!("alias_name {} 包含 . 字符, 自动去除扩展名", alias_name);
            alias_name.split('.').next().unwrap().to_string()
        } else {
            alias_name
        })
    };
    if target_name.is_err() {
        let target_name = target_name.unwrap();
        bail!("Invalid target executable name {target_path} \n Error TargetName :{target_name}")
    }
    let target_name = target_name.unwrap();

    let cmd_content = if program_args.is_none() {
        format!("@rem {target_path}\r\n@\"{target_path}\" %*\r\n")
    } else {
        let args = program_args.clone().unwrap();
        format!("@rem {target_path}\r\n@\"{target_path}\" {args} %*\r\n")
    };
    let out_shim_dir: Result<&str, anyhow::Error> = if out_shim_dir.exists() {
        Ok(out_shim_dir.to_str().unwrap())
    } else {
        bail!("shim 根目录 {} 不存在", out_shim_dir.display());
    };
    let out_shim_dir = out_shim_dir?;
    let shim_cmd_path = format!("{out_shim_dir}\\{target_name}.cmd");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_cmd_path.to_string().dark_green().bold()
    );

    let crlf_content = cmd_content.replace(LineEnding::LF.as_str(), LineEnding::CRLF.as_str());

    write_utf8_file(&shim_cmd_path, &crlf_content, &options)?;

    let shim_shell_path = format!("{out_shim_dir}\\{target_name}");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_shell_path.to_string().dark_green().bold()
    );
    let sh_content = if program_args.is_none() {
        format!(
            "#!/bin/sh\n# {}\nMSYS2_ARG_CONV_EXCL=/C cmd.exe /C \"{}\"  \"$@\"\n",
            target_path, target_path
        )
    } else {
        let args = program_args.unwrap();
        format!(
            "#!/bin/sh\n# {}\nMSYS2_ARG_CONV_EXCL=/C cmd.exe /C \"{}\" {} \"$@\"\n",
            target_path, target_path, args
        )
    };

    let crlf_content = sh_content.replace(LineEnding::LF.as_str(), LineEnding::CRLF.as_str());

    write_utf8_file(&shim_shell_path, &crlf_content, &options)?;
    Ok(())
}

pub fn create_shell_shim_scripts(
    target_path: &str,
    out_shim_dir: PathBuf,
    alias_name: Option<String>,
    program_args: Option<String>,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let target_name = if alias_name.is_none() {
        Path::new(&target_path)
            .file_stem()
            .and_then(|s| s.to_str().and_then(|s| Some(s.to_lowercase())))
            .ok_or("Invalid target executable name")
    } else {
        let alias_name = alias_name.unwrap();
        Ok(if alias_name.contains('.') {
            eprintln!("alias_name {} 包含 . 字符, 自动去除扩展名", alias_name);
            alias_name.split('.').next().unwrap().to_string()
        } else {
            alias_name
        })
    };
    if target_name.is_err() {
        let target_name = target_name.unwrap();
        bail!("Invalid target executable name {target_path} \n Error TargetName :{target_name}")
    }
    if !out_shim_dir.exists() {
        fs::create_dir_all(&out_shim_dir)?;
    }
    let out_shim_dir = out_shim_dir.to_str().unwrap();
    let target_name = target_name.unwrap();

    let shim_cmd_path = format!("{out_shim_dir}\\{target_name}.cmd");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_cmd_path.to_string().dark_green().bold()
    );
    let shell_args = program_args.unwrap_or(String::new());
    let cmd_lines = vec![
        format!("@rem {}", target_path),
        format!(
            r#"@bash "$(wslpath -u '{}')" {} %* 2>nul"#,
            target_path, shell_args
        ),
        "@if %errorlevel% neq 0 (".to_string(),
        format!(
            r#"  @bash "$(cygpath -u '{}')" {} %* 2>nul"#,
            target_path, shell_args
        ),
        ")".to_string(),
    ];
    let cmd_content = cmd_lines.join("\r\n");

    write_utf8_file(shim_cmd_path.as_str(), &cmd_content, &options)?;

    let shim_shell_path = format!("{out_shim_dir}\\{target_name}");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_shell_path.to_string().dark_green().bold()
    );
    let sh_lines = vec![
        "#!/bin/sh".to_string(),
        format!("# {}", target_path),
        "if [ $WSL_INTEROP ]".to_string(),
        "then".to_string(),
        format!(r#"  "$(wslpath -u '{}')" {} "$@""#, target_path, shell_args),
        "else".to_string(),
        format!(r#"  "$(cygpath -u '{}')" {} "$@""#, target_path, shell_args),
        "fi".to_string(),
    ];
    let sh_content = sh_lines.join("\n");
    write_utf8_file(&shim_shell_path, &sh_content, &options)?;
    Ok(())
}

pub fn create_exe_type_shim_file_and_shim_bin<P1: AsRef<Path>, P2: AsRef<Path>>(
    target_path: P1,
    output_dir: P2,
    alias_name: Option<String>,
    program_params: Option<String>,
    options: &[InstallOptions],
) -> anyhow::Result<()> {
    let target_path = target_path.as_ref().to_str().unwrap();
    let output_dir = output_dir.as_ref().to_path_buf();

    let target_name = if alias_name.is_none() {
        Path::new(target_path)
            .file_stem()
            .and_then(|s| s.to_str().and_then(|s| Some(s.to_lowercase())))
            .ok_or("Invalid target executable name")
    } else {
        let alias_name = alias_name.unwrap();
        Ok(if alias_name.contains('.') {
            eprintln!("alias_name {} 包含 . 字符, 自动去除扩展名", alias_name);
            alias_name.split('.').next().unwrap().to_string()
        } else {
            alias_name
        })
    };

    if target_name.is_err() {
        let target_name = target_name.unwrap();
        bail!("Invalid target executable name {target_path} \n Error TargetName :{target_name}")
    }

    let content = if program_params.is_none() {
        format!("path = \"{}\"", target_path)
    } else {
        let program_params = program_params.clone().unwrap();
        format!("path = \"{target_path}\"\nargs = \"{program_params}\"")
    };
    let target_name = target_name.unwrap();
    // Determine the shim file name
    let shim_name = format!("{}.shim", target_name);
    let shim_name2 = format!("{}.exe", target_name);
    let shim_path = output_dir.join(&shim_name);
    let shim_path2 = output_dir.join(&shim_name2);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).context("failed create output_dir at line 880")?;
    }

    let crlf_content = if program_params.is_some() {
        content.clone()
    } else {
        content.replace(LineEnding::LF.as_str(), LineEnding::CRLF.as_str())
    };
    write_utf8_file(
        shim_path.as_path().to_str().unwrap(),
        crlf_content.as_str(),
        options,
    )?;
    println!(
        "{} {}",
        "Creating  shim  file => ".to_string().dark_blue().bold(),
        &shim_path.to_str().unwrap().dark_green().bold()
    );
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_path2.display().to_string().dark_green().bold()
    );

    if DRIVER_SHIM_BYTES.is_empty() {
        bail!("origin driver shim.exe not found");
    }
    #[cfg(windows)]
    {
        if shim_path2.exists() {
            return Ok(());
        }
        fs::write(&shim_path2, DRIVER_SHIM_BYTES)
            .expect("failed create shim.exe, maybe process is running");
    }
    Ok(())
}

#[cfg(test)]
mod test_shim {
    #[allow(unused)]
    use super::*;
    #[allow(unused)]
    use std::env;
    #[test]
    #[ignore]
    fn test_create_shortcuts() {
        use crate::install::create_start_menu_shortcuts;
        use crate::manifest::install_manifest::InstallManifest;
        let file = r"A:\Scoop\buckets\ScoopMaster\bucket\zigmod.json";
        let content = fs::read_to_string(file).unwrap();
        let manifest: InstallManifest = serde_json::from_str(&content).unwrap();
        let shortcuts = manifest.shortcuts.unwrap();
        let app_name = "zigmod".to_string();
        let options = vec![];
        create_start_menu_shortcuts(shortcuts, app_name, &options).unwrap();
    }

    #[test]
    fn test_create_exe_shims() {
        let cwd = env::current_dir().unwrap();
        let output_dir = cwd.join("src\\bin\\output");
        println!("{}", cwd.display());
        let shim_exe = cwd.join("src\\bin\\shim.exe");
        if shim_exe.exists() {
            println!("{:?}", shim_exe);
        }
        let target_path = r#"A:\Scoop\apps\zig\current\zig.exe"#;
        let args = "run -h";
        let options = vec![];
        create_exe_type_shim_file_and_shim_bin(
            target_path,
            output_dir,
            None,
            Some(args.into()),
            &options,
        )
        .unwrap();
    }

    #[test]
    fn test_create_bat_and_cmd_shim() {
        let cwd = env::current_dir().unwrap();
        let output_dir = cwd.join("src\\bin\\output");
        let app_name = "sbt".to_string();
        let exe_name = r"bin\\sbt.bat".to_string();
        let options = vec![InstallOptions::Global];
        let target_path =
            get_app_current_bin_path(app_name.as_str(), &exe_name, options.as_slice());
        if Path::new(&target_path).exists() {
            println!("target {target_path}");
        }
        let options = vec![];
        let _ = create_cmd_or_bat_shim_scripts(
            target_path.as_str(),
            output_dir,
            Some("sbtsbt".into()),
            None,
            &options,
        );
    }

    #[test]
    fn test_create_py_shim() {}

    #[test]
    fn test_create_ps_shim() {
        let cwd = env::current_dir().unwrap();
        let output_dir = cwd.join("src\\bin\\output");
        let app_name = "composer".to_string();
        let exe_name = r"composer.ps1".to_string();
        let options = vec![InstallOptions::Global];
        let target_path =
            get_app_current_bin_path(app_name.as_str(), &exe_name, options.as_slice());
        if Path::new(&target_path).exists() {
            println!("target {target_path}");
        }
        let options = vec![];
        let _ = create_ps1_shim_scripts(
            target_path.as_ref(),
            output_dir,
            Some("composer".into()),
            None,
            &options,
        );
    }
    #[test]
    fn find_cmd_bat_ps_scripts_alias() {
        use crate::buckets::get_buckets_path;
        use rayon::prelude::*;

        let bucket = get_buckets_path().unwrap();
        let buckets = bucket
            .iter()
            .par_bridge()
            .map(|path| Path::new(path).join("bucket"))
            .collect::<Vec<_>>();

        let files = buckets
            .iter()
            .flat_map(|path| path.read_dir().unwrap().map(|res| res.unwrap().path()))
            .collect::<Vec<_>>();
        for path in files {
            let content = fs::read_to_string(&path);
            if content.is_err() {
                println!("decode   error {:?}", path.display());
                continue;
            }
            let content = content.unwrap();
            let manifest = serde_json::from_str::<InstallManifest>(&content);
            if manifest.is_err() {
                println!("decode manifest error {:?}", path.display());
                eprintln!("Error : {:?}", manifest.unwrap_err());
                return;
            }
            let manifest = manifest.unwrap();
            let bin = manifest.bin;
            if bin.is_some() {
                let bin = bin.unwrap();
                match bin {
                    StringOrArrayOrDoubleDimensionArray::Null => {}
                    StringOrArrayOrDoubleDimensionArray::String(_) => {}
                    StringOrArrayOrDoubleDimensionArray::StringArray(_) => {}
                    StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(arrs) => {
                        for arr in arrs {
                            if arr.len() == 2 {
                                let exe_name = arr[0].clone();
                                let suffix = exe_name.split('.').last().unwrap();
                                let _alias_name = arr[1].clone();
                                if suffix == "cmd" || suffix == "bat" {
                                    println!("{:?}", arr);
                                    println!("script path {:?}", path.display());
                                }
                            }
                            if arr.len() == 3 {
                                let exe_name = arr[0].clone();
                                let suffix = exe_name.split('.').last().unwrap();
                                let _alias_name = arr[1].clone();
                                if suffix == "cmd" || suffix == "bat" {
                                    println!("{:?}", arr);
                                    println!("script path has  args {:?}", path.display());
                                    return;
                                }
                            }
                        }
                    }
                    StringOrArrayOrDoubleDimensionArray::NestedStringArray(nested_arr) => {
                        for arr in nested_arr {
                            match arr {
                                StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(arrs) => {
                                    for arr in arrs {
                                        if arr.len() == 2 {
                                            let exe_name = arr[0].clone();
                                            let suffix = exe_name.split('.').last().unwrap();
                                            let _alias_name = arr[1].clone();
                                            if suffix == "cmd" || suffix == "bat" {
                                                println!("{:?}", arr);
                                                println!("script path {:?}", path.display());
                                            }
                                        }
                                        if arr.len() == 3 {
                                            let exe_name = arr[0].clone();
                                            let suffix = exe_name.split('.').last().unwrap();
                                            let _alias_name = arr[1].clone();
                                            if suffix == "cmd" || suffix == "bat" {
                                                println!("{:?}", arr);
                                                println!(
                                                    "script path has  args {:?}",
                                                    path.display()
                                                );
                                                return;
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
