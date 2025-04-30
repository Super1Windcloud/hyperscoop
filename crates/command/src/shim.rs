use crate::init_env::{get_shims_root_dir, get_shims_root_dir_global};
use crate::init_hyperscoop;
use crate::install::{
    create_cmd_or_bat_shim_scripts, create_exe_type_shim_file_and_shim_bin,
    create_jar_shim_scripts, create_ps1_shim_scripts, create_py_shim_scripts,
    exclude_scoop_self_scripts,
};
use anyhow::bail;
use crossterm::style::Stylize;
use std::path::{Path, PathBuf};

pub fn list_all_shims(global: bool) -> Result<(), anyhow::Error> {
    let shim_path = if global {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    let shim_path = Path::new(&shim_path);
    if !shim_path.exists() {
        bail!("{} is not exist", shim_path.display());
    }
    let mut shims = vec![];
    for entry in shim_path.read_dir()? {
        let entry = entry?;
        let file_name = entry.file_type()?;
        let path = entry.path();
        if !file_name.is_file() {
            continue;
        }
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if !file_name.ends_with(".exe") && !file_name.ends_with(".cmd") {
            continue;
        }
        let (shim_name, suffix) = if file_name.ends_with(".exe") {
            (file_name.replace(".exe", ""), "exe")
        } else {
            (file_name.replace(".cmd", ""), "cmd")
        };

        let shim_file_path = if suffix == "exe" {
            path.to_str().unwrap().to_owned().replace(".exe", ".shim")
        } else {
            path.to_str().unwrap().to_owned()
        };

        let shim_source = std::fs::read_to_string(&shim_file_path)?.replace("path =", "");
        shims.push((shim_name, path.to_str().unwrap().to_owned(), shim_source));
    }

    for (name, path, source) in shims {
        println!(
            "Names: {:<15}  Path: {:<30} \nSource: {:<30}",
            name, path, source
        );
    }

    Ok(())
}

pub fn list_shims_by_regex(regex: String, global: bool) {
    log::info!("list_shims_by_regex {}", regex);
    let shim_path = if global {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    let shim_path = Path::new(&shim_path);
    let mut shims = vec![];
    for entry in shim_path.read_dir().unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        let path = entry.path();
        if !file_type.is_file() {
            continue;
        }
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if !file_name.ends_with(".exe") && !file_name.ends_with(".cmd") {
            continue;
        }

        let (file_name, suffix) = if file_name.ends_with(".exe") {
            (file_name.replace(".exe", ""), "exe")
        } else {
            (file_name.replace(".cmd", ""), "cmd")
        };
        let re = regex::Regex::new(&regex).unwrap();
        let cap = re.captures(&file_name);
        if cap.is_none() {
            continue;
        }
        let shim_file_path = if suffix == "exe" {
            path.to_str().unwrap().to_owned().replace(".exe", ".shim")
        } else {
            path.to_str().unwrap().to_owned()
        };
        let shim_source = std::fs::read_to_string(&shim_file_path)
            .unwrap()
            .replace("path =", "");
        shims.push((file_name, path.to_str().unwrap().to_owned(), shim_source));
    }

    for (name, path, source) in shims {
        println!(
            "Names: {:<15}  Path: {:<30} \nSource: {:<30}",
            name, path, source
        );
    }
}

pub fn list_shim_info(name: Option<String>, global: bool) -> anyhow::Result<()> {
    if name.is_none() {
        return Ok(());
    }
    let shim_name = name.unwrap();
    let shim_path = if global {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    let shim_path = Path::new(&shim_path);
    if !shim_path.exists() {
        bail!("{} is not exist", shim_path.display());
    }
    for entry in shim_path.read_dir()? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if !file_type.is_file() {
            continue;
        }
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let pure_name = path.file_stem().unwrap().to_str().unwrap();
        if pure_name != shim_name
            || (!file_name.ends_with(".exe") && !file_name.ends_with(".cmd"))
            || file_name.ends_with(".shim")
        {
            continue;
        }
        let shim_path = path.to_str().unwrap().to_owned();
        println!(
            "{:<10} : {}",
            "Name ",
            shim_name.clone().dark_green().bold()
        );
        println!(
            "{:<10} : {}",
            "Path ",
            shim_path.clone().dark_green().bold()
        );
        println!(
            "{:<10} : {}",
            "Source ",
            shim_name.clone().dark_green().bold()
        );
        println!("{:<10} : {}", "Type ", "Application".dark_green().bold());
        println!("{:<10} : {}", "IsGlobal ", "False".dark_green().bold());
        println!("{:<10} : {}", "IsHidden ", "False".dark_green().bold());
    }
    println!(
        "{}{}",
        "No shim found for name: ".red().bold(),
        shim_name.dark_cyan().bold()
    );

    Ok(())
}

pub fn execute_add_shim(
    shim_name: Option<String>,
    command_path: Option<String>,
    args: Option<String>,
    global: bool,
) -> anyhow::Result<()> {
    if command_path.is_none() {
        bail!("Command path is must required");
    }
    let shim_name = shim_name.unwrap();
    let target_path = command_path.unwrap();
    let shim_path = if global {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    if args.is_none() {
        create_shims(shim_name.as_str(), &shim_path, &target_path, "")?;
    } else {
        create_shims(shim_name.as_str(), &shim_path, &target_path, args.unwrap().as_str())?;
    } 
    println!("{}", format!("Shim '{}' has been created successfully!", shim_name).dark_green().bold().to_owned());
    Ok(())
}

pub fn create_shims<'a>(
    shim_name: &str ,
    shim_dir: &str,
    target_path: &str,
    args: impl Into<Option<&'a str>>,
) -> anyhow::Result<()> {
    let out_dir = PathBuf::from(shim_dir);
    let args = args.into().unwrap_or_default();

    let suffix = target_path.split('.').last().unwrap();
    // log::debug!("Origin file type {}", suffix);
    if suffix.is_empty() {
        bail!(format!("shim 文件名 {shim_name} 后缀为空 WTF?"))
    }
    if !out_dir.exists() {
        bail!(format!("shim 目录 {shim_dir} 不存在"));
    }
    if shim_name == "hp" {
        bail!("hp 不能作为 shim 名称")
    }
    if !Path::new(target_path).exists() {
        bail!(format!("链接目标文件 {target_path} 不存在"))
    };
    if suffix == "exe" || suffix == "com" {
        if !args.is_empty() {
            create_exe_type_shim_file_and_shim_bin(
                target_path,
                out_dir,
                Some(shim_name.into()),
                Some(args.to_owned()),
            )?;
        } else {
            create_exe_type_shim_file_and_shim_bin(target_path, out_dir, Some(shim_name.into()), None)?;
        }
    } else if suffix == "cmd" || "bat" == suffix {
        let result = exclude_scoop_self_scripts(target_path, Some(shim_name))?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{shim_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        if !args.is_empty() {
            create_cmd_or_bat_shim_scripts(
                target_path,
                out_dir,
                Some(shim_name.into()),
                Some(args.to_owned()),
            )?;
        } else {
            create_cmd_or_bat_shim_scripts(target_path, out_dir, Some(shim_name.into()), None)?;
        }
    } else if suffix == "ps1" {
        let result = exclude_scoop_self_scripts(target_path, Some(shim_name))?;
        if result != 0 {
            bail!("Origin 二进制名或者该二进制别名 '{shim_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖")
        }
        if !args.is_empty() {
            create_ps1_shim_scripts(target_path, out_dir, Some(shim_name.into()), Some(args.to_owned()))?
        } else {
            create_ps1_shim_scripts(target_path, out_dir, Some(shim_name .into()), None)?
        };
    } else if suffix == "jar" {
        if !args.is_empty() {
            create_jar_shim_scripts(target_path, out_dir, Some(shim_name.into()), Some(args.to_owned()))?
        } else {
            create_jar_shim_scripts(target_path, out_dir, Some(shim_name.into()), None)?;
        }
    } else if suffix == "py" {
        if !args.is_empty() {
            create_py_shim_scripts(target_path, out_dir, Some(shim_name.into()), Some(args.to_owned()))?
        } else {
            create_py_shim_scripts(target_path, out_dir, Some(shim_name.into()), None)?;
        }
    } else {
        bail!(format!(" 后缀{suffix}类型文件不支持, WTF?"))
    }
    Ok(())
}

pub fn alter_shim_source(name: String, source: String) {
    let shim_name = name.clone();
    let shim_source = source.clone();
    let shim_path = init_hyperscoop().unwrap().get_shims_root_dir();
    let shim_file = shim_path.clone() + "\\" + &shim_name + ".shim";
    let shim_content = format!("path = {}\n", shim_source);
    let shim_file = Path::new(&shim_file);
    log::info!("Altering shim: {} ", shim_file.display());
    std::fs::write(shim_file, shim_content).unwrap();
}

pub fn remove_shim(name: Option<String>, global: bool) -> anyhow::Result<()> {
    if name.is_none() {
        return Ok(());
    }
    let shim_name = name.unwrap();
    let shim_path = if global {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    let shim_path = Path::new(&shim_path);
    if !shim_path.exists() {
        bail!("{} is not exist", shim_path.display());
    }
    let shim_dir = std::fs::read_dir(&shim_path)?;
    let matched_shims = shim_dir
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_type = entry.file_type().ok()?;
            let path = entry.path();
            if !file_type.is_file() {
                return None;
            }
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            if file_name == shim_name {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    matched_shims.iter().for_each(|shim_path| {
        if !shim_path.exists() {
            eprintln!("{} is not exist", shim_path.display());
            return;
        }
        println!("Removing shim: {} ", shim_path.display());
        std::fs::remove_file(shim_path).unwrap();
    });
    Ok(())
}
