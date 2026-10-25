use crate::command_args::cleanup::CleanupArgs;
use anyhow::Context;
use command_util_lib::init_env::{
    get_app_dir, get_app_dir_global, get_apps_path, get_apps_path_global,
};
use command_util_lib::list::VersionJSON;
use command_util_lib::utils::system::kill_processes_using_app;
use command_util_lib::utils::utility::compare_versions;
use crossterm::style::Stylize;
use std::path::{Path, PathBuf};

pub fn execute_cleanup_command(args: CleanupArgs) -> anyhow::Result<()> {
    if args.all {
        let root = if args.global {
            get_apps_path_global()
        } else {
            get_apps_path()
        };
        for entry in std::fs::read_dir(&root).context("Failed to read apps root directory")? {
            let entry = entry?;
            if entry.file_name() != "scoop" && entry.file_type()?.is_dir() {
                cleanup_app_versions(&entry.path(), args.dry_run)?;
            }
        }
    } else if let Some(names) = args.app_names {
        for name in names {
            command_util_lib::info::validate_app_name(&name)?;
            anyhow::ensure!(
                !name.contains('/') && !name.contains('@'),
                "Expected an installed app name"
            );
            let dir = if args.global {
                get_app_dir_global(&name)
            } else {
                get_app_dir(&name)
            };
            cleanup_app_versions(Path::new(&dir), args.dry_run)?;
        }
    }
    Ok(())
}

fn old_version_directories(app_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut installed = Vec::new();
    for entry in std::fs::read_dir(app_dir).context("Failed to read app directory")? {
        let entry = entry?;
        if entry.file_name() == "current" || !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        if !path.join("install.json").is_file() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path.join("manifest.json")) else {
            continue;
        };
        let Ok(manifest) = serde_json::from_str::<VersionJSON>(&content) else {
            continue;
        };
        let Some(version) = manifest.version else {
            continue;
        };
        if entry.file_name().to_str() == Some(version.as_str()) {
            installed.push((version, path));
        }
    }
    let Some((_, newest)) = installed
        .iter()
        .max_by(|(a, _), (b, _)| compare_versions(a.clone(), b.clone()))
    else {
        return Ok(Vec::new());
    };
    let current = app_dir.join("current");
    let current_target = match std::fs::symlink_metadata(&current) {
        Ok(_) => {
            Some(std::fs::canonicalize(&current).context("Failed to resolve current app version")?)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    let mut old = Vec::new();
    for (_, path) in &installed {
        if path != newest && Some(std::fs::canonicalize(path)?) != current_target {
            old.push(path.clone());
        }
    }
    Ok(old)
}

fn cleanup_app_versions(app_dir: &Path, dry_run: bool) -> anyhow::Result<()> {
    let root = std::fs::canonicalize(app_dir).context("Failed to resolve app directory")?;
    let app_name = app_dir
        .file_name()
        .and_then(|name| name.to_str())
        .context("Invalid app name")?;
    for path in old_version_directories(app_dir)? {
        if dry_run {
            println!(
                "{} {}",
                "[dry-run] Would remove old version:".dark_yellow().bold(),
                path.display()
            );
            continue;
        }
        // Recheck the selected directory before deleting; links must never be followed.
        anyhow::ensure!(
            std::fs::symlink_metadata(&path)?.file_type().is_dir()
                && std::fs::canonicalize(&path)?.parent() == Some(root.as_path()),
            "Refusing to clean a version outside the app directory: {}",
            path.display()
        );
        if std::fs::remove_dir_all(&path).is_err() {
            kill_processes_using_app(app_name);
            std::fs::remove_dir_all(&path).context("Failed to remove old version")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    fn installed_version(root: &Path, version: &str) -> anyhow::Result<PathBuf> {
        let dir = root.join(version);
        std::fs::create_dir(&dir)?;
        std::fs::write(
            dir.join("manifest.json"),
            format!(r#"{{"version":"{version}"}}"#),
        )?;
        std::fs::write(dir.join("install.json"), "{}")?;
        Ok(dir)
    }

    #[test]
    fn cleanup_preserves_active_version_latest_and_non_version_data() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().join("dingtalk");
        std::fs::create_dir(&root)?;
        let old = installed_version(&root, "1.0")?;
        let active = installed_version(&root, "2.0")?;
        let newest = installed_version(&root, "10.0")?;
        for name in ["PS", "plugin_upgrade", "incomplete"] {
            std::fs::create_dir(root.join(name))?;
            std::fs::write(root.join(name).join("data"), "keep")?;
        }
        std::fs::write(root.join("notes.txt"), "keep")?;
        command_util_lib::utils::system::create_dir_link(&active, root.join("current"))?;

        assert_eq!(old_version_directories(&root)?, vec![old.clone()]);
        cleanup_app_versions(&root, true)?;
        assert!(old.exists(), "dry run must preserve all versions");
        cleanup_app_versions(&root, false)?;
        assert!(!old.exists());
        assert!(active.join("manifest.json").is_file());
        assert!(newest.join("manifest.json").is_file());
        assert!(root.join("current/manifest.json").is_file());
        for name in ["PS", "plugin_upgrade", "incomplete"] {
            assert!(root.join(name).join("data").is_file());
        }
        assert!(root.join("notes.txt").is_file());
        Ok(())
    }

    #[test]
    fn cleanup_skips_directory_links_and_invalid_metadata() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().join("app");
        std::fs::create_dir(&root)?;
        let latest = installed_version(&root, "10.0")?;
        let external = installed_version(temp.path(), "1.0")?;
        command_util_lib::utils::system::create_dir_link(&external, root.join("1.0"))?;
        let mismatched = installed_version(&root, "2.0")?;
        std::fs::write(
            mismatched.join("manifest.json"),
            r#"{"version":"another-version"}"#,
        )?;
        let incomplete = installed_version(&root, "3.0")?;
        std::fs::remove_file(incomplete.join("install.json"))?;
        cleanup_app_versions(&root, false)?;
        assert!(external.join("manifest.json").is_file());
        assert!(root.join("1.0/manifest.json").is_file());
        assert!(mismatched.exists());
        assert!(incomplete.exists());
        assert!(latest.exists());
        Ok(())
    }
}

mod tests {

    #[test]
    #[cfg(windows)]
    fn test_file_stem() {
        use std::path::Path;
        let path =
            Path::new("C:\\Users\\Administrator\\scoop\\apps\\rustup\\current\\bin\\rustup.exe");
        let app_name = path.file_stem().unwrap().to_str().unwrap();
        let bin_name = path.file_name().unwrap().to_str().unwrap();
        assert_eq!(bin_name, "rustup.exe");
        assert_eq!(app_name, "rustup");
        let dir = Path::new("C:\\Users\\Administrator\\scoop\\apps\\rustup\\current");
        let app_name = dir.file_stem().unwrap().to_str().unwrap();
        let file_name = dir.file_name().unwrap().to_str().unwrap();
        assert_eq!(file_name, "current");
        assert_eq!(app_name, "current");
    }
}
