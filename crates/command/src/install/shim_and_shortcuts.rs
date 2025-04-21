use crate::init_env::{get_app_current_bin_path, get_shims_path, get_shims_path_global};
use crate::install::InstallOptions;
use crate::manifest::install_manifest::InstallManifest;
use crate::manifest::manifest_deserialize::{
    ArrayOrDoubleDimensionArray, StringOrArrayOrDoubleDimensionArray,
};
use crate::utils::system::get_system_default_arch;
use crate::utils::utility::write_utf8_file;
use anyhow::bail;
use crossterm::style::Stylize;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

const DRIVER_SHIM_BYTES: &[u8] = include_bytes!("..\\bin\\shim.exe");

pub fn create_shim_or_shortcuts(
    manifest_json: &str,
    app_name: &str,
    options: &Box<[InstallOptions]>,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(manifest_json)?;
    let serde_obj: InstallManifest = serde_json::from_str(&content)?;
    let bin = serde_obj.bin;
    let architecture = serde_obj.architecture;
    let shortcuts = serde_obj.shortcuts;
    if bin.is_some() {
        create_shims_file(bin.unwrap(), app_name, options)?;
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
            create_shims_file(bin, app_name, options)?;
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
            create_shims_file(bin, app_name, options)?;
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
            create_shims_file(bin, app_name, options)?;
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
    app_name: &str,
    options: &Box<[InstallOptions]>,
) -> anyhow::Result<()> {
    let shim_path = if options.contains(&InstallOptions::Global) {
        get_shims_path_global()
    } else {
        get_shims_path()
    };
    if !Path::new(&shim_path).exists() {
        fs::create_dir_all(&shim_path)?;
    }
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
                    create_alias_shim_name_file(exe_name, alias_name, &shim_path, app_name, None)?;
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
                    let target_path = get_app_current_bin_path(app_name, &bin_name_with_extension);
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
                        let target_path =
                            get_app_current_bin_path(app_name.clone(), &bin_name_with_extension);
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
        "Creating  Shortcuts for".to_string().dark_blue().bold(),
        app_name.to_string().dark_cyan().bold(),
        link.to_string().dark_green().bold()
    );
    let shell_link = ShellLink::new(link_target_path)?;
    shell_link.create_lnk(start_menu_path)?;
    Ok(())
}

///   *-------------------------------------------*
pub fn create_alias_shim_name_file(
    exe_name: String,
    alias_name: String,
    shim_dir: &str,
    app_name: &str,
    program_args: Option<String>,
) -> anyhow::Result<()> {
    let out_dir = PathBuf::from(shim_dir);
    let temp = exe_name.clone();
    let suffix = temp.split('.').last().unwrap();
    // log::debug!("Origin file type {}", suffix);

    let target_path = get_app_current_bin_path(app_name.into(), &exe_name);
    if !out_dir.exists() {
        bail!(format!("shim 目录 {shim_dir} 不存在"));
    }
    if !Path::new(&target_path).exists() {
        bail!(format!("链接目标文件 {target_path} 不存在"))
    };
    if suffix == "exe" || suffix == "com" {
        create_exe_type_shim_file_and_shim_bin(
            target_path,
            out_dir,
            Some(alias_name),
            program_args,
        )?;
    } else if suffix == "cmd" || "bat" == suffix {
        let result = exclude_scoop_self_scripts(&exe_name, None)?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{exe_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        create_cmd_or_bat_shim_scripts(target_path, out_dir, Some(alias_name), program_args)?;
    } else if suffix == "ps1" {
        let result = exclude_scoop_self_scripts(&exe_name, None)?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{exe_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        create_ps1_shim_scripts(target_path, out_dir, Some(alias_name), program_args)?;
    } else if suffix == "jar" {
        create_jar_shim_scripts(target_path, out_dir, Some(alias_name), program_args)?;
    } else if suffix == "py" {
        create_py_shim_scripts(target_path, out_dir, Some(alias_name), program_args)?;
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
) -> anyhow::Result<()> {
    let out_dir = PathBuf::from(shim_dir);
    let temp = exe_name.clone();
    let suffix = temp.split('.').last().unwrap();
    // log::debug!("Origin file type {}", suffix);
    if suffix.is_empty() {
        bail!(format!("shim 文件名 {exe_name} 后缀为空 WTF?"))
    }
    let target_path = get_app_current_bin_path(app_name.into(), &exe_name);
    if !out_dir.exists() {
        bail!(format!("shim 目录 {shim_dir} 不存在"));
    }
    if !Path::new(&target_path).exists() {
        bail!(format!("链接目标文件 {target_path} 不存在"))
    };
    if suffix == "exe" || suffix == "com" {
        create_exe_type_shim_file_and_shim_bin(target_path, out_dir, None, None)?;
    } else if suffix == "cmd" || "bat" == suffix {
        let result = exclude_scoop_self_scripts(&exe_name, None)?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{exe_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        create_cmd_or_bat_shim_scripts(target_path, out_dir, None, None)?;
    } else if suffix == "ps1" {
        let result = exclude_scoop_self_scripts(&exe_name, None)?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{exe_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        create_ps1_shim_scripts(target_path, out_dir, None, None)?;
    } else if suffix == "jar" {
        create_jar_shim_scripts(target_path, out_dir, None, None)?;
    } else if suffix == "py" {
        create_py_shim_scripts(target_path, out_dir, None, None)?;
    } else {
        bail!(format!(" 后缀{suffix}类型文件不支持, WTF?"))
    }
    Ok(())
}

fn create_py_shim_scripts(
    target_path: String,
    out_shim_dir: PathBuf,
    alias_name: Option<String>,
    program_args: Option<String>,
) -> anyhow::Result<()> {
    let target_name = if alias_name.is_none() {
        Path::new(&target_path)
            .file_stem()
            .and_then(|s| s.to_str().and_then(|s| Some(s.to_lowercase())))
            .ok_or("Invalid target executable name")
    } else {
        Ok(alias_name.unwrap().to_string())
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
    write_utf8_file(&shim_cmd_script, &cmd_content)?;

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
    write_utf8_file(&shim_shell_script, &sh_content)?;
    Ok(())
}

fn create_jar_shim_scripts(
    target_path: String,
    out_shim_dir: PathBuf,
    alias_name: Option<String>,
    program_args: Option<String>,
) -> anyhow::Result<()> {
    let target_name = if alias_name.is_none() {
        Path::new(&target_path)
            .file_stem()
            .and_then(|s| s.to_str().and_then(|s| Some(s.to_lowercase())))
            .ok_or("Invalid target executable name")
    } else {
        Ok(alias_name.unwrap().to_string())
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
    write_utf8_file(&shim_cmd_script, &cmd_content)?;

    let shim_shell_script = format!("{out_shim_dir}\\{target_name}");
    println!(
        "{} {}",
        "Creating  shim  proxy launcher =>".dark_blue().bold(),
        &shim_shell_script.to_string().dark_green().bold()
    );

    // 生成 Unix shell 脚本
    let sh_content = if program_args.is_some() {
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
        let arg = program_args.unwrap();
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
            target_path, parent_dir, parent_dir, target_path, arg
        )
    };
    write_utf8_file(&shim_shell_script, &sh_content)?;

    Ok(())
}

fn create_ps1_shim_scripts(
    target_path: String,
    out_shim_dir: PathBuf,
    alias_name: Option<String>,
    program_params: Option<String>,
) -> anyhow::Result<()> {
    let target_name = if alias_name.is_none() {
        Path::new(&target_path)
            .file_stem()
            .and_then(|s| s.to_str().and_then(|s| Some(s.to_lowercase())))
            .ok_or("Invalid target executable name")
    } else {
        Ok(alias_name.unwrap().to_string())
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
    write_utf8_file(&shim_ps1_path, &ps1_content)?;
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
    write_utf8_file(&shim_cmd_path, &cmd_content)?;

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
    write_utf8_file(&shim_shell_path, &sh_content)?;
    Ok(())
}

pub fn exclude_scoop_self_scripts(
    script_name: &String,
    alias_name: Option<String>,
) -> anyhow::Result<u8> {
    let split = script_name.split(".").collect::<Vec<&str>>();
    if split.len() != 2 {
        bail!("shim target {script_name} 文件名格式错误, WTF?")
    }
    if alias_name.is_some() {
        let script_name = alias_name.unwrap();
        let exclude_list = vec!["scoop", "scoop-pre", "scoop-premake", "scoop-rm_nm"];
        if exclude_list.contains(&script_name.as_str()) {
            return Ok(1);
        }
        return Ok(0);
    }
    let script_name = split.get(0).unwrap();
    let exclude_list = vec!["scoop", "scoop-pre", "scoop-premake", "scoop-rm_nm"];
    if exclude_list.contains(&script_name) {
        return Ok(1);
    }
    Ok(0)
}

pub fn create_cmd_or_bat_shim_scripts(
    target_path: String,
    out_shim_dir: PathBuf,
    alias_name: Option<String>,
    program_args: Option<String>,
) -> anyhow::Result<()> {
    let target_name = if alias_name.is_none() {
        Path::new(&target_path)
            .file_stem()
            .and_then(|s| s.to_str().and_then(|s| Some(s.to_lowercase())))
            .ok_or("Invalid target executable name")
    } else {
        Ok(alias_name.unwrap().to_string())
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

    let mut cmd_file = File::create(shim_cmd_path)?;
    cmd_file.write_all(cmd_content.as_bytes())?;

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

    let mut sh_file = File::create(shim_shell_path)?;
    sh_file.write_all(sh_content.as_bytes())?;

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
        let target_name = target_name.unwrap();
        bail!("Invalid target executable name {target_path} \n Error TargetName :{target_name}")
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
        "Creating  shim  file => ".to_string().dark_blue().bold(),
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
            "Creating  shim  proxy launcher =>".dark_blue().bold(),
            &output_shim_exe.display().to_string().dark_green().bold()
        );
        fs::write(&output_shim_exe, DRIVER_SHIM_BYTES)?;
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
        create_start_menu_shortcuts(shortcuts, app_name).unwrap();
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
        create_exe_type_shim_file_and_shim_bin(target_path, output_dir, None, Some(args.into()))
            .unwrap();
    }

    #[test]
    fn test_create_bat_and_cmd_shim() {
        let cwd = env::current_dir().unwrap();
        let output_dir = cwd.join("src\\bin\\output");
        let app_name = "sbt".to_string();
        let exe_name = r"bin\\sbt.bat".to_string();
        let target_path = get_app_current_bin_path(app_name.into(), &exe_name);
        if Path::new(&target_path).exists() {
            println!("target {target_path}");
        }
        let _ =
            create_cmd_or_bat_shim_scripts(target_path, output_dir, Some("sbtsbt".into()), None);
    }

    #[test]
    fn test_create_py_shim() {}

    #[test]
    fn test_create_ps_shim() {
        let cwd = env::current_dir().unwrap();
        let output_dir = cwd.join("src\\bin\\output");
        let app_name = "composer".to_string();
        let exe_name = r"composer.ps1".to_string();
        let target_path = get_app_current_bin_path(app_name.into(), &exe_name);
        if Path::new(&target_path).exists() {
            println!("target {target_path}");
        }
        let _ = create_ps1_shim_scripts(target_path, output_dir, Some("composer".into()), None);
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
