use crate::init_hyperscoop;
use crate::manifest::manifest_deserialize::StringArrayOrString;
use crate::manifest::uninstall_manifest::UninstallManifest;
use crate::utils::invoke_hook_script::*;
use anyhow::bail;
use crossterm::style::Stylize;
use serde_json::Value;
use std::path::{Path, PathBuf};
mod env_set;
use env_set::*;
pub(crate) mod shim_and_shortcuts;
use shim_and_shortcuts::*;
pub fn uninstall_app_with_purge(app_name: &str) -> Result<(), anyhow::Error> {
    uninstall_app(app_name)?;
    println!(
        "{} '{}'", "Removing Persisted data for".to_string().dark_blue().bold(), 
        app_name.dark_cyan().bold()
    );
    let hyperscoop = init_hyperscoop()?;
    let persist_path = hyperscoop.get_persist_path();
    let app_persist_path = Path::new(&persist_path).join(app_name);
    log::info!("Removing {}", app_persist_path.display());
    if !app_persist_path.exists() {
        eprintln!(
            "{} {}",
            "persisted data is not  having".dark_red().bold(),
            app_persist_path.to_str().unwrap().dark_green().bold()
        );
        return Ok(());
    }
    std::fs::remove_dir_all(app_persist_path)?;
    Ok(())
}

pub fn uninstall_app(app_name: &str) -> Result<(), anyhow::Error> {
    let hyperscoop = init_hyperscoop()?;
    let app_path = hyperscoop.get_apps_path();
    let shim_path = hyperscoop.get_shims_path();
    let lower = app_name.to_lowercase();
    let app_name = lower.as_str();
    if app_name == "scoop" {
        let mut uninstall_script = Path::new(&app_path)
            .join("scoop")
            .join("current")
            .join("bin")
            .join("uninstall.ps1");
        if !uninstall_script.exists() {
            uninstall_script = Path::new(&shim_path)
                .join("apps")
                .join("scoop")
                .join("current")
                .join("bin")
                .join("uninstall.ps1");
            if !uninstall_script.exists() {
                bail!("Scoop Uninstall script not found");
            }
        }
        log::info!(
            "Running Scoop Uninstall script {}",
            uninstall_script.display()
        );
        let output = std::process::Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(uninstall_script)
            .output()?;
        if !output.status.success() {
            bail!("Scoop Uninstall script failed");
        }
        println!("Scoop Uninstall script completed successfully");
        std::process::exit(0);
    }
    if let Err(e) = check_installed_status(app_name) {
        eprintln!("{}", e);
        bail!("checked installed status, {e}");
    }
    let result = uninstall_matched_app(app_path.clone(), app_name, shim_path.clone());
    if let Err(e) = result {
        eprintln!("{}", e);
        let app_path = Path::new(&app_path).join(app_name); 
        if!app_path.exists() {
             eprintln!("{} is not exists", app_path.display());
             return Ok(()); 
        }
        let app_path =app_path.to_str().unwrap();
        println!("{}", format!("Removing Error  Installation of '{app_name}' => {app_path}")) ; 
        rm_all_dir(app_path)?;
       bail!(
        "'{}' {}",
        app_name.to_string().dark_cyan().bold(),
        "was not uninstalled as expect".dark_green().bold()
      );
    }
  
     Ok(())
}  

fn uninstall_matched_app(
  app_path: String,
  app_name: &str,
  shim_path: String,
) -> Result<(), anyhow::Error> {
  for entry in std::fs::read_dir(app_path)? {
    let entry = entry?;
    let path = entry.path();
    if let Some(file_name) = path.file_name() {
      if file_name.to_str().unwrap().to_lowercase() == app_name {
        let current_path = path.join("current");
        let manifest_path = current_path.join("manifest.json");
        let install_path = current_path.join("install.json");

        if !install_path.exists() {
          log::error!("{} is not existing ", install_path.display());
        }
        if !manifest_path.exists() {
          bail!("{} is not  existing ", manifest_path.display());
        }
        let contents = std::fs::read_to_string(manifest_path)?;
        let mut manifest: UninstallManifest = serde_json::from_str(&contents)?;
        manifest.set_name(&app_name.to_string()); // 先进行可变借用

        let version = &manifest.version;
        if version.is_none() {
          bail!("version is not existing")
        };
        let version = version.clone().unwrap();
        let install_info = std::fs::read_to_string(install_path)?;
        let install_info: serde_json::Value = serde_json::from_str(&install_info)?;
        let arch = install_info["architecture"].as_str().unwrap_or("Unknown");
        invoke_hook_script(HookType::PreUninstall, &manifest, arch)?;
        println!(
          "{} '{}'  ({})", "Uninstalling".to_string().dark_blue().bold(),
          app_name.dark_red().bold(),
          version.dark_red().bold()
        );
        invoke_hook_script(HookType::Uninstaller, &manifest, arch)?;
        invoke_hook_script(HookType::PostUninstall, &manifest, arch)?;
        uninstall_psmodule(&manifest)?;

        env_path_var_rm(&current_path, &manifest)?;

        env_var_rm(&manifest)?;
        rm_shim_file(shim_path.clone(), &manifest, app_name)?;
        rm_start_menu_shortcut(&manifest)?;
        println!("{} {}","Unlinking".dark_blue().bold(), &current_path.display().to_string().dark_green().bold());
        rm_all_dir(path.clone())?;
        return Ok(());
      }
    }
  }
  Ok(())
}
fn env_path_var_rm(current: &PathBuf, manifest: &UninstallManifest) -> Result<(), anyhow::Error> {
    use winreg::enums::*;
    use winreg::RegKey;
    if let Some(StringArrayOrString::String(env_add_path_str)) = manifest.env_add_path.clone() {
        let path_var = current.join(env_add_path_str);
        if path_var.exists() {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let environment_key = hkcu.open_subkey("Environment")?;
            let user_path: String = environment_key.get_value("PATH")?;
            log::trace!("\n 当前用户的 PATH: {}", user_path);
            let mut paths: Vec<PathBuf> = std::env::split_paths(&user_path).collect();
            paths.retain(|p| p != &path_var);
            let user_path = paths
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect::<Vec<String>>()
                .join(";");
            log::trace!("\n 更新后的用户的 PATH: {}", user_path);

            // environment_key.set_value("PATH", &user_path)?;
            let script = format!(
                r#"[System.Environment]::SetEnvironmentVariable("PATH","{user_path}", "User")"#
            );
            let output = std::process::Command::new("powershell")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-Command")
                .arg(script)
                .output()?;
            if !output.status.success() {
                bail!("Failed to remove path var");
            }
        }
    } else if let Some(StringArrayOrString::StringArray(env_add_path_arr)) =
        manifest.env_add_path.clone()
    {
        let env_add_path_arr = env_add_path_arr
            .iter()
            .map(|p| current.join(p.as_str()))
            .collect::<Vec<PathBuf>>();

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let environment_key = hkcu.open_subkey("Environment")?;

        let user_path: String = environment_key.get_value("PATH")?;
        let origin = user_path.clone();
        log::trace!("\n 当前用户的 PATH: {}", user_path);
        let mut paths: Vec<PathBuf> = std::env::split_paths(&user_path).collect();

        for path_var in env_add_path_arr {
            paths.retain(|p| p != &path_var);
        }

        let user_path = paths
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join(";");
        log::trace!("\n 更新后的用户的 PATH: {}", user_path);
        if user_path == origin {
            log::trace!("\n 没有需要移除的路径变量");
            return Ok(());
        }
        let script = format!(
            r#"[System.Environment]::SetEnvironmentVariable("PATH","{user_path}", "User")"#
        );
        let output = std::process::Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(script)
            .output()?;
        if !output.status.success() {
            bail!("Failed to remove path var");
        }
    }
    Ok(())
}

fn uninstall_psmodule(manifest: &UninstallManifest) -> Result<(), anyhow::Error> {
    let psmodule = manifest.clone().psmodule;
    if psmodule.is_none() {
        return Ok(());
    }
    let psmodule = psmodule.unwrap();
    let hp = init_hyperscoop()?;
    let psmodule_dir = hp.get_psmodule_path();
    let module_name = psmodule.get("name").unwrap().as_str().unwrap();
    println!(
        "Uninstalling PowerShell module  '{}'",
        module_name.dark_red().bold()
    );
    let lind_path = Path::new(&psmodule_dir).join(module_name);
    if lind_path.exists() {
        println!("Removing psmodule path {}", &lind_path.display());
        std::fs::remove_dir_all(lind_path)?;
    }
    Ok(())
}

fn rm_all_dir<P: AsRef<Path> >(path: P ) -> Result<(), anyhow::Error> {
    match std::fs::remove_dir_all(path) {
        Ok(_) => Ok(()),
        Err(err) => {
            bail!("{}", err.to_string().red().bold());
        }
    }
}

fn check_installed_status(app_name: &str) -> Result<bool, anyhow::Error> {
    use regex::Regex;
    let pattern = r"[\[\]\(\)\*\+\?\{\}\|\^\$\#]";
    let re = Regex::new(pattern)?;
    if re.is_match(app_name) {
        bail!(
            "'{}' {}",
            app_name.red().bold(),
            "is not valid app name".red().bold()
        );
    }
    let app_path = init_hyperscoop()?.get_apps_path();
    let app_path = Path::new(&app_path).join(app_name);
   if  !app_path.exists() {
        bail!(
            "'{}' {}",
            app_name.red().bold(),
            "is not installed".red().bold()
        );
    }
    let is_having_current = app_path.join("current").exists();
    if !is_having_current {
        bail!(
            "'{}' {}",
            app_name.red().bold(),
            "don't have current dir,is not installed correctly".red().bold()
        );
    }
    let manifest = app_path.join("current").join("manifest.json"); 
    let  install_json = app_path.join("current").join("install.json"); 
     if  !install_json.exists() {
       bail!(
            "'{}' {}",
            app_name.red().bold(),
            "don't have install.json, is not installed correctly".red().bold()
        );
     }
    if !manifest.exists() {
        bail!(
            "'{}' {}",
            app_name.red().bold(),
            "don't have manifest.json, is not installed correctly".red().bold()
        );
    }
    
    Ok(true)
}
