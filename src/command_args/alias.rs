use anyhow::{bail, Context};
use clap::ArgAction;
use clap::{Args, Subcommand};
use command_util_lib::init_env::{get_shims_root_dir, get_shims_root_dir_global};
use command_util_lib::utils::utility::clap_args_to_lowercase;
use crossterm::style::Stylize;
use rayon::prelude::*;
use std::cmp::max;
use std::env;
use std::path::Path;
use command_util_lib::utils::system::{is_admin, request_admin};

#[derive(Debug, Clone, Args)]
#[clap(author, version, about="🎉\t\t创建Window终端命令的别名",  long_about = None)]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
#[command(after_help = r#"
Available subcommands: add, rm, list.
Aliases are custom hp subcommands that can be created to make common tasks easier.
To add an alias:       hp alias add <name> <command> [<description>], 自动生成一个hp-<name>的别名
To rm an alias:        hp alias rm <name> , 删除时别名名称必须以hp-开头
To list all aliases:   hp alias list  , 包含name, command , description 字段
示例:  hp alias add rm 'hp uninstall $args[0]' 'Uninstall an app' [描述内容可选]
      alias_name创建之后, 运行hp-<alias> ,例如运行hp-rm 就可以替代 hp uninstall命令进行操作
"#)]
pub struct AliasArgs {
    #[command(subcommand)]
    pub(crate) command: Option<AliasSubcommands>,
    #[arg(from_global)]
    pub global: bool,
}

#[derive(Subcommand, Debug, Clone)]
#[command(no_binary_name = true)]
#[command(infer_subcommands = true, infer_long_args = true)]
#[command(disable_help_subcommand = true, next_line_help = false)]
pub enum AliasSubcommands {
    Add(AddArgs),
    List(ListArgs),
    Rm(RmArgs),
}

#[derive(Args, Debug, Clone)]
#[command(about = "删除一个alias shim")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct RmArgs {
    #[arg(required = false, help = "删除的alias名称",
    value_parser = clap_args_to_lowercase,)]
    pub(crate) alias_name: Option<String>,
    #[arg(required = false, short, long, help = "删除所有的alias")]
    pub all: bool,
}

#[derive(Args, Debug, Clone)]
#[command(about = "添加一个alias shim")]
#[command(arg_required_else_help = true, subcommand_negates_reqs = true)]
pub struct AddArgs {
    #[arg(required = false, help = "添加的alias名称"
        ,value_parser = clap_args_to_lowercase,
        action = ArgAction::Set
    )]
    pub(crate) alias_name: Option<String>,
    #[arg(required = false, help = "alias的目标命令")]
    pub(crate) command: Option<String>,

    #[arg(required = false, help = "alias的描述")]
    pub(crate) description: Option<String>,
}

#[derive(Args, Debug, Clone)]
#[command(about = "列出所有alias的ps1脚本 ")]
pub struct ListArgs {}

pub fn execute_alias_command(args: AliasArgs) -> anyhow::Result<()> { 
    if args.global && !is_admin()? {
      let args =env::args().skip(1). collect::<Vec<String>>();
      let  args_str= args.join(" ");
      log::warn!("Global command arguments: {}", args_str.clone().dark_yellow());
      request_admin( args_str.as_str())?;
      return Ok(());
    }
  
    let shim_root_dir = if args.global {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    match args.command {
        Some(AliasSubcommands::Add(add_args)) => {
            add_alias(
                add_args.alias_name,
                add_args.command,
                &shim_root_dir,
                add_args.description,
            )?;
        }
        Some(AliasSubcommands::List(_)) => {
            list_alias(&shim_root_dir)?;
        }
        Some(AliasSubcommands::Rm(rm_args)) => {
            rm_alias(rm_args.alias_name, &shim_root_dir, rm_args.all)?;
        }
        None => {}
    }

    Ok(())
}

fn rm_alias(alias_name: Option<String>, shim_root_dir: &str, all: bool) -> anyhow::Result<()> {
    if alias_name.is_none() && !all {
        return Ok(());
    }
    if all {
        let dirs = std::fs::read_dir(shim_root_dir)
          .context("Failed to read shim root directory at line 104")?;
        for dir in dirs {
            let dir = dir.context("Failed to read directory at line 106")?;
            let child_type = dir.file_type()?;
            if child_type.is_dir() {
                continue;
            }
            let path = dir.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name.starts_with("hp-") && file_name.ends_with(".ps1") {
                println!("{}", format!("remove file: {}", path.display()).dark_grey());
                std::fs::remove_file(path).context("Failed to remove ps1 script at line 115")?;
            }
        }
        return Ok(());
    }
    let alias_name = alias_name.unwrap();
    let shim_ps_script = format!("{}\\{}.ps1", shim_root_dir, alias_name);
    if !Path::new(&shim_ps_script).exists() {
        bail!(format!("alias  {} does not exist", alias_name));
    } else {
        println!(
            "{}",
            format!("Alias '{alias_name}' is removed successfully!")
                .dark_green()
                .bold()
        );
        std::fs::remove_file(&shim_ps_script).context("Failed to remove ps1 script at line 131")?;
    }
    Ok(())
}

fn list_alias(shim_root_dir: &str) -> anyhow::Result<()> {
    let dirs = std::fs::read_dir(shim_root_dir)
      .context("Failed to read shim root directory at line 138")?;

    let result = dirs
        .par_bridge()
        .filter_map(|dir| {
            let dir = dir.unwrap();
            let child_type = dir.file_type().unwrap();
            if child_type.is_dir() {
                None
            } else {
                let path = dir.path();
                let file_name = path.file_name().unwrap();
                let file_name = file_name.to_str().unwrap();
                if file_name.starts_with("hp-") && file_name.ends_with(".ps1") {
                    let content = std::fs::read_to_string(&path).unwrap();
                    let contents = content.lines().collect::<Vec<&str>>(); // 自动处理CRLF
                    let contents = contents
                        .into_iter()
                        .filter(|line| !line.trim().is_empty())
                        .collect::<Vec<&str>>();
                    let line_count = contents.len();
                    let file_name = file_name.replace(".ps1", "");
                    log::debug!(
                        "{}",
                        format!("file_name: {}, line_count: {}", file_name, line_count).green()
                    );
                    if line_count == 1 {
                        Some((file_name.into(), contents[0].trim().to_owned(), None))
                    } else if line_count == 2 {
                        let first = contents[0].trim();
                        let second = contents[1].trim();
                        let summary = first
                            .replace("#", "")
                            .replace("Summary:", "")
                            .trim()
                            .to_string();
                        Some((file_name.to_owned(), second.to_owned(), Some(summary)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        })
        .collect::<Vec<_>>();
    let len = result.len();
    if len == 0 {
        println!("{}", "No alias found".dark_green().bold());
        return Ok(());
    }
    let alias_width = max(
        result
            .iter()
            .map(|(alias_name, _, _)| alias_name.len())
            .max()
            .unwrap(),
        10,
    ) + 5;
    let command_width = max(
        result
            .iter()
            .map(|(_, command, _)| command.len())
            .max()
            .unwrap(),
        7,
    ) + 5;
    let summary_width = max(
        result
            .iter()
            .map(|(_, _, summary)| {
                if let Some(summary) = summary {
                    summary.len()
                } else {
                    0
                }
            })
            .max()
            .unwrap(),
        7,
    );
    let mut flag = false;
    for (alias_name, command, summary) in result {
        if !flag {
            use console::{pad_str, style, Alignment};

            let alias_title = pad_str("Alias_Name", alias_width, Alignment::Left, None);
            let command_title = pad_str("Command", command_width, Alignment::Left, None);
            let summary_title = pad_str("Summary", summary_width, Alignment::Left, None);

            println!(
                "{}{}{}",
                style(alias_title).bold().green(),
                style(command_title).bold().green(),
                style(summary_title).bold().green()
            );
            println!(
                "{:width1$}{:<width2$}{:<width3$}",
                "-".repeat(10),
                "-".repeat(7),
                "-".repeat(7),
                width1 = alias_width,
                width2 = command_width,
                width3 = summary_width
            );
        }
        flag = true;
        if let Some(summary) = summary {
            println!(
                "{:width1$}{:<width2$}{:<width3$}",
                alias_name,
                command,
                summary,
                width1 = alias_width,
                width2 = command_width,
                width3 = summary_width
            )
        } else {
            println!(
                "{:width1$}{:<width2$} ",
                alias_name,
                command,
                width1 = alias_width,
                width2 = command_width,
            )
        }
    }
    Ok(())
}

fn add_alias(
    alias_name: Option<String>,
    target_command: Option<String>,
    shim_root_dir: &str,
    description: Option<String>,
) -> anyhow::Result<()> {
    if alias_name.is_none() || target_command.is_none() {
        bail!("target command  can't be empty");
    }
    let [alias_name, target_command] = [alias_name.unwrap(), target_command.unwrap()];
    let alias_ps_path = format!("{}\\hp-{}.ps1", shim_root_dir, alias_name);
    if Path::new(&alias_ps_path).exists() {
        bail!(format!("Alias already exists at {}", alias_ps_path));
    }
    let description = if let Some(description) = description {
        description
    } else {
        String::from("")
    };
    let alias_ps_content = format!(
        r#"
    # Summary: {description}
    {target_command}
    "#
    );
    std::fs::write(&alias_ps_path, alias_ps_content).context("Failed to write ps1 script")?;
    println!(
        "{}",
        format!("Alias command(hp-{alias_name})  created successfully!")
            .dark_green()
            .bold()
    );
    Ok(())
}
