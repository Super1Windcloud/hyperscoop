use crate::init_env::*;
use crossterm::style::Stylize;
use std::path::Path;
use crate::install::create_shim_or_shortcuts;

pub fn check_before_install(name: &String, version: &String) -> anyhow::Result<u8> {
    let app_dir = get_app_dir(name);
    let app_dir_path = Path::new(&app_dir);
    if !app_dir_path.exists() {
        return Ok(0);
    }
    let app_version_dir = get_app_version_dir(name, &version);
    let app_current_dir = get_app_current_dir(name.into());
    let app_version_path = Path::new(&app_version_dir);
    let app_current_path = Path::new(&app_current_dir);
    if app_version_path.exists() &&  app_current_path.exists() {
        let install_json = get_app_dir_install_json(name);
        if Path::new(&install_json).exists() {
            println!(
                "{}",
                format!("WARNING  '{name }' ({version}) is already installed")
                    .to_string()
                    .dark_cyan()
                    .bold(),
            );
          println!("{}" ,format!("You can use 'hp update {name}' to  install another version").to_string().dark_cyan().bold());
            return Ok(1);
        }
    } else if  app_version_path.exists() && !app_current_path.exists()  {
        println!(
          "{}",
          "WARNING  修复缺失的链接和快捷方式".to_string()
                .dark_cyan()
                .bold()
        );
        println!(
            "{}",
            format!("Resetting '{name}' ({version})").dark_cyan().bold()
        );
        create_dir_symbolic_link(&app_version_dir, &app_current_dir)?;
       let  manifest_json = get_app_dir_manifest_json(name) ;  
        create_shim_or_shortcuts(manifest_json ,name )?;  
        let install_json = app_version_dir.clone() + "\\install.json";
        if Path::new(&install_json).exists() {
            println!(
                "{}",
                format!("WARNING  '{name}' ({version}) is already installed")
                    .to_string()
                    .dark_cyan()
                    .bold(),
            );
            return Ok(1);
        } else if !app_version_path.exists() &&  app_current_path.exists() {
            println!(
                "{}",
                format!("WARNING  '{name}' 先清除之前安装失败的文件")
                    .dark_cyan()
                    .bold(),
            );
            println!(
                "{}",
                format!("ERROR   '{name}' isn't installed correctly")
                    .dark_red()
                    .bold(),
            );
          println!(
                "{}",
                format!("'{name}' was uninstalled ")
                    .dark_green()
                    .bold(),
            );
        }
    }
    Ok(0)
}

pub fn create_dir_symbolic_link(version_dir: &String, current_dir: &String) -> anyhow::Result<()> {
    std::os::windows::fs::symlink_dir(version_dir, current_dir)?;    
    println!("Linking  {}", format!("{version_dir}  => {current_dir}").dark_green().bold());
    Ok(())
}
