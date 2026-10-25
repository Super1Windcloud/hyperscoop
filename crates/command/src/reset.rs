use crate::i18n::tr;
use crate::init_env::{get_app_dir, get_app_dir_global};
use crate::install::{InstallOptions, create_shims_file};
use crate::manifest::install_manifest::InstallManifest;
use crate::tr_fmt;
use crate::utils::system::get_system_default_arch;
use crate::utils::utility::compare_versions;
use anyhow::{Context, bail};
use crossterm::style::Stylize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

pub fn reset_latest_version(
    name: &str,
    global: bool,
    shim_reset: bool,
) -> Result<(), anyhow::Error> {
    let _lock = crate::utils::lock::HpProcessLock::acquire(global, "reset")?;
    let app_dir = if global {
        get_app_dir_global(&name)
    } else {
        get_app_dir(&name)
    };
    reset_latest_in_dir(name, Path::new(&app_dir), global, shim_reset)
}

fn reset_latest_in_dir(
    name: &str,
    app_dir: &Path,
    global: bool,
    shim_reset: bool,
) -> anyhow::Result<()> {
    let mut child_dirs = Vec::new();
    for entry in std::fs::read_dir(app_dir).context("Failed to read app root dir")? {
        let entry = entry.context("Failed to read app version entry")?;
        let path = entry.path();
        if entry.file_name() != "current"
            && entry.file_type()?.is_dir()
            && path.join("manifest.json").is_file()
        {
            child_dirs.push(path);
        }
    }
    let count = child_dirs.len();
    log::info!("Resetting app: {}", name);
    let app_current_path = app_dir.join("current");
    if count < 1 {
        bail!("{}", tr("App directory is empty", "app 文件目录为空"))
    } else if count == 1 {
        if std::fs::symlink_metadata(&app_current_path).is_ok() {
            std::fs::remove_dir_all(&app_current_path)
                .context(format!("remove old app dir {}", app_current_path.display()))?;
        };
        let version_path = child_dirs.first().unwrap();
        crate::utils::system::create_dir_link(version_path, app_current_path.as_path())
            .context("failed to create app link")?;
        println!(
            "{} {} => {}",
            tr_fmt!(
                "Resetting '{name}' successfully!",
                "重置 '{name}' 成功！",
                name = name
            )
            .dark_blue()
            .bold(),
            app_current_path.display().to_string().dark_green().bold(),
            version_path.display().to_string().dark_green().bold()
        );
    } else {
        let mut max_version = child_dirs[0]
            .file_name()
            .and_then(|name| name.to_str())
            .context("Invalid app version directory name")?
            .to_string();
        for version_path in child_dirs.iter().skip(1) {
            let version_name = version_path
                .file_name()
                .and_then(|name| name.to_str())
                .context("Invalid app version directory name")?;
            match compare_versions(version_name.into(), max_version.clone()) {
                Ordering::Less => {}
                Ordering::Equal => {}
                Ordering::Greater => max_version = version_name.to_string(),
            }
        }
        let max_version_path = app_dir.join(&max_version);
        if std::fs::symlink_metadata(&app_current_path).is_ok() {
            std::fs::remove_dir_all(app_current_path.as_path())
                .context("failed remove app current dir")?;
        }
        log::info!("Resetting app: {}", max_version_path.display());
        crate::utils::system::create_dir_link(&max_version_path, app_current_path.as_path())
            .context("Failed to create app link for reset")?;
        println!(
            "{}",
            tr_fmt!(
                "Resetting {name}@{version} successfully!",
                "重置 {name}@{version} 成功！",
                name = name,
                version = max_version
            )
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
        bail!(tr_fmt!(
            "manifest.json not found in {dir} directory",
            "目录 {dir} 中未找到 manifest.json",
            dir = app_current_path.display()
        ));
    }
    let manifest_json =
        std::fs::read_to_string(manifest_path).context("Failed to read manifest.json")?;
    let manifest: InstallManifest =
        serde_json::from_str(&manifest_json).context("Failed to parse manifest.json")?;
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
    let _lock = crate::utils::lock::HpProcessLock::acquire(global, "reset")?;
    log::info!("Resetting app: {}@{}", name, version);
    let app_dir = if global {
        get_app_dir_global(&name)
    } else {
        get_app_dir(&name)
    };
    reset_specific_in_dir(name, version, Path::new(&app_dir), global, shim_reset)
}

fn reset_specific_in_dir(
    name: &str,
    version: &str,
    app_dir: &Path,
    global: bool,
    shim_reset: bool,
) -> anyhow::Result<()> {
    let version_path = app_dir.join(version);
    if !version_path.exists() {
        bail!(tr_fmt!(
            "Specified version '{version}' not found in {app_dir}",
            "在 {app_dir} 中未找到指定的版本 '{version}'",
            version = version,
            app_dir = app_dir.display()
        ));
    }
    if !version_path.is_dir() || !version_path.join("manifest.json").is_file() {
        bail!("Incomplete installed version: {}", version_path.display());
    }

    let app_current_path = app_dir.join("current");

    if std::fs::symlink_metadata(&app_current_path).is_ok() {
        std::fs::remove_dir_all(&app_current_path).context(format!(
            "Failed remove app current dir {}",
            app_current_path.display()
        ))?;
    };
    crate::utils::system::create_dir_link(&version_path, app_current_path.as_path()).context(
        format!("Failed to create app link {}", version_path.display()),
    )?;
    println!(
        "{}",
        tr_fmt!(
            "Resetting {name}@{version} successfully!",
            "重置 {name}@{version} 成功！",
            name = name,
            version = version
        )
        .dark_green()
        .bold()
    );
    if shim_reset {
        reset_shim_file(name, app_current_path, global)?;
    }
    Ok(())
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn reset_selects_a_version_when_all_versions_compare_equal() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        for version in ["0.0", "0.0.0"] {
            let dir = temp.path().join(version);
            std::fs::create_dir(&dir)?;
            std::fs::write(dir.join("manifest.json"), "{}")?;
        }
        reset_latest_in_dir("test-app", temp.path(), false, false)?;
        assert!(temp.path().join("current").join("manifest.json").is_file());
        assert_ne!(
            std::fs::canonicalize(temp.path().join("current"))?,
            std::fs::canonicalize(temp.path())?
        );
        Ok(())
    }

    #[test]
    fn reset_specific_rejects_incomplete_version_before_changing_current() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let installed = temp.path().join("1.0");
        std::fs::create_dir(&installed)?;
        std::fs::write(installed.join("manifest.json"), "{}")?;
        std::fs::create_dir(temp.path().join("2.0"))?;
        let current = temp.path().join("current");
        crate::utils::system::create_dir_link(&installed, &current)?;

        assert!(reset_specific_in_dir("test-app", "2.0", temp.path(), false, false).is_err());
        assert_eq!(
            std::fs::canonicalize(&current)?,
            std::fs::canonicalize(&installed)?
        );
        Ok(())
    }

    #[test]
    fn reset_replaces_valid_and_broken_current_links_and_skips_incomplete_versions()
    -> anyhow::Result<()> {
        for broken in [false, true] {
            let temp = tempfile::tempdir()?;
            let app_dir = temp.path();
            let installed = app_dir.join("1.0");
            std::fs::create_dir(&installed)?;
            std::fs::write(installed.join("manifest.json"), "{}")?;
            std::fs::create_dir(app_dir.join("99.0"))?;
            let current = app_dir.join("current");
            let old_target = app_dir.join("old-target");
            std::fs::create_dir(&old_target)?;
            crate::utils::system::create_dir_link(&old_target, &current)?;
            if broken {
                std::fs::remove_dir(&old_target)?;
            }

            reset_latest_in_dir("test-app", app_dir, false, false)?;

            assert!(current.join("manifest.json").is_file());
            assert_eq!(
                std::fs::canonicalize(&current)?,
                std::fs::canonicalize(&installed)?
            );
            assert!(installed.join("manifest.json").is_file());
        }
        Ok(())
    }

    #[test]
    fn reset_keeps_current_when_no_complete_version_exists() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let app_dir = temp.path();
        let current = app_dir.join("current");
        std::fs::create_dir(&current)?;
        std::fs::write(current.join("manifest.json"), "{}")?;
        std::fs::create_dir(app_dir.join("2.0"))?;

        assert!(reset_latest_in_dir("test-app", app_dir, false, false).is_err());
        assert!(current.join("manifest.json").is_file());
        Ok(())
    }
}

mod test_reset {

    #[test]
    fn test_reset_latest() {
        use crate::reset::reset_latest_version;

        let name = "7zip-zs";
        let global = false;
        reset_latest_version(name, global, false).unwrap();
    }

    #[test]
    fn test_remove_dir() {
        use std::path::Path;
        let _current1 = Path::new(r"A:\Scoop\apps\gping\current");
        let current2 = Path::new(r"A:\Scoop\apps\gh\current");

        if current2.exists() {
            std::fs::remove_dir_all(current2).unwrap()
        }
    }
}
