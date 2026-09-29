use anyhow::{Context, Result};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::i18n::tr;

static LOCK_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct HpProcessLock {
    _file: Option<File>,
    pub path: PathBuf,
}

impl HpProcessLock {
    /// Acquire an exclusive cross-process lock for mutating operations (install, uninstall, update, reset).
    /// Supports re-entrant acquisition within the same process (e.g. recursive dependency installation).
    /// If another separate process holds the lock, notifies the user and waits until it is released.
    pub fn acquire(is_global: bool, _operation: &str) -> Result<Self> {
        let scoop_dir = if is_global {
            crate::init_env::init_scoop_global()
        } else {
            crate::init_env::init_user_scoop()
        };
        #[cfg(not(windows))]
        let scoop_dir = scoop_dir.replace('\\', "/");
        let lock_path = Path::new(&scoop_dir).join("hp.lock");

        // Re-entrant acquisition check within current process
        if LOCK_COUNT.fetch_add(1, Ordering::SeqCst) > 0 {
            return Ok(Self {
                _file: None,
                path: lock_path,
            });
        }

        if let Some(parent) = lock_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
        {
            Ok(f) => f,
            Err(e) => {
                LOCK_COUNT.fetch_sub(1, Ordering::SeqCst);
                return Err(e)
                    .with_context(|| format!("Failed to open lock file: {}", lock_path.display()));
            }
        };

        match file.try_lock_exclusive() {
            Ok(()) => Ok(Self {
                _file: Some(file),
                path: lock_path,
            }),
            Err(_) => {
                println!(
                    "{}",
                    tr(
                        "Another hp or scoop process is currently running. Waiting for lock...",
                        "检测到另一个 hp/scoop 进程正在运行，正在排队等待文件锁..."
                    )
                );
                if let Err(e) = file.lock_exclusive() {
                    LOCK_COUNT.fetch_sub(1, Ordering::SeqCst);
                    return Err(e).with_context(|| {
                        format!("Failed to acquire lock on {}", lock_path.display())
                    });
                }
                Ok(Self {
                    _file: Some(file),
                    path: lock_path,
                })
            }
        }
    }
}

impl Drop for HpProcessLock {
    fn drop(&mut self) {
        if LOCK_COUNT.fetch_sub(1, Ordering::SeqCst) == 1 {
            if let Some(file) = self._file.take() {
                let _ = file.unlock();
            }
        }
    }
}
