use crate::command_args::purge::PurgeArgs;
use crate::i18n::{tr, tr_fmt};
use anyhow::Context;
use command_util_lib::init_env::{get_apps_path, get_apps_path_global};
use command_util_lib::utils::system::{kill_processes_using_app, read_dir_link};
use crossterm::style::Stylize;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

pub fn execute_purge_command(args: PurgeArgs) -> anyhow::Result<()> {
    let root = if args.global {
        get_apps_path_global()
    } else {
        get_apps_path()
    };
    purge_apps_root(Path::new(&root), args.app_names.as_deref(), args.dry_run)
}

fn purge_apps_root(
    apps_root: &Path,
    app_names: Option<&[String]>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let targets = app_targets(apps_root, app_names)?;
    let mut removed = 0;
    let mut errors = Vec::new();
    for app_dir in targets {
        let explicit = app_names.is_some();
        match purge_app(apps_root, &app_dir, explicit, dry_run) {
            Ok(count) => removed += count,
            Err(error) => errors.push(format!("{}: {error:#}", app_dir.display())),
        }
    }
    if removed == 0 && errors.is_empty() {
        println!(
            "{}",
            tr(
                "No leftover error directories found.",
                "未发现残留错误目录。"
            )
            .dark_green()
            .bold()
        );
    } else if removed > 0 {
        let summary = if dry_run {
            tr_fmt!(
                "Would remove {removed} leftover directories.",
                "将清理 {removed} 个残留目录。",
                removed = removed
            )
        } else {
            tr_fmt!(
                "Removed {removed} leftover directories.",
                "已清理 {removed} 个残留目录。",
                removed = removed
            )
        };
        println!("{}", summary.dark_green().bold());
    }
    if !errors.is_empty() {
        anyhow::bail!("{}", errors.join("\n"));
    }
    Ok(())
}

fn app_targets(apps_root: &Path, app_names: Option<&[String]>) -> anyhow::Result<Vec<PathBuf>> {
    let Some(names) = app_names else {
        let mut apps = Vec::new();
        for entry in fs::read_dir(apps_root).context("Failed to read apps root directory")? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                apps.push(entry.path());
            }
        }
        apps.sort();
        return Ok(apps);
    };

    let mut apps = Vec::new();
    for name in names {
        command_util_lib::info::validate_app_name(name)?;
        anyhow::ensure!(
            !name.contains('/') && !name.contains('@'),
            "Expected an installed app name"
        );
        let app_dir = apps_root.join(name);
        if fs::symlink_metadata(&app_dir).is_err() {
            anyhow::bail!(
                "{}",
                tr_fmt!("'{name}' is not installed", "'{name}' 未安装", name = name)
            );
        }
        apps.push(app_dir);
    }
    Ok(apps)
}

fn purge_app(
    apps_root: &Path,
    app_dir: &Path,
    explicit: bool,
    dry_run: bool,
) -> anyhow::Result<usize> {
    if is_scoop_app(app_dir) {
        if explicit {
            println!(
                "{}",
                tr(
                    "Skipping protected app 'scoop'.",
                    "已跳过受保护的应用 'scoop'。"
                )
                .dark_yellow()
                .bold()
            );
        }
        return Ok(0);
    }
    if is_directory_link(app_dir) {
        return Ok(0);
    }

    let plan = plan_app(app_dir)?;
    let app_name = app_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let mut removed = 0;
    for path in &plan.leftovers {
        if dry_run {
            println!(
                "{} {}",
                tr(
                    "[dry-run] Would remove leftover directory:",
                    "[dry-run] 将清理残留目录:"
                )
                .dark_yellow()
                .bold(),
                path.display()
            );
            removed += 1;
            continue;
        }
        remove_leftover(app_dir, path, app_name)?;
        println!(
            "{} {}",
            tr("Removing leftover directory:", "正在清理残留目录:")
                .dark_cyan()
                .bold(),
            path.display()
        );
        removed += 1;
    }

    if plan.remove_app_dir && (dry_run || dir_is_empty(app_dir)?) {
        if dry_run {
            println!(
                "{} {}",
                tr(
                    "[dry-run] Would remove empty app directory:",
                    "[dry-run] 将清理空的 APP 目录:"
                )
                .dark_yellow()
                .bold(),
                app_dir.display()
            );
        } else {
            remove_empty_app_dir(apps_root, app_dir)?;
            println!(
                "{} {}",
                tr("Removing empty app directory:", "正在清理空的 APP 目录:")
                    .dark_cyan()
                    .bold(),
                app_dir.display()
            );
        }
        removed += 1;
    }
    Ok(removed)
}

struct AppPurgePlan {
    leftovers: Vec<PathBuf>,
    remove_app_dir: bool,
}

fn plan_app(app_dir: &Path) -> anyhow::Result<AppPurgePlan> {
    let current_target = current_link_target(app_dir);
    let mut entry_count = 0;
    let mut leftovers = Vec::new();
    for entry in fs::read_dir(app_dir).context("Failed to read app directory")? {
        let entry = entry?;
        entry_count += 1;
        if let Some(path) = leftover_candidate(&entry, current_target.as_deref())? {
            leftovers.push(path);
        }
    }
    leftovers.sort();
    Ok(AppPurgePlan {
        remove_app_dir: entry_count == leftovers.len(),
        leftovers,
    })
}

/// A leftover is a real directory that is not `current`, not a link, not the
/// active version, and has no `install.json`. Installed versions stay for `hp cleanup`.
fn leftover_candidate(
    entry: &DirEntry,
    current_target: Option<&Path>,
) -> anyhow::Result<Option<PathBuf>> {
    if entry.file_name() == "current" {
        return Ok(None);
    }
    let file_type = entry.file_type()?;
    if !file_type.is_dir() || file_type.is_symlink() {
        return Ok(None);
    }
    let path = entry.path();
    if is_directory_link(&path) || path.join("install.json").is_file() {
        return Ok(None);
    }
    if current_target.is_some_and(|target| same_dir(&path, target)) {
        return Ok(None);
    }
    Ok(Some(path))
}

fn remove_leftover(app_dir: &Path, path: &Path, app_name: &str) -> anyhow::Result<()> {
    let root = fs::canonicalize(app_dir).context("Failed to resolve app directory")?;
    let meta = fs::symlink_metadata(path).context("Failed to inspect leftover directory")?;
    anyhow::ensure!(
        meta.file_type().is_dir() && !meta.file_type().is_symlink() && !is_directory_link(path),
        "Refusing to remove a path that is not a real directory: {}",
        path.display()
    );
    anyhow::ensure!(
        !path.join("install.json").is_file(),
        "Refusing to remove an installed version: {}",
        path.display()
    );
    if current_link_target(app_dir).is_some_and(|target| same_dir(path, &target)) {
        anyhow::bail!(
            "Refusing to remove the current version directory: {}",
            path.display()
        );
    }
    let canonical = fs::canonicalize(path).context("Failed to resolve leftover directory")?;
    anyhow::ensure!(
        canonical.parent() == Some(root.as_path()),
        "Refusing to remove a directory outside the app directory: {}",
        path.display()
    );
    if fs::remove_dir_all(path).is_err() {
        if !app_name.is_empty() {
            kill_processes_using_app(app_name);
        }
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to remove leftover directory {}", path.display()))?;
    }
    Ok(())
}

fn remove_empty_app_dir(apps_root: &Path, app_dir: &Path) -> anyhow::Result<()> {
    let root = fs::canonicalize(apps_root).context("Failed to resolve apps root")?;
    let meta = fs::symlink_metadata(app_dir).context("Failed to inspect app directory")?;
    anyhow::ensure!(
        meta.file_type().is_dir() && !meta.file_type().is_symlink() && !is_directory_link(app_dir),
        "Refusing to remove an app directory that is not a real directory: {}",
        app_dir.display()
    );
    anyhow::ensure!(
        dir_is_empty(app_dir)?,
        "Refusing to remove an app directory that is not empty: {}",
        app_dir.display()
    );
    let canonical = fs::canonicalize(app_dir).context("Failed to resolve app directory")?;
    anyhow::ensure!(
        canonical.parent() == Some(root.as_path()),
        "Refusing to remove an app directory outside the apps root: {}",
        app_dir.display()
    );
    fs::remove_dir(app_dir).context("Failed to remove empty app directory")?;
    Ok(())
}

fn current_link_target(app_dir: &Path) -> Option<PathBuf> {
    let target = read_dir_link(app_dir.join("current"))?;
    if target.is_absolute() {
        Some(target)
    } else {
        Some(app_dir.join(target))
    }
}

fn same_dir(path: &Path, target: &Path) -> bool {
    match (fs::canonicalize(path), fs::canonicalize(target)) {
        (Ok(left), Ok(right)) => left == right,
        _ => path == target,
    }
}

fn is_directory_link(path: &Path) -> bool {
    read_dir_link(path).is_some()
}

fn is_scoop_app(app_dir: &Path) -> bool {
    app_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("scoop"))
}

fn dir_is_empty(path: &Path) -> anyhow::Result<bool> {
    Ok(fs::read_dir(path)
        .context("Failed to read directory")?
        .next()
        .is_none())
}

#[cfg(test)]
mod tests {
    use super::*;
    use command_util_lib::utils::system::create_dir_link;

    fn installed_version(app_dir: &Path, version: &str) -> PathBuf {
        let dir = app_dir.join(version);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("manifest.json"),
            format!(r#"{{"version":"{version}"}}"#),
        )
        .unwrap();
        fs::write(dir.join("install.json"), "{}").unwrap();
        dir
    }

    #[test]
    fn purge_removes_stray_and_empty_app_dirs_but_keeps_installed_versions() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let apps = temp.path().join("apps");
        fs::create_dir(&apps)?;

        let dingtalk = apps.join("dingtalk");
        fs::create_dir(&dingtalk)?;
        let old = installed_version(&dingtalk, "1.0.0");
        let current_version = installed_version(&dingtalk, "7.6.10");
        create_dir_link(&current_version, dingtalk.join("current"))?;
        for name in ["PS", "plugin_upgrade"] {
            fs::create_dir(dingtalk.join(name))?;
        }
        fs::create_dir(dingtalk.join("incomplete"))?;
        fs::write(dingtalk.join("incomplete").join("data"), "junk")?;
        fs::write(dingtalk.join("notes.txt"), "keep")?;
        let mismatched = installed_version(&dingtalk, "2.0.0");
        fs::write(mismatched.join("manifest.json"), r#"{"version":"other"}"#)?;

        let external = temp.path().join("external");
        fs::create_dir(&external)?;
        fs::write(external.join("secret"), "keep")?;
        create_dir_link(&external, dingtalk.join("linked"))?;

        let scoop = apps.join("scoop");
        fs::create_dir(&scoop)?;
        fs::create_dir(scoop.join("stray"))?;
        fs::write(scoop.join("stray").join("data"), "keep")?;

        fs::create_dir(apps.join("grok"))?;

        let broken = apps.join("broken");
        fs::create_dir_all(broken.join("4.1.1"))?;
        fs::write(broken.join("4.1.1").join("partial-download"), "incomplete")?;

        let active = apps.join("active-junk");
        fs::create_dir_all(active.join("PS"))?;
        fs::write(active.join("PS").join("data"), "active")?;
        create_dir_link(active.join("PS"), active.join("current"))?;

        purge_apps_root(&apps, None, true)?;
        assert!(dingtalk.join("PS").is_dir());
        assert!(apps.join("grok").is_dir());
        assert!(broken.join("4.1.1").is_dir());

        purge_apps_root(&apps, None, false)?;

        assert!(!dingtalk.join("PS").exists());
        assert!(!dingtalk.join("plugin_upgrade").exists());
        assert!(!dingtalk.join("incomplete").exists());
        assert!(dingtalk.join("notes.txt").is_file());
        assert!(old.join("install.json").is_file());
        assert!(current_version.join("install.json").is_file());
        assert!(dingtalk.join("current").join("install.json").is_file());
        assert!(mismatched.join("install.json").is_file());
        assert!(external.join("secret").is_file());
        assert!(dingtalk.join("linked").join("secret").is_file());
        assert!(scoop.join("stray").join("data").is_file());
        assert!(!apps.join("grok").exists());
        assert!(!broken.exists());
        assert!(active.join("PS").join("data").is_file());
        assert!(active.join("current").join("data").is_file());
        Ok(())
    }

    #[test]
    fn purge_named_missing_app_is_an_error_and_does_not_touch_siblings() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let apps = temp.path().join("apps");
        let git = apps.join("git");
        installed_version(&git, "2.0");
        fs::create_dir(git.join("PS"))?;

        let error = purge_apps_root(&apps, Some(&["missing".to_string()]), false)
            .expect_err("missing app must fail");
        let message = format!("{error:#}");
        assert!(message.contains("missing"), "{message}");
        assert!(git.join("PS").is_dir());
        Ok(())
    }
}
