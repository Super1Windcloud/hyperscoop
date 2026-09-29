use crate::i18n::tr;
use crate::init_hyperscoop;
use crate::manifest::uninstall_manifest::UninstallManifest;
use crate::tr_fmt;
use anyhow::{Context, bail};
use crossterm::style::Stylize;
use std::path::Path;
mod env_set;
use env_set::*;
pub(crate) mod shim_and_shortcuts;
use crate::init_env::{
    get_app_dir, get_app_dir_global, get_apps_path, get_apps_path_global, get_persist_dir_path,
    get_persist_dir_path_global, get_psmodules_root_dir, get_psmodules_root_global_dir,
    get_shims_root_dir, get_shims_root_dir_global,
};
use crate::install::LifecycleScripts::{PostUninstall, PreUninstall, Uninstaller};
use crate::install::{InstallOptions, parse_lifecycle_scripts};
use crate::utils::system::kill_processes_using_app;
use shim_and_shortcuts::*;

pub fn uninstall_app_with_purge(app_name: &str, global: bool) -> Result<(), anyhow::Error> {
    if uninstall_app(app_name, global).is_err() {
        let app_dir = if global {
            get_app_dir_global(&app_name)
        } else {
            get_app_dir(&app_name)
        };
        let app_dir = Path::new(&app_dir);
        if app_dir.exists() {
            if std::fs::remove_dir_all(app_dir)
                .context(format!(
                    "Failed to remove app directory {}",
                    app_dir.display()
                ))
                .is_err()
            {
                kill_processes_using_app(&app_name);
                std::fs::remove_dir_all(app_dir)
                    .context(format!("Failed to remove app dir  {}", app_dir.display()))?;
            }
        } else {
            bail!(
                "{}",
                tr_fmt!(
                    "'{app_name}' is not installed",
                    "'{app_name}' 并没有安装",
                    app_name = app_name
                )
            );
        }
    }
    println!(
        "{}",
        tr_fmt!(
            "Removing persisted data for '{app_name}'",
            "正在删除 '{app_name}' 的持久化数据",
            app_name = app_name.dark_cyan().bold()
        )
        .dark_blue()
        .bold()
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
            tr("Persisted data does not exist:", "持久化数据不存在:")
                .dark_red()
                .bold(),
            app_persist_path.to_str().unwrap().dark_green().bold()
        );
        return Ok(());
    }
    std::fs::remove_dir_all(app_persist_path).context("Failed to remove app persisted data")?;
    Ok(())
}

pub fn uninstall_app(app_name: &str, is_global: bool) -> Result<(), anyhow::Error> {
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
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(uninstall_script)
            .output()?;
        if !output.status.success() {
            bail!(
                "{}",
                tr("Scoop uninstall script failed", "Scoop 卸载脚本执行失败")
            );
        }
        println!(
            "{}",
            tr(
                "Scoop uninstall script completed successfully",
                "Scoop 卸载脚本执行成功"
            )
        );
        std::process::exit(0);
    }
    if let Err(e) = check_installed_status(app_name) {
        eprintln!("{}", e);
        bail!(
            "{}",
            tr_fmt!(
                "Checked installed status failed: {e}",
                "检查安装状态失败: {e}",
                e = e
            )
        );
    }
    let result = uninstall_matched_app(&app_path, app_name, &shim_path, is_global);
    if let Err(e) = result {
        eprintln!("{}", e);
        let app_path = Path::new(&app_path).join(app_name);
        if !app_path.exists() {
            eprintln!(
                "{}",
                tr_fmt!(
                    "{path} does not exist",
                    "{path} 不存在",
                    path = app_path.display()
                )
            );
            return Ok(());
        }
        let app_path = app_path.to_str().unwrap();
        println!(
            "{}",
            tr_fmt!(
                "Removing failed installation of '{app_name}' => {app_path}",
                "正在清理 '{app_name}' 的异常安装目录 => {app_path}",
                app_name = app_name,
                app_path = app_path
            )
        );
        rm_all_dir(app_path)?;
        bail!(
            "{}",
            tr_fmt!(
                "'{app_name}' was not uninstalled as expected",
                "'{app_name}' 未按预期卸载完成",
                app_name = app_name.dark_cyan().bold()
            )
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
    for entry in std::fs::read_dir(app_path).context("Failed to read app path ")? {
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
                    .context("Failed to read app current manifest.json")?;
                let mut manifest: UninstallManifest = serde_json::from_str(&contents)
                    .context("Failed to parse app current manifest.json")?;
                manifest.set_name(&app_name.to_string()); // 先进行可变借用

                let version = &manifest.version;
                if version.is_none() {
                    bail!("version is not existing")
                };
                let version = version.clone().unwrap();
                let install_info = std::fs::read_to_string(install_path)
                    .context("Failed to read app install.json")?;
                let install_info: serde_json::Value = serde_json::from_str(&install_info)
                    .context("Failed to parse app install.json")?;
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
                .context("Failed to run pre-uninstall lifecycle script")?;
                println!(
                    "{} '{}'  ({})",
                    tr("Uninstalling", "正在卸载").dark_blue().bold(),
                    app_name.dark_red().bold(),
                    version.dark_red().bold()
                );
                parse_lifecycle_scripts(Uninstaller, manifest_path, &options, app_name, Some(arch))
                    .context("Failed to run Uninstaller lifecycle script")?;
                parse_lifecycle_scripts(
                    PostUninstall,
                    manifest_path,
                    &options,
                    app_name,
                    Some(arch),
                )
                .context("Failed to run PostUninstall lifecycle script")?;

                // invoke_hook_script(HookType::Uninstaller, &manifest, arch)?;
                // invoke_hook_script(HookType::PostUninstall, &manifest, arch)?;
                uninstall_psmodule(&manifest, is_global)?;

                env_path_var_rm(&current_path, &manifest, is_global)?;

                env_var_rm(&manifest, is_global)?;
                rm_shim_file(shim_path, &manifest, app_name)?;
                rm_start_menu_shortcut(&manifest, is_global)?;
                println!(
                    "{} {}",
                    tr("Unlinking", "正在取消链接").dark_blue().bold(),
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
        "{} '{}'",
        tr("Uninstalling PowerShell module", "正在卸载 PowerShell 模块")
            .dark_blue()
            .bold(),
        module_name.clone().dark_red().bold()
    );
    let lind_path = Path::new(&psmodule_dir).join(module_name);
    if lind_path.exists() {
        println!(
            "{} {}",
            tr(
                "Removing PowerShell module path",
                "正在删除 PowerShell 模块路径"
            )
            .dark_blue()
            .bold(),
            &lind_path.display()
        );
        std::fs::remove_dir_all(lind_path).context("Failed to remove psmodule path")?;
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
