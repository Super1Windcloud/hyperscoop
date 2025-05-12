use crate::init_env::{get_app_dir, get_app_dir_global};
use crate::install::{create_shims_file, InstallOptions};
use crate::manifest::install_manifest::InstallManifest;
use crate::utils::system::get_system_default_arch;
use crate::utils::utility::compare_versions;
use anyhow::{bail, Context};
use crossterm::style::Stylize;
use std::cmp::Ordering;
use std::os::windows::fs::symlink_dir;
use std::path::{Path, PathBuf};

pub fn reset_latest_version(
    name: &str,
    global: bool,
    shim_reset: bool,
) -> Result<(), anyhow::Error> {
    let app_dir = if global {
        get_app_dir_global(&name)
    } else {
        get_app_dir(&name)
    };
    let child_dirs = std::fs::read_dir(&app_dir)
        .context("Failed to read app root dir at line 23")?
        .filter_map(|entry| {
            let file_type = entry.as_ref().unwrap().file_type().unwrap();
            let path = entry.as_ref().unwrap().path();
            if file_type.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>(); // 不包含 current
    let count = child_dirs.len();
    log::info!("Resetting app: {}", name);
    let app_dir = Path::new(&app_dir);
    let app_current_path = app_dir.join("current");
    if count < 1 {
        bail!("app 文件目录为空")
    } else if count == 1 {
        if app_current_path.exists() {
            std::fs::remove_dir(&app_current_path).context(format!(
                "remove old app dir {} at line 43",
                app_current_path.display()
            ))?;
        };
        let version_path = child_dirs.first().unwrap();
        let result = symlink_dir(version_path, app_current_path.as_path());
        if result.is_err() {
            std::fs::remove_dir(&app_current_path)
                .context("failed remove current dir at line 49")?;
            symlink_dir(&version_path.as_path(), app_current_path.as_path())
                .context("failed to create app symlink at line 51")?;
        }
        println!(
            "{} {} => {}",
            format!("Resetting '{name}' successfully")
                .dark_blue()
                .bold(),
            app_current_path.display().to_string().dark_green().bold(),
            version_path.display().to_string().dark_green().bold()
        );
    } else {
        if app_current_path.exists() {
            std::fs::remove_dir(app_current_path.as_path())
                .context("failed remove app current dir at line 64")?;
        }
        let mut max_version = String::new();
        let _ = child_dirs.iter().for_each(|version_path| {
            let version_name = version_path.file_name().unwrap().to_str().unwrap();
            match compare_versions(version_name.into(), max_version.clone()) {
                Ordering::Less => {}
                Ordering::Equal => {}
                Ordering::Greater => max_version = version_name.to_string(),
            }
        });
        let max_version_path = app_dir.join(&max_version);
        log::info!("Resetting app: {}", max_version_path.display());
        symlink_dir(max_version_path, app_current_path.as_path())
            .context("Failed to create app symlink for reset at line 80")?;
        println!(
            "{}",
            format!("Resetting {}@{} successfully!", name, &max_version)
                .dark_green()
                .bold()
        );
    }
    if shim_reset {
        reset_shim_file(name, app_current_path, global)?;
    }
    Ok(())
}

fn reset_shim_file(app_name: &str, app_current_path: PathBuf, global: bool) -> anyhow::Result<()> {
    let manifest_path = app_current_path.join("manifest.json");
    if !manifest_path.exists() {
        bail!(format!(
            "manifest.json not found in {} dir",
            app_current_path.display()
        ));
    }
    let manifest_json = std::fs::read_to_string(manifest_path)
        .context("Failed to read manifest.json at line 101")?;
    let manifest: InstallManifest = serde_json::from_str(&manifest_json)
        .context("Failed to parse manifest.json at line 103")?;
    let bin = manifest.bin;
    let architecture = manifest.architecture;
    let arch = get_system_default_arch()?;
    let options: Box<[InstallOptions]> = if global {
        vec![InstallOptions::Global].into_boxed_slice()
    } else {
        vec![].into_boxed_slice()
    };

    if bin.is_some() {
        create_shims_file(bin.unwrap(), app_name, &options)?;
    } else if architecture.is_some() {
        let architecture = architecture.unwrap();
        let base_arch = architecture.get_specific_architecture(arch.as_str());
        if base_arch.is_some() {
            let bin = base_arch.unwrap().clone().bin;
            if bin.is_some() {
                create_shims_file(bin.unwrap(), app_name, &options)?;
            }
        }
    }
    Ok(())
}

pub fn reset_specific_version(
    name: &str,
    version: &str,
    global: bool,
    shim_reset: bool,
) -> Result<(), anyhow::Error> {
    log::info!("Resetting app: {}@{}", name, version);
    let app_dir = if global {
        get_app_dir_global(&name)
    } else {
        get_app_dir(&name)
    };
    let version_path = Path::new(&app_dir).join(version);
    if !version_path.exists() {
        bail!("special version not found in {app_dir} dir")
    }

    let app_dir = Path::new(&app_dir);
    let app_current_path = app_dir.join("current");

    if app_current_path.exists() {
        std::fs::remove_dir(&app_current_path).context(format!(
            "Failed remove app current dir {} at line 150",
            app_current_path.display()
        ))?;
    };
    let result = symlink_dir(&version_path, app_current_path.as_path());
    if result.is_err() {
        std::fs::remove_dir(&app_current_path)
            .context("failed remove app current dir at line 159")?;
        symlink_dir(version_path.as_path(), app_current_path.as_path()).context(format!(
            "Failed to create app symlink {} at line 161",
            version_path.display()
        ))?;
    }
    println!(
        "{}",
        format!("Resetting {}@{} successfully!", name, version)
            .dark_green()
            .bold()
    );
    if shim_reset {
        reset_shim_file(name, app_current_path, global)?;
    }
    Ok(())
}

mod test_reset {
    #[test]
    fn test_reset_latest() {
        use crate::reset::reset_latest_version;

        let name = "7zip-zs";
        let global = false;
        reset_latest_version(name, global, false).unwrap();
    }
}
