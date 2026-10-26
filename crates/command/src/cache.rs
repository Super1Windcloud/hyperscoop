pub(crate) mod download;

use crate::i18n::tr;
use crate::init_env::{get_cache_dir_path, get_cache_dir_path_global};
use anyhow::Context;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

fn cache_root(is_global: bool) -> PathBuf {
    PathBuf::from(if is_global {
        get_cache_dir_path_global()
    } else {
        get_cache_dir_path()
    })
}

// Group completed packages and their resume/control files into one cleanup unit.
fn package_name(name: &str) -> Option<&str> {
    let mut base = name.strip_suffix(".aria2").unwrap_or(name);
    base = base.strip_suffix(".incomplete").unwrap_or(base);
    if let Some(part) = base.strip_suffix(".part") {
        let (prefix, random) = part.rsplit_once('.')?;
        if !random.is_empty() && random.bytes().all(|byte| byte.is_ascii_digit()) {
            base = prefix;
        }
    }
    let mut parts = base.split('#');
    let app = parts.next()?;
    let version = parts.next()?;
    let hash = parts.next()?;
    if app.is_empty()
        || version.is_empty()
        || parts.next().is_some()
        || hash.len() < 7
        || !hash.as_bytes()[..7].iter().all(u8::is_ascii_hexdigit)
        || (hash.len() > 7 && hash.as_bytes()[7] != b'.')
    {
        return None;
    }
    Some(base)
}

pub fn display_all_cache_info(is_global: bool) -> anyhow::Result<()> {
    let root = cache_root(is_global);
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&root).context("Failed to read cache directory")? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if package_name(name) != Some(name) || !download::is_complete(&entry.path()) {
            continue;
        }
        let mut parts = name.split('#');
        entries.push((
            parts.next().unwrap().to_string(),
            parts.next().unwrap().to_string(),
            entry.metadata()?.len(),
        ));
    }
    entries.sort();
    let total: u64 = entries.iter().map(|entry| entry.2).sum();
    println!(
        "{}: {} {} ({:.2} MB)",
        tr("Total", "总计"),
        entries.len(),
        tr("files", "个文件"),
        total as f64 / 1_048_576.0
    );
    println!(
        "{:<25} {:<20} {}",
        tr("App", "应用"),
        tr("Version", "版本"),
        tr("Size (MB)", "大小 (MB)")
    );
    for (app, version, bytes) in entries {
        println!("{app:<25} {version:<20} {:.2}", bytes as f64 / 1_048_576.0);
    }
    Ok(())
}

#[derive(Debug, Default)]
struct CleanupResult {
    files: usize,
    bytes: u64,
}

fn clean_directory(
    root: &Path,
    apps: &[String],
    age: Option<Duration>,
    dry_run: bool,
    protected: &[String],
    now: SystemTime,
) -> anyhow::Result<CleanupResult> {
    let mut packages: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for entry in std::fs::read_dir(root).context("Failed to read cache directory")? {
        let entry = entry?;
        // Never traverse directories, junctions or symbolic links in a custom cache directory.
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(package) = package_name(name) else {
            continue;
        };
        let app = package.split('#').next().unwrap();
        if !apps.is_empty() && !apps.iter().any(|name| name.eq_ignore_ascii_case(app)) {
            continue;
        }
        if protected.iter().any(|path| {
            Path::new(path)
                .file_name()
                .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case(package))
        }) {
            continue;
        }
        packages
            .entry(package.to_string())
            .or_default()
            .push((entry.path(), entry.metadata()?));
    }
    let mut result = CleanupResult::default();
    for files in packages.values() {
        // A fresh resume file protects the entire package, including an old preallocated file.
        if let Some(age) = age {
            let mut expired = true;
            for (_, metadata) in files {
                if !now
                    .duration_since(metadata.modified()?)
                    .is_ok_and(|elapsed| elapsed >= age)
                {
                    expired = false;
                    break;
                }
            }
            if !expired {
                continue;
            }
        }
        for (path, metadata) in files {
            if !dry_run {
                std::fs::remove_file(path)
                    .with_context(|| format!("Failed to remove {}", path.display()))?;
            }
            println!(
                "{}: {}",
                if dry_run {
                    tr("Would remove", "将清理")
                } else {
                    tr("Removing", "正在清理")
                },
                path.display()
            );
            result.files += 1;
            result.bytes += metadata.len();
        }
    }
    Ok(result)
}

fn age_in_days(days: u64) -> anyhow::Result<Duration> {
    Ok(Duration::from_secs(
        days.checked_mul(86_400).context("Cache age is too large")?,
    ))
}

pub fn clean_cache(
    is_global: bool,
    apps: &[String],
    days: Option<u64>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let age = days.map(age_in_days).transpose()?;
    let root = cache_root(is_global);
    let _lock = download::lock_cleanup(&root)?;
    let result = clean_directory(&root, apps, age, dry_run, &[], SystemTime::now())?;
    println!(
        "{}: {} {} ({:.2} MB)",
        if dry_run {
            tr("Would delete", "预计清理")
        } else {
            tr("Deleted", "已清理")
        },
        result.files,
        tr("files", "个文件"),
        result.bytes as f64 / 1_048_576.0
    );
    Ok(())
}

pub fn display_specified_cache_info(app_name: &str, is_global: bool) -> anyhow::Result<()> {
    let apps = if app_name.trim() == "*" || app_name.is_empty() {
        vec![]
    } else {
        vec![app_name.to_string()]
    };
    clean_cache(is_global, &apps, None, false)
}

pub fn rm_all_cache(is_global: bool) -> anyhow::Result<()> {
    clean_cache(is_global, &[], None, false)
}

pub(crate) fn auto_cleanup(root: &Path, protected: &[String]) -> anyhow::Result<()> {
    use crate::config::get_config_value_no_print;
    if matches!(
        get_config_value_no_print("cache_auto_cleanup")
            .to_lowercase()
            .as_str(),
        "false" | "0"
    ) {
        return Ok(());
    }
    let days = get_config_value_no_print("cache_max_age_days");
    let days = if days.is_empty() {
        120
    } else {
        days.parse::<u64>().context("Invalid cache_max_age_days")?
    };
    anyhow::ensure!(days > 0, "cache_max_age_days must be greater than zero");
    let age = age_in_days(days)?;
    auto_cleanup_directory(root, protected, age, SystemTime::now())
}

fn auto_cleanup_directory(
    root: &Path,
    protected: &[String],
    age: Duration,
    now: SystemTime,
) -> anyhow::Result<()> {
    let marker = root.join(".hp-cache-pruned");
    // Check the maintenance marker under the lock so concurrent completions cannot prune twice.
    let Some(_lock) = download::try_lock_cleanup(root)? else {
        return Ok(());
    };
    if std::fs::metadata(&marker)
        .and_then(|meta| meta.modified())
        .is_ok_and(|modified| {
            now.duration_since(modified)
                .map_or(true, |elapsed| elapsed < Duration::from_secs(86_400))
        })
    {
        return Ok(());
    }
    clean_directory(root, &[], Some(age), false, protected, now)?;
    std::fs::write(marker, b"")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn cache_regression_auto_cleanup_is_periodic_and_skips_busy_cache() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let now = SystemTime::now();
        let old = now - Duration::from_secs(121 * 86_400);
        let expired = temp.path().join("old#1#abcdef0.zip");
        std::fs::write(&expired, b"data")?;
        File::options()
            .write(true)
            .open(&expired)?
            .set_modified(old)?;
        let locks = download::DownloadLocks::acquire(temp.path(), &[])?;
        auto_cleanup_directory(temp.path(), &[], age_in_days(120)?, now)?;
        assert!(expired.exists());
        drop(locks);
        auto_cleanup_directory(temp.path(), &[], age_in_days(120)?, now)?;
        assert!(!expired.exists());
        assert!(temp.path().join(".hp-cache-pruned").exists());
        std::fs::write(&expired, b"data")?;
        File::options()
            .write(true)
            .open(&expired)?
            .set_modified(old)?;
        auto_cleanup_directory(temp.path(), &[], age_in_days(120)?, now)?;
        assert!(expired.exists());
        Ok(())
    }

    #[test]
    fn cache_regression_prune_groups_resume_files_and_preserves_unknown_files() -> anyhow::Result<()>
    {
        let temp = tempfile::tempdir()?;
        let old = SystemTime::now() - Duration::from_secs(121 * 86_400);
        for name in [
            "old#1#abcdef0.zip",
            "old#1#abcdef0.zip.aria2",
            "busy#1#abcdef0.zip.incomplete",
            "busy#1#abcdef0.zip.incomplete.aria2",
            "old#1#1234567.zip.42.part",
            "notes.txt",
            ".hp-cache.lock",
        ] {
            let path = temp.path().join(name);
            std::fs::write(&path, b"data")?;
            File::options().write(true).open(path)?.set_modified(old)?;
        }
        std::fs::write(
            temp.path().join("busy#1#abcdef0.zip.incomplete.aria2"),
            b"active",
        )?;
        let age = Some(age_in_days(120)?);
        let preview = clean_directory(temp.path(), &[], age, true, &[], SystemTime::now())?;
        assert_eq!(preview.files, 3);
        assert!(temp.path().join("old#1#abcdef0.zip").exists());
        let result = clean_directory(temp.path(), &[], age, false, &[], SystemTime::now())?;
        assert_eq!(result.files, preview.files);
        assert_eq!(result.bytes, 12);
        for name in [
            "busy#1#abcdef0.zip.incomplete",
            "busy#1#abcdef0.zip.incomplete.aria2",
            "notes.txt",
            ".hp-cache.lock",
        ] {
            assert!(temp.path().join(name).exists());
        }
        Ok(())
    }

    #[test]
    fn cache_regression_removal_matches_exact_apps_and_protects_current_downloads()
    -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        for name in ["hp#1#abcdef0.exe", "php#1#abcdef0.zip", "hp#2#abcdef0.exe"] {
            std::fs::write(temp.path().join(name), b"data")?;
        }
        let protected = vec![
            temp.path()
                .join("hp#2#abcdef0.exe")
                .to_string_lossy()
                .into_owned(),
        ];
        let result = clean_directory(
            temp.path(),
            &["HP".into()],
            None,
            false,
            &protected,
            SystemTime::now(),
        )?;
        assert_eq!(result.files, 1);
        assert!(temp.path().join("php#1#abcdef0.zip").exists());
        assert!(temp.path().join("hp#2#abcdef0.exe").exists());
        assert!(age_in_days(u64::MAX).is_err());
        Ok(())
    }
}
