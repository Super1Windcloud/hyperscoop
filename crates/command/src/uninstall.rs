use crate::init_hyperscoop;
use anyhow::bail;
use crossterm::style::Stylize;
use std::path::Path;
use serde_json::Value;

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
          "{} {}", "persisted data is not  having".dark_red().bold(),
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
  let persist_path = hyperscoop.get_persist_path();
  let  lower = app_name.to_lowercase();
  let  app_name = lower.as_str();
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

                if !manifest_path.exists() {
                    bail!("{} is not  existing ", manifest_path.display());
                }
                let contents = std::fs::read_to_string(manifest_path)?;
                let manifest: serde_json::Value = serde_json::from_str(&contents)?;
                let version = manifest["version"].as_str().unwrap_or("Unknown");


                println!(
                    "Uninstalling '{}'  ({})",
                    app_name.dark_red().bold(),
                    version.dark_red().bold()
                );

              rm_shim_file(shim_path.clone(), app_name );
              rm_start_menu_shortcut();
                println!("Unlinking {}", current_path.display());
               // std::fs::remove_dir_all(path)?;

                return Ok(());
            }
        }
    }
    bail!("'{}' {}", app_name.red().bold() ,"is not installed".red().bold());
}

 
fn check_installed_status(app_name : &str) -> Result<bool, anyhow::Error> {

  use regex::Regex;
    let pattern = r"[\[\]\(\)\*\+\?\{\}\|\^\$\#]";
  let re = Regex::new(pattern)?;
 if re.is_match(app_name) {
    bail!("'{}' {}", app_name.red().bold(),"is not valid app name".red().bold());
 }
   let  app_path = init_hyperscoop()?.get_apps_path();
  let app_path = Path::new(&app_path).join(app_name);
  let  is_having_current = app_path.join("current").exists();
  if  !is_having_current {
    bail!("'{}' {}", app_name.red().bold(),"is not installed correctly".red().bold());
  }
  let   manifest = app_path.join("current").join("manifest.json");
  if  !manifest.exists() {
    bail!("'{}' {}", app_name.red().bold(),"is not installed correctly".red().bold());
  }
  let contents = std::fs::read_to_string(manifest)?;
  let manifest: serde_json::Value = serde_json::from_str(&contents)?;
  let bin  = manifest["bin"].as_str().unwrap_or("");
  if  !bin.is_empty() {
    let  bin_path = app_path.join("current").join(bin);
    log::info!("{}", bin_path.display());

    if  !bin_path.exists() {
      bail!("'{}' {}", app_name.red().bold(),"is not installed correctly".red().bold());
    }
  }
  if bin.is_empty() {
    let default_array :Vec<Value> =Vec::new();
   let bin = manifest["bin"].as_array().unwrap_or(&default_array);
    if bin.is_empty() {
      return Ok(true);
     }
    for  bin_item in bin {
      let bin_item = bin_item.as_str().unwrap_or("");
      let  bin_path = app_path.join("current").join(bin_item);
      if  !bin_path.exists() {
        bail!("'{}' {}", app_name.red().bold(),"is not installed correctly".red().bold());
      }
    }
  }
  Ok(true)
}

fn rm_start_menu_shortcut() {
}

fn rm_shim_file(shim_path: String, app_name: &str) {
  let  shim_path =  Path::new(shim_path.as_str());
  for entry in std::fs::read_dir(shim_path).unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();
    if ! path.is_file(){ continue;   }
    let file_name = path.file_name().unwrap().to_str().unwrap();
    if file_name.starts_with(app_name) {
      println!("Removing shim file {}", file_name.dark_red().bold());
    }
  }

}
