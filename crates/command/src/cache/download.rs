use anyhow::{Context, bail};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

pub(super) const LOCK_FILE: &str = ".hp-cache.lock";
const LOCK_DIR: &str = ".hp-cache-locks";

fn open_lock(path: &Path) -> anyhow::Result<File> {
    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .with_context(|| format!("Failed to open cache lock {}", path.display()))?)
}

// Shared directory access permits unrelated downloads; cleanup takes an exclusive lock.
// Lock files stay in place so waiters always synchronize on the same file.
#[derive(Debug)]
pub(crate) struct DownloadLocks {
    _directory: File,
    _downloads: Vec<File>,
}

impl DownloadLocks {
    pub(crate) fn acquire(root: &Path, targets: &[String]) -> anyhow::Result<Self> {
        std::fs::create_dir_all(root.join(LOCK_DIR))?;
        let directory = open_lock(&root.join(LOCK_FILE))?;
        FileExt::lock_shared(&directory)?;
        let mut names: Vec<_> = targets
            .iter()
            .map(|target| {
                use sha2::{Digest, Sha256};
                let name = Path::new(target)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                hex::encode(Sha256::digest(name.to_lowercase().as_bytes()))
            })
            .collect();
        names.sort();
        names.dedup();
        let mut downloads = Vec::new();
        for name in names {
            let lock = open_lock(&root.join(LOCK_DIR).join(name))?;
            FileExt::lock_exclusive(&lock)?;
            downloads.push(lock);
        }
        Ok(Self {
            _directory: directory,
            _downloads: downloads,
        })
    }
}

pub(super) fn lock_cleanup(root: &Path) -> anyhow::Result<File> {
    let lock = open_lock(&root.join(LOCK_FILE))?;
    FileExt::lock_exclusive(&lock)?;
    Ok(lock)
}

pub(super) fn try_lock_cleanup(root: &Path) -> anyhow::Result<Option<File>> {
    let lock = open_lock(&root.join(LOCK_FILE))?;
    match FileExt::try_lock_exclusive(&lock) {
        Ok(()) => Ok(Some(lock)),
        Err(error)
            if error.kind() == std::io::ErrorKind::WouldBlock
                || error.raw_os_error() == fs2::lock_contended_error().raw_os_error() =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

pub(crate) fn is_complete(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_file())
        && !with_suffix(path, ".aria2").exists()
        && !with_suffix(path, ".incomplete.aria2").exists()
}

pub(crate) fn remove_if_exists(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

pub(crate) fn remove_download(path: &Path) -> anyhow::Result<()> {
    for suffix in ["", ".aria2", ".incomplete", ".incomplete.aria2"] {
        remove_if_exists(&with_suffix(path, suffix))?;
    }
    Ok(())
}

// Move old aria2 partials out of the completed namespace while preserving resume data.
pub(crate) fn prepare_aria2(path: &Path) -> anyhow::Result<PathBuf> {
    let pending = with_suffix(path, ".incomplete");
    let control = with_suffix(path, ".aria2");
    let pending_control = with_suffix(&pending, ".aria2");
    // Recover a process exit between moving the control file and its data file.
    if pending_control.exists() && !pending.exists() && path.is_file() {
        std::fs::rename(path, &pending)?;
    }
    if control.exists() {
        if path.is_file() && !pending.exists() {
            std::fs::rename(&control, &pending_control)?;
            std::fs::rename(path, &pending)?;
        } else {
            remove_if_exists(path)?;
            remove_if_exists(&control)?;
        }
    }
    // Without resume metadata, an interrupted preallocated file can look full-sized.
    if pending.exists() && !pending_control.exists() {
        remove_if_exists(&pending)?;
    }
    Ok(pending)
}

pub(crate) fn publish(pending: &Path, target: &Path) -> anyhow::Result<()> {
    if !is_complete(pending) {
        bail!("Download did not complete: {}", pending.display());
    }
    std::fs::rename(pending, target)
        .with_context(|| format!("Failed to publish download {}", target.display()))?;
    remove_if_exists(&with_suffix(target, ".aria2"))?;
    remove_if_exists(&with_suffix(target, ".incomplete.aria2"))?;
    remove_if_exists(&with_suffix(target, ".incomplete"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_regression_legacy_partial_is_resumable_and_never_a_hit() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let target = temp.path().join("app#1#abcdef0.zip");
        std::fs::write(&target, b"partial")?;
        std::fs::write(with_suffix(&target, ".aria2"), b"resume")?;
        assert!(!is_complete(&target));
        let pending = prepare_aria2(&target)?;
        assert!(!target.exists());
        assert_eq!(std::fs::read(&pending)?, b"partial");
        assert_eq!(std::fs::read(with_suffix(&pending, ".aria2"))?, b"resume");
        assert!(publish(&pending, &target).is_err());
        remove_if_exists(&with_suffix(&pending, ".aria2"))?;
        std::fs::write(&pending, b"complete")?;
        publish(&pending, &target)?;
        assert!(is_complete(&target));
        assert_eq!(std::fs::read(target)?, b"complete");
        Ok(())
    }

    #[test]
    fn cache_regression_force_removes_orphan_resume_state() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let target = temp.path().join("app.zip");
        for suffix in [".aria2", ".incomplete", ".incomplete.aria2"] {
            std::fs::write(with_suffix(&target, suffix), b"partial")?;
        }
        remove_download(&target)?;
        assert_eq!(std::fs::read_dir(temp.path())?.count(), 0);
        Ok(())
    }

    #[test]
    fn cache_regression_migration_recovers_between_renames_and_discards_untracked_partial()
    -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let target = temp.path().join("app.zip");
        std::fs::write(&target, b"preallocated")?;
        std::fs::write(with_suffix(&target, ".incomplete.aria2"), b"resume")?;
        assert!(!is_complete(&target));
        let pending = prepare_aria2(&target)?;
        assert!(!target.exists());
        assert_eq!(std::fs::read(&pending)?, b"preallocated");
        remove_if_exists(&with_suffix(&pending, ".aria2"))?;
        prepare_aria2(&target)?;
        assert!(!pending.exists());
        Ok(())
    }

    #[test]
    fn cache_regression_cleanup_and_same_target_downloads_are_locked() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let target = temp.path().join("app.zip").to_string_lossy().into_owned();
        let locks = DownloadLocks::acquire(temp.path(), &[target])?;
        let directory = open_lock(&temp.path().join(LOCK_FILE))?;
        assert!(FileExt::try_lock_exclusive(&directory).is_err());
        let lock_path = std::fs::read_dir(temp.path().join(LOCK_DIR))?
            .next()
            .unwrap()?
            .path();
        let download = open_lock(&lock_path)?;
        assert!(FileExt::try_lock_exclusive(&download).is_err());
        drop(locks);
        FileExt::try_lock_exclusive(&directory)?;
        FileExt::try_lock_exclusive(&download)?;
        Ok(())
    }
}
