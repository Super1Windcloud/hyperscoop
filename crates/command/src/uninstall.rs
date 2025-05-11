use crate::init_hyperscoop;
use crate::manifest::uninstall_manifest::UninstallManifest;
use anyhow::{bail, Context};
use crossterm::style::Stylize;
use std::path::Path;
mod env_set;
use env_set::*;
pub(crate) mod shim_and_shortcuts;
use crate::init_env::{
    get_apps_path, get_apps_path_global, get_persist_dir_path, get_persist_dir_path_global,
    get_psmodules_root_dir, get_psmodules_root_global_dir, get_shims_root_dir,
    get_shims_root_dir_global,
};
use crate::install::LifecycleScripts::{PostUninstall, PreUninstall, Uninstaller};
use crate::install::{parse_lifecycle_scripts, InstallOptions};
use crate::utils::system::{is_admin, request_admin};
use shim_and_shortcuts::*;

pub fn uninstall_app_with_purge(app_name: &str, global: bool) -> Result<(), anyhow::Error> {
    uninstall_app(app_name, global)?;
    println!(
        "{} '{}'",
        "Removing Persisted data for".to_string().dark_blue().bold(),
        app_name.dark_cyan().bold()
    );
    let persist_path = if global {
        get_persist_dir_path_global()
    } else {
        get_persist_dir_path()
    };
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
    std::fs::remove_dir_all(app_persist_path)
        .context("Failed to remove app persisted data at line 41")?;
    Ok(())
}

pub fn uninstall_app(app_name: &str, is_global: bool) -> Result<(), anyhow::Error> {
    if is_global && !is_admin()? {
        request_admin();
    }
    let app_path = if is_global {
        get_apps_path_global()
    } else {
        get_apps_path()
    };
    let shim_path = if is_global {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    if !Path::new(&app_path).exists() {
        bail!("{} is not existing", app_path);
    }
    if !Path::new(&shim_path).exists() {
        bail!("{} is not existing", shim_path);
    }
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
    let result = uninstall_matched_app(&app_path, app_name, &shim_path, is_global);
    if let Err(e) = result {
        eprintln!("{}", e);
        let app_path = Path::new(&app_path).join(app_name);
        if !app_path.exists() {
            eprintln!("{} is not exists", app_path.display());
            return Ok(());
        }
        let app_path = app_path.to_str().unwrap();
        println!(
            "{}",
            format!("Removing Error  Installation of '{app_name}' => {app_path}")
        );
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
    app_path: &str,
    app_name: &str,
    shim_path: &str,
    is_global: bool,
) -> Result<(), anyhow::Error> {
    for entry in std::fs::read_dir(app_path).context("Failed to read app path at line  135")? {
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
                let contents = std::fs::read_to_string(&manifest_path)
                    .context("Failed to read app current manifest.json at line 150")?;
                let mut manifest: UninstallManifest = serde_json::from_str(&contents)
                    .context("Failed to parse app current manifest.json at line 152")?;
                manifest.set_name(&app_name.to_string()); // 先进行可变借用

                let version = &manifest.version;
                if version.is_none() {
                    bail!("version is not existing")
                };
                let version = version.clone().unwrap();
                let install_info = std::fs::read_to_string(install_path)
                    .context("Failed to read app install.json at line 161")?;
                let install_info: serde_json::Value = serde_json::from_str(&install_info)
                    .context("Failed to parse app install.json at line 163")?;
                let arch = install_info["architecture"].as_str().unwrap_or("Unknown");
                let manifest_path = manifest_path.to_str().unwrap();
                let options = if is_global {
                    vec![InstallOptions::Global]
                } else {
                    vec![]
                };
                parse_lifecycle_scripts(
                    PreUninstall,
                    manifest_path,
                    &options,
                    app_name,
                    Some(arch),
                )
                .expect("Failed to run pre-uninstall lifecycle script");
                println!(
                    "{} '{}'  ({})",
                    "Uninstalling".to_string().dark_blue().bold(),
                    app_name.dark_red().bold(),
                    version.dark_red().bold()
                );
                parse_lifecycle_scripts(Uninstaller, manifest_path, &options, app_name, Some(arch))
                    .expect("Failed to run Uninstaller lifecycle script");
                parse_lifecycle_scripts(
                    PostUninstall,
                    manifest_path,
                    &options,
                    app_name,
                    Some(arch),
                )
                .expect("Failed to run PostUninstall lifecycle script");

                // invoke_hook_script(HookType::Uninstaller, &manifest, arch)?;
                // invoke_hook_script(HookType::PostUninstall, &manifest, arch)?;
                uninstall_psmodule(&manifest, is_global)?;

                env_path_var_rm(&current_path, &manifest, is_global)?;

                env_var_rm(&manifest, is_global)?;
                rm_shim_file(shim_path, &manifest, app_name)?;
                rm_start_menu_shortcut(&manifest, is_global)?;
                println!(
                    "{} {}",
                    "Unlinking".dark_blue().bold(),
                    &current_path.display().to_string().dark_green().bold()
                );
                rm_all_dir(path.clone())?;
                return Ok(());
            }
        }
    }
    Ok(())
}

fn uninstall_psmodule(manifest: &UninstallManifest, is_global: bool) -> Result<(), anyhow::Error> {
    let psmodule = manifest.clone().psmodule;
    if psmodule.is_none() {
        return Ok(());
    }
    let psmodule = psmodule.unwrap();
    let psmodule_dir = if is_global {
        get_psmodules_root_global_dir()
    } else {
        get_psmodules_root_dir()
    };
    let module_name = psmodule.name;
    println!(
        "Uninstalling PowerShell module  '{}'",
        module_name.clone().dark_red().bold()
    );
    let lind_path = Path::new(&psmodule_dir).join(module_name);
    if lind_path.exists() {
        println!("Removing psmodule path {}", &lind_path.display());
        std::fs::remove_dir_all(lind_path).context("Failed to remove psmodule path at line 235")?;
    }
    Ok(())
}

fn rm_all_dir<P: AsRef<Path>>(path: P) -> Result<(), anyhow::Error> {
    match std::fs::remove_dir_all(&path) {
        Ok(_) => Ok(()),
        Err(err) => {
            bail!(
                "remove dir '{}' error: {}",
                path.as_ref().display(),
                err.to_string().red().bold()
            );
        }
    }
}

fn check_installed_status(app_name: &str) -> Result<bool, anyhow::Error> {
    use regex::Regex;
    let pattern = r"[\[\]()*+?{}|^$#]";
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
    if !app_path.exists() {
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
            "don't have current dir,is not installed correctly"
                .red()
                .bold()
        );
    }
    let manifest = app_path.join("current").join("manifest.json");
    let install_json = app_path.join("current").join("install.json");
    if !install_json.exists() {
        bail!(
            "'{}' {}",
            app_name.red().bold(),
            "don't have install.json, is not installed correctly"
                .red()
                .bold()
        );
    }
    if !manifest.exists() {
        bail!(
            "'{}' {}",
            app_name.red().bold(),
            "don't have manifest.json, is not installed correctly"
                .red()
                .bold()
        );
    }

    Ok(true)
}
