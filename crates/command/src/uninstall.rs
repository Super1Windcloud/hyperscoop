use crate::init_hyperscoop;
use crate::utils::invoke_hook_script::*;
use anyhow::bail;
use crossterm::style::Stylize;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn uninstall_app_with_purge(app_name: &str) -> Result<(), anyhow::Error> {
    uninstall_app(app_name)?;
    println!(
        "Removing Persisted data for '{}'",
        app_name.dark_red().bold()
    );
    let hyperscoop = init_hyperscoop()?;
    let persist_path = hyperscoop.get_persist_path();
    let app_persist_path = Path::new(&persist_path).join(app_name);
    log::info!("Removing {}", app_persist_path.display());
    if !app_persist_path.exists() {
        eprintln!(
            "{} {}",
            "persisted data is not  having".dark_red().bold(),
            app_persist_path.display()
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
    }
    for entry in std::fs::read_dir(app_path)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            if file_name.to_str().unwrap().to_lowercase() == app_name {
                let current_path = path.join("current");
                let manifest_path = current_path.join("manifest.json");
                let install_path = current_path.join("install.json");

                if !manifest_path.exists() || !install_path.exists() {
                    bail!("{} is not  existing ", manifest_path.display());
                }
                let contents = std::fs::read_to_string(manifest_path)?;
                let manifest: serde_json::Value = serde_json::from_str(&contents)?;
                let version = manifest["version"].as_str().unwrap_or("Unknown");
                let install_info = std::fs::read_to_string(install_path)?;
                let install_info: serde_json::Value = serde_json::from_str(&install_info)?;
                let arch = install_info["architecture"].as_str().unwrap_or("Unknown");
                invoke_hook_script(HookType::PreUninstall, &manifest, arch)?;
                println!(
                    "Uninstalling '{}'  ({})",
                    app_name.dark_red().bold(),
                    version.dark_red().bold()
                );
                invoke_hook_script(HookType::Uninstaller, &manifest, arch)?;
                env_path_var_rm(&current_path, &manifest)?;
                env_var_rm(&manifest)?;
                rm_shim_file(shim_path.clone(), &manifest ,app_name  )?;
                rm_start_menu_shortcut()?;
                println!("Unlinking {}", &current_path.display());

                invoke_hook_script(HookType::PostUninstall, &manifest, arch)?;

                uninstall_psmodule(&manifest)?;
                 // rm_all_dir(path.clone())? ;
                return Ok(());
            }
        }
    }
    bail!(
        "'{}' {}",
        app_name.red().bold(),
        "is not installed".red().bold()
    );
}

fn env_var_rm(manifest: &Value) -> Result<(), anyhow::Error> {
    let env_set = manifest["env_set"].as_object();
    if env_set.is_none() || env_set.unwrap().is_empty() {
        return Ok(());
    }
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment_key = hkcu.open_subkey("Environment")?;
    let env_set = env_set.unwrap();
    for (key, _ ) in env_set {
        let env_value: String = environment_key.get_value(key).unwrap_or("".into());
        if env_value.is_empty() {
            continue;
        }
        if Path::new(&env_value).exists() {
            if Path::new(&env_value).is_dir() {
                std::fs::remove_dir_all(&env_value)?;
            } else if Path::new(&env_value).is_file() {
                std::fs::remove_file(&env_value)?;
            }
        }
        let cmd = format!(r#"Remove-ItemProperty -Path "HKCU:\Environment" -Name {key}"#);
        let output = std::process::Command::new("powershell")
            .arg("-Command")
            .arg(cmd)
            .output()?;
        if !output.status.success() {
            bail!("powershell failed to set environment variable");
        }

        log::trace!("env set  : {}", env_value);
    }
    Ok(())
}

fn env_path_var_rm(current: &PathBuf, manifest: &Value) -> Result<(), anyhow::Error> {
    use winreg::enums::*;
    use winreg::RegKey;

    let env_add_path_str = manifest["env_add_path"].as_str();
    let env_add_path_arr = manifest["env_add_path"].as_array();
    if env_add_path_str.is_none() && env_add_path_arr.is_none() {
        return Ok(());
    }
    if env_add_path_str.is_some() {
        let path_var = current.join(env_add_path_str.unwrap());
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
    }
    if env_add_path_arr.is_some() {
        let env_add_path_arr = env_add_path_arr.unwrap();
        let env_add_path_arr = env_add_path_arr
            .iter()
            .map(|p| current.join(p.as_str().unwrap()))
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

fn uninstall_psmodule(manifest: &Value) -> Result<(), anyhow::Error> {
    let psmodule = manifest["psmodule"]
        .as_object()
        .map(|o| serde_json::to_string(o).unwrap());
    if psmodule.is_none() {
        return Ok(());
    }
    let hp = init_hyperscoop().unwrap();
    let psmodule_dir = hp.get_psmodule_path();
    let module_name: Value = serde_json::from_str(psmodule.as_ref().unwrap().as_str())?;
    let module_name = module_name.get("name").unwrap().as_str().unwrap();
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

fn rm_all_dir(path: PathBuf) -> Result<(), anyhow::Error> {
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
    let is_having_current = app_path.join("current").exists();
    if !is_having_current {
        bail!(
            "'{}' {}",
            app_name.red().bold(),
            "is not installed correctly".red().bold()
        );
    }
    let manifest = app_path.join("current").join("manifest.json");
    if !manifest.exists() {
        bail!(
            "'{}' {}",
            app_name.red().bold(),
            "is not installed correctly".red().bold()
        );
    }
    let contents = std::fs::read_to_string(manifest)?;
    let manifest: serde_json::Value = serde_json::from_str(&contents)?;
    let bin = manifest["bin"].as_str().unwrap_or("");
    if !bin.is_empty() {
        let bin_path = app_path.join("current").join(bin);
        log::info!("{}", bin_path.display());

        if !bin_path.exists() {
            bail!(
                "'{}' {}",
                app_name.red().bold(),
                "is not installed correctly".red().bold()
            );
        }
    }
    if bin.is_empty() {
        let default_array: Vec<Value> = Vec::new();
        let bin = manifest["bin"].as_array().unwrap_or(&default_array);
        if bin.is_empty() {
            return Ok(true);
        }
        for bin_item in bin {
            let bin_item = bin_item.as_str().unwrap_or("");
            let bin_path = app_path.join("current").join(bin_item);
            if !bin_path.exists() {
                bail!(
                    "'{}' {}",
                    app_name.red().bold(),
                    "is not installed correctly".red().bold()
                );
            }
        }
    }
    Ok(true)
}

fn rm_start_menu_shortcut() ->Result< (), anyhow::Error>  {

  Ok(())
}

fn rm_shim_file(shim_path: String, manifests : &Value, app_name: &str) ->Result<() , anyhow::Error> {
  let  app_name = app_name.to_lowercase()+".json";
    let shim_path = Path::new(shim_path.as_str());
  let      manifest = manifests.get("bins") ;
  if manifest.is_none() {
       bail!("Failed to find shim  on '{}' manifest ", app_name);
  }
  match  manifest.unwrap() {
    Value::String(s) => {
      let mut  s = s.clone() ;
      if  s.contains('\\') {
        let  split = s.split(r"\").collect::<Vec<&str>>();
        s= split.last().unwrap().to_string();
      }
      if  s.contains('/') {
        let  split = s.split(r"/").collect::<Vec<&str>>();
        s= split.last().unwrap().to_string();
      }

      let suffix = s.split(".").last().unwrap();
      let  prefix   = s.split(".") .next().unwrap();
      let shim_file = shim_path.join(s.clone()  );
      if shim_file.exists()  && suffix == "exe"{
        println!("Removing shim file {}", shim_file.display().to_string().dark_blue().bold());
        std::fs::remove_file(&shim_file)?;
        let shim = prefix.to_string()+".shim" ;
        let shim_file = shim_path.join(shim);
        if  !shim_file.exists() { return Ok(()) ; }
         println !("Removing shim file {}", shim_file.display() .to_string().dark_blue().bold());
        std::fs::remove_file(shim_file)?;
      }
      if   suffix == "bat"{
        if shim_file.exists()  {
          println!("Removing shim file {}", shim_file.display().to_string().dark_blue().bold());
          std::fs::remove_file(&shim_file)?;
        }
        let   cmd_str = prefix.to_string()+".cmd" ;
        let  shell_file =shim_path.join(prefix);
        let cmd_file = shim_path.join(cmd_str);
        if   shell_file.exists() {
          println!("Removing shim file {}", shell_file.display().to_string().dark_blue().bold());
          std::fs::remove_file(&shell_file)?;
        }
        if  cmd_file.exists() {
          println!("Removing shim file {}", cmd_file.display().to_string().dark_blue().bold());
          std::fs::remove_file(&cmd_file)?;
        }
      }

      if  shim_file.exists()  && suffix == "ps1"{
        println!("Removing shim file {}", shim_file.display().to_string().dark_blue().bold());

        let  cmd_str = prefix.to_string()+".cmd" ;
        let  shell_file =shim_path.join(prefix);
        let cmd_file = shim_path.join(cmd_str);
        if   shell_file.exists() {
          println!("Removing shim file {}", shell_file.display().to_string().dark_blue().bold());
          std::fs::remove_file(&shell_file)?;
        }
        if  cmd_file.exists() {
            println!("Removing shim file {}", cmd_file.display().to_string().dark_blue().bold());
          std::fs::remove_file(&cmd_file)?;
        }
      }
    }
    Value::Array(a) => {
      for item in a {

        let mut  s =  item.as_str().unwrap().to_string() ;
        if  s.contains('\\') {
          let  split = s.split(r"\").collect::<Vec<&str>>();
          s= split.last().unwrap().to_string();
        }
        if  s.contains('/') {
          let  split = s.split(r"/").collect::<Vec<&str>>();
          s= split.last().unwrap().to_string();
        }
        let  suffix = item.as_str().unwrap().split(".").last().unwrap();
        let prefix = item.as_str().unwrap().split(".").next().unwrap();
        if  suffix == "exe" {
          let  exe_file = shim_path.join( (prefix.to_string()+".exe" ).as_str() );
          if  exe_file.exists() {
            println!("Removing shim file {}", exe_file.display().to_string().dark_blue().bold());
             let  shim_str = prefix.to_string()+".shim" ;
            let  shim_file = shim_path.join(shim_str);
            if shim_file.exists() {
              println!("Removing shim file {}", shim_file.display().to_string().dark_blue().bold());
            }
          }
        }
        if  suffix == "bat" {

        }

        if suffix == "ps1" {

        }

      }
    }
    _ => {
      bail!("can't parser this bin object type ")
    }
  }

    for entry in std::fs::read_dir(shim_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

    }

    Ok(())
}
