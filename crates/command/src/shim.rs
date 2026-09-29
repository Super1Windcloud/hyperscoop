use crate::i18n::tr;
use crate::info::validate_app_name;
use crate::init_env::{get_shims_root_dir, get_shims_root_dir_global};
use crate::init_hyperscoop;
use crate::install::{
    create_cmd_or_bat_shim_scripts, create_exe_type_shim_file_and_shim_bin,
    create_jar_shim_scripts, create_ps1_shim_scripts, create_py_shim_scripts,
};
use crate::tr_fmt;
use crate::utils::utility::{exclude_scoop_self_scripts, extract_target_path_from_shell_script};
use anyhow::{Context, bail};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use crossterm::style::Stylize;
use std::path::{Path, PathBuf};

pub fn list_all_shims(global: bool) -> anyhow::Result<Vec<(String, String, String)>> {
    let shim_path = if global {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    let shim_path = Path::new(&shim_path);
    if !shim_path.exists() {
        bail!(tr_fmt!(
            "Shim directory '{path}' does not exist",
            "shim 目录 '{path}' 不存在",
            path = shim_path.display()
        ));
    }
    let mut shims = vec![];
    for entry in shim_path
        .read_dir()
        .context("Failed to read  shim root dir at line 28")?
    {
        let entry = entry.context("Failed to read  shim root dir at line 30")?;
        let file_name = entry.file_type()?;
        let path = entry.path();
        if !file_name.is_file() {
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

        let shim_file_path = if suffix == "exe" {
            path.to_str().unwrap().replace(".exe", ".shim")
        } else {
            path.to_str().unwrap().to_owned()
        };

        let shim_source = if suffix == "cmd" {
            let content = extract_rem_comments(path.to_str().unwrap());
            content.trim().to_owned()
        } else {
            let content = std::fs::read_to_string(&shim_file_path)
                .context("Failed to read shim file content at line 57")?;
            let first_line = content.lines().next().unwrap().trim();
            first_line
                .replace("path =", "")
                .replace("\"", "")
                .trim()
                .to_owned()
        };
        let ps1_script = path.to_str().unwrap().replace(".cmd", ".ps1");
        let ps1_script = Path::new(&ps1_script);
        if ps1_script != path.as_path() && ps1_script.exists() {
            shims.push((
                file_name,
                ps1_script.to_str().unwrap().to_owned(),
                shim_source,
            ));
        } else {
            shims.push((file_name, path.to_str().unwrap().to_owned(), shim_source));
        }
    }

    let count = shims.len();
    println!(
        "{}{}",
        tr("\tFound shims count: ", "\t找到 shim 数量: ")
            .dark_green()
            .bold(),
        count.to_string().dark_green().bold()
    );

    let shims_vec = shims
        .iter()
        .map(|(name, path, source)| vec![name, path, source])
        .collect::<Vec<_>>();

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new(tr("ShimName", "Shim名称"))
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
            Cell::new(tr("Path", "路径"))
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
            Cell::new(tr("Source", "来源"))
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
        ])
        .add_rows(shims_vec.as_slice());

    log::debug!(
        "{:?}",
        shims
            .get(0)
            .unwrap_or(&(String::new(), String::new(), String::new()))
    );
    println!("{table}");

    Ok(shims)
}

pub fn list_shims_by_regex(regex: String, global: bool) -> anyhow::Result<()> {
    log::info!("list_shims_by_regex {}", regex);
    let shim_path = if global {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    let shim_path = Path::new(&shim_path);
    let mut shims = vec![];
    for entry in shim_path
        .read_dir()
        .context("Failed to read  shim root dir at line 75")?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
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
        let re = regex::Regex::new(&regex)?;
        let cap = re.captures(&file_name);
        if cap.is_none() {
            continue;
        }
        let shim_file_path = if suffix == "exe" {
            path.to_str().unwrap().replace(".exe", ".shim")
        } else {
            path.to_str().unwrap().to_owned()
        };

        let shim_source = if suffix == "cmd" {
            let content = extract_rem_comments(path.to_str().unwrap());
            content.trim().to_owned()
        } else {
            let content = std::fs::read_to_string(&shim_file_path)
                .context("Failed to read shim file content at line 165")?;
            let first_line = content.lines().next().unwrap().trim();
            first_line
                .replace("path =", "")
                .replace("\"", "")
                .trim()
                .to_owned()
        };
        let ps1_script = path.to_str().unwrap().replace(".cmd", ".ps1");
        let ps1_script = Path::new(&ps1_script);
        if ps1_script != path.as_path() && ps1_script.exists() {
            shims.push((
                file_name,
                ps1_script.to_str().unwrap().to_owned(),
                shim_source,
            ));
        } else {
            shims.push((file_name, path.to_str().unwrap().to_owned(), shim_source));
        }
    }
    let count = shims.len();
    if count == 0 {
        println!(
            "{}",
            tr_fmt!(
                "No shims found for regex: {regex}",
                "未找到匹配正则表达式 '{regex}' 的 shim",
                regex = regex
            )
            .dark_green()
            .bold()
        );
        return Ok(());
    }
    println!(
        "{}{}",
        tr("\tFound shims count: ", "\t找到 shim 数量: ")
            .dark_green()
            .bold(),
        count.to_string().dark_green().bold()
    );

    let shims_vec = shims
        .iter()
        .map(|(name, path, source)| vec![name, path, source])
        .collect::<Vec<_>>();

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new(tr("ShimName", "Shim名称"))
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
            Cell::new(tr("Path", "路径"))
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
            Cell::new(tr("Source", "来源"))
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkCyan),
        ])
        .add_rows(shims_vec.as_slice());

    log::debug!("{:?}", shims.get(0).unwrap());

    println!("{table}");

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
        bail!(tr_fmt!(
            "Shim directory '{path}' does not exist",
            "shim 目录 '{path}' 不存在",
            path = shim_path.display()
        ));
    }
    for entry in shim_path
        .read_dir()
        .context("Failed to read  shim root dir at line 127")?
    {
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
        let app_type = if file_name.ends_with(".exe") {
            "Application"
        } else {
            "ExternalScript"
        };
        let shim_path = path.to_str().unwrap().to_owned();
        println!(
            "{:<10} : {}",
            tr("Name", "名称"),
            shim_name.clone().dark_green().bold()
        );
        println!(
            "{:<10} : {}",
            tr("Path", "路径"),
            shim_path.clone().dark_green().bold()
        );
        println!(
            "{:<10} : {}",
            tr("Source", "来源"),
            shim_name.clone().dark_green().bold()
        );
        println!(
            "{:<10} : {}",
            tr("Type", "类型"),
            app_type.dark_green().bold()
        );
        println!(
            "{:<10} : {}",
            tr("IsGlobal", "是否全局"),
            if global { "True" } else { "False" }.dark_green().bold()
        );
        println!(
            "{:<10} : {}",
            tr("IsHidden", "是否隐藏"),
            "False".dark_green().bold()
        );
        return Ok(());
    }

    println!(
        "{}",
        tr_fmt!(
            "No shim found for app: {name}",
            "未找到应用的 shim: {name}",
            name = shim_name.dark_cyan().bold()
        )
        .dark_red()
        .bold()
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
        bail!(tr("Command path is required", "命令路径为必填项"));
    }
    let shim_name = shim_name.unwrap();

    validate_app_name(&shim_name)?;
    let target_path = command_path.unwrap().trim().to_string();
    if target_path.is_empty() {
        bail!(tr("Command path is empty", "命令路径为空"));
    }

    let shim_path = if global {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    if args.is_none() {
        create_shims(shim_name.as_str(), &shim_path, &target_path, "")?;
    } else {
        create_shims(
            shim_name.as_str(),
            &shim_path,
            &target_path,
            args.unwrap().as_str(),
        )?;
    }
    println!(
        "{}",
        tr_fmt!(
            "Shim '{name}' has been created successfully!",
            "Shim '{name}' 创建成功！",
            name = shim_name
        )
        .dark_green()
        .bold()
    );
    Ok(())
}

pub fn create_shims<'a>(
    shim_name: &str,
    shim_dir: &str,
    target_path: &str,
    args: impl Into<Option<&'a str>>,
) -> anyhow::Result<()> {
    let out_dir = PathBuf::from(shim_dir);
    let args = args.into().unwrap_or_default();
    let options = vec![];
    let suffix = target_path.split('.').last().unwrap();
    // log::debug!("Origin file type {}", suffix);
    if suffix.is_empty() {
        bail!(
            "{}",
            tr_fmt!(
                "Shim name '{shim_name}' has empty extension",
                "shim 文件名 {shim_name} 后缀为空",
                shim_name = shim_name
            )
        );
    }
    if !out_dir.exists() {
        bail!(
            "{}",
            tr_fmt!(
                "Shim directory '{shim_dir}' does not exist",
                "shim 目录 {shim_dir} 不存在",
                shim_dir = shim_dir
            )
        );
    }
    if shim_name == "hp" {
        bail!(
            "{}",
            tr(
                "'hp' cannot be used as a shim name",
                "hp 不能作为 shim 名称"
            )
        );
    }
    if !Path::new(target_path).exists() {
        bail!(
            "{}",
            tr_fmt!(
                "Target path '{target_path}' does not exist",
                "链接目标路径 {target_path} 不存在",
                target_path = target_path
            )
        );
    };
    if suffix == "exe" || suffix == "com" {
        if !args.is_empty() {
            create_exe_type_shim_file_and_shim_bin(
                target_path,
                out_dir,
                Some(shim_name.into()),
                Some(args.to_owned()),
                options.as_slice(),
            )?;
        } else {
            create_exe_type_shim_file_and_shim_bin(
                target_path,
                out_dir,
                Some(shim_name.into()),
                None,
                options.as_slice(),
            )?;
        }
    } else if suffix == "cmd" || "bat" == suffix {
        let result = exclude_scoop_self_scripts(target_path, Some(shim_name))?;
        if result != 0 {
            bail!(
                "{}",
                tr_fmt!(
                    "Binary name or alias '{shim_name}' conflicts with scoop built-in script shim, overwrite prohibited",
                    "Origin 二进制名或者该二进制别名 '{shim_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖",
                    shim_name = shim_name
                )
            );
        }
        if !args.is_empty() {
            create_cmd_or_bat_shim_scripts(
                target_path,
                out_dir,
                Some(shim_name.into()),
                Some(args.to_owned()),
                options.as_slice(),
            )?;
        } else {
            create_cmd_or_bat_shim_scripts(
                target_path,
                out_dir,
                Some(shim_name.into()),
                None,
                options.as_slice(),
            )?;
        }
    } else if suffix == "ps1" {
        let result = exclude_scoop_self_scripts(target_path, Some(shim_name))?;
        if result != 0 {
            bail!(
                "{}",
                tr_fmt!(
                    "Binary name or alias '{shim_name}' conflicts with scoop built-in script shim, overwrite prohibited",
                    "Origin 二进制名或者该二进制别名 '{shim_name}' 与scoop 内置脚本的shim 冲突, 禁止覆盖",
                    shim_name = shim_name
                )
            );
        }
        if !args.is_empty() {
            create_ps1_shim_scripts(
                target_path,
                out_dir,
                Some(shim_name.into()),
                Some(args.to_owned()),
                options.as_slice(),
            )?
        } else {
            create_ps1_shim_scripts(
                target_path,
                out_dir,
                Some(shim_name.into()),
                None,
                options.as_slice(),
            )?;
        };
    } else if suffix == "jar" {
        if !args.is_empty() {
            create_jar_shim_scripts(
                target_path,
                out_dir,
                Some(shim_name.into()),
                Some(args.to_owned()),
                options.as_slice(),
            )?;
        } else {
            create_jar_shim_scripts(
                target_path,
                out_dir,
                Some(shim_name.into()),
                None,
                options.as_slice(),
            )?;
        }
    } else if suffix == "py" {
        if !args.is_empty() {
            create_py_shim_scripts(
                target_path,
                out_dir,
                Some(shim_name.into()),
                Some(args.to_owned()),
                options.as_slice(),
            )?
        } else {
            create_py_shim_scripts(
                target_path,
                out_dir,
                Some(shim_name.into()),
                None,
                options.as_slice(),
            )?;
        }
    } else {
        bail!(
            "{}",
            tr_fmt!(
                "File extension '.{suffix}' is not supported",
                "后缀.{suffix}类型文件不支持",
                suffix = suffix
            )
        )
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
        bail!(tr_fmt!(
            "Shim directory '{path}' does not exist",
            "shim 目录 '{path}' 不存在",
            path = shim_path.display()
        ));
    }
    let shim_dir =
        std::fs::read_dir(&shim_path).context("Failed to read  shim root dir at line 366")?;
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
            eprintln!(
                "{}",
                tr_fmt!(
                    "'{path}' does not exist",
                    "'{path}' 不存在",
                    path = shim_path.display()
                )
            );
            return;
        }
        println!(
            "{}",
            tr_fmt!(
                "Removing shim: {path}",
                "正在删除 shim: {path}",
                path = shim_path.display()
            )
            .dark_green()
            .bold()
        );
        std::fs::remove_file(shim_path).unwrap();
    });
    Ok(())
}

pub fn clear_invalid_shims(global: bool) -> anyhow::Result<()> {
    let shim_root_dir = if global {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    let shim_root_dir = Path::new(&shim_root_dir);
    let result = shim_root_dir.read_dir()?.try_for_each(|entry| {
        let entry = entry.ok().unwrap();
        let file_type = entry.file_type().ok().unwrap();
        if !file_type.is_file() {
            return Ok(());
        }
        let path = entry.path();
        let extension = path.extension().unwrap_or_default().to_str().unwrap();
        if extension.is_empty() {
            return Ok(());
        }
        if extension == "shim" {
            let content = std::fs::read_to_string(&path).unwrap();
            let first_line = content.lines().next().unwrap().trim();
            let target_path = first_line
                .replace("path =", "")
                .replace("\"", "")
                .trim()
                .to_owned();
            let exe_path = path.with_extension("exe");
            if !Path::new(&target_path).exists() {
                println!(
                    "{}",
                    tr_fmt!(
                        "Removing invalid shim: {path}",
                        "正在删除无效 shim: {path}",
                        path = path.display()
                    )
                    .dark_green()
                    .bold()
                );
                println!(
                    "{}",
                    tr_fmt!(
                        "Removing invalid shim: {path}",
                        "正在删除无效 shim: {path}",
                        path = exe_path.display()
                    )
                    .dark_green()
                    .bold()
                );
                std::fs::remove_file(&path).unwrap();
                if exe_path.exists() {
                    std::fs::remove_file(&exe_path).unwrap();
                }
            }
        } else if extension == "cmd" || extension == "bat" {
            let content = extract_rem_comments(path.to_str().unwrap());
            let target_path = content.trim().to_owned();
            let shell_path = path.with_extension("");
            if !Path::new(&target_path).exists() {
                println!(
                    "{}",
                    tr_fmt!(
                        "Removing invalid shim: {path}",
                        "正在删除无效 shim: {path}",
                        path = path.display()
                    )
                    .dark_green()
                    .bold()
                );
                println!(
                    "{}",
                    tr_fmt!(
                        "Removing invalid shim: {path}",
                        "正在删除无效 shim: {path}",
                        path = shell_path.display()
                    )
                    .dark_green()
                    .bold()
                );
                std::fs::remove_file(&path).unwrap();
                if !shell_path.exists() {
                    return Ok(());
                }
                std::fs::remove_file(&shell_path).unwrap();
            }
        } else if extension.is_empty() {
            log::info!("Current app is shell script of running with wsl");
            let target_path = extract_target_path_from_shell_script(path.to_str().unwrap())?;
            let cmd_path = path.with_extension("cmd");

            if !Path::new(&target_path).exists() {
                println!(
                    "{}",
                    tr_fmt!(
                        "Removing invalid shim: {path}",
                        "正在删除无效 shim: {path}",
                        path = path.display()
                    )
                    .dark_green()
                    .bold()
                );

                println!(
                    "{}",
                    tr_fmt!(
                        "Removing invalid shim: {path}",
                        "正在删除无效 shim: {path}",
                        path = cmd_path.display()
                    )
                    .dark_green()
                    .bold()
                );
                std::fs::remove_file(&path).unwrap();
                std::fs::remove_file(&cmd_path).unwrap();
            }
        }
        Ok(())
    }) as anyhow::Result<()>;

    if result.is_err() {
        bail!(result.unwrap_err());
    }
    Ok(())
}

mod test_shim {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    fn test_shim_count() {
        use std::collections::HashSet;
        let shim_root_dir = get_shims_root_dir();
        let shim_root_dir = Path::new(&shim_root_dir);
        let mut set = HashSet::new();

        for entry in shim_root_dir.read_dir().unwrap() {
            let entry = entry.ok().unwrap();
            let file_type = entry.file_type().ok().unwrap();
            let path = entry.path();
            if !file_type.is_file() {
                continue;
            }

            let file_name = path.file_stem().unwrap().to_str().unwrap();
            if file_name.starts_with("scoop-") {
                continue;
            }
            set.insert(file_name.to_string());
        }

        let shims = list_all_shims(false).unwrap();
        let shim_names = shims.iter().map(|s| s.0.clone()).collect::<Vec<String>>();

        let diff: Vec<_> = set
            .iter()
            .filter(|item| !shim_names.contains(item))
            .collect();

        println!("shims count: {:?}", shim_names.len());
        println!("set count {}", set.len());
        println!("diff   {:?}", diff);
    }
}
