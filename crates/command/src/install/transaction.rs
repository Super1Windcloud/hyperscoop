//! 安装事务：保证安装/升级失败时不会破坏已安装的版本
use crate::i18n::tr;
use crate::utils::system::{
    create_dir_link, kill_processes_using_app, read_dir_link, remove_dir_link,
};
use anyhow::Context;
use crossterm::style::Stylize;
use std::fs;
use std::path::{Path, PathBuf};

/// 记录安装开始前 current 链接与版本目录的状态，未 commit 时在 Drop 中回滚
pub struct InstallTransaction {
    current_dir: PathBuf,
    version_dir: PathBuf,
    previous_target: Option<PathBuf>,
    version_dir_existed: bool,
    committed: bool,
}

impl InstallTransaction {
    /// 必须在创建版本目录、切换 current 之前调用
    pub fn begin(current_dir: impl Into<PathBuf>, version_dir: impl Into<PathBuf>) -> Self {
        let current_dir = current_dir.into();
        let version_dir = version_dir.into();
        Self {
            previous_target: read_dir_link(&current_dir),
            version_dir_existed: version_dir.exists(),
            current_dir,
            version_dir,
            committed: false,
        }
    }

    /// 实际版本目录与预期不一致时放弃清理版本目录，避免误删
    pub fn verify_version_dir(&mut self, actual_version_dir: &str) {
        if Path::new(actual_version_dir) != self.version_dir {
            log::warn!(
                "install transaction version dir mismatch: expected {}, actual {}",
                self.version_dir.display(),
                actual_version_dir
            );
            self.version_dir_existed = true;
        }
    }

    pub fn commit(mut self) {
        self.committed = true;
    }

    /// 未发生安装（如仅下载）时静默撤销预创建的版本目录
    pub fn discard(mut self) {
        self.committed = true;
        if let Err(err) = self.rollback() {
            log::warn!("discard install transaction failed: {:#}", err);
        }
    }

    fn rollback(&self) -> anyhow::Result<()> {
        let now_target = read_dir_link(&self.current_dir);
        if now_target != self.previous_target {
            if now_target.is_some() {
                remove_dir_link(&self.current_dir).context("remove new current link failed")?;
            }
            if let Some(previous) = self.previous_target.as_ref().filter(|p| p.exists()) {
                create_dir_link(previous, &self.current_dir)
                    .context("restore previous current link failed")?;
            }
        }
        // 只删除本次安装新建的版本目录，已存在的版本目录保持原样
        if !self.version_dir_existed && self.version_dir.exists() {
            fs::remove_dir_all(&self.version_dir).context(format!(
                "remove incomplete version dir {} failed",
                self.version_dir.display()
            ))?;
        }
        Ok(())
    }
}

impl Drop for InstallTransaction {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        match self.rollback() {
            Ok(()) => eprintln!(
                "{}",
                tr(
                    "Installation failed, previous state has been restored",
                    "安装失败，已回滚到安装前的状态"
                )
                .dark_yellow()
                .bold()
            ),
            Err(err) => eprintln!(
                "{} {:#}",
                tr("Rollback failed:", "回滚失败:").dark_red().bold(),
                err
            ),
        }
    }
}

/// `--force` 重装时先把整个 app 目录移入备份目录，安装成功后删除备份，失败时还原
pub struct AppDirBackup {
    app_dir: PathBuf,
    backup_dir: Option<PathBuf>,
    committed: bool,
}

impl AppDirBackup {
    pub fn create(
        app_dir: impl Into<PathBuf>,
        backup_root: impl AsRef<Path>,
        app_name: &str,
    ) -> anyhow::Result<Self> {
        let app_dir = app_dir.into();
        if !app_dir.exists() {
            return Ok(Self {
                app_dir,
                backup_dir: None,
                committed: false,
            });
        }
        let backup_root = backup_root.as_ref();
        fs::create_dir_all(backup_root).context("create backup root dir failed")?;
        // 带上 pid 与时间戳，避免覆盖异常退出遗留的备份
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default();
        let backup_dir = backup_root.join(format!("{}-{}-{}", app_name, std::process::id(), stamp));
        if fs::rename(&app_dir, &backup_dir).is_err() {
            kill_processes_using_app(app_name);
            fs::rename(&app_dir, &backup_dir).context(format!(
                "move app dir '{}' to backup failed",
                app_dir.display()
            ))?;
        }
        log::debug!("backup {} => {}", app_dir.display(), backup_dir.display());
        Ok(Self {
            app_dir,
            backup_dir: Some(backup_dir),
            committed: false,
        })
    }

    /// 安装成功后删除备份；删除失败只告警，不影响安装结果
    pub fn commit(mut self) {
        self.committed = true;
        if let Some(backup_dir) = &self.backup_dir {
            if let Err(err) = remove_app_dir(backup_dir) {
                log::warn!(
                    "remove backup dir {} failed: {:#}",
                    backup_dir.display(),
                    err
                );
            }
        }
    }

    fn restore(&self, backup_dir: &Path) -> anyhow::Result<()> {
        if self.app_dir.exists() {
            remove_app_dir(&self.app_dir).context("remove incomplete app dir failed")?;
        }
        fs::rename(backup_dir, &self.app_dir).context(format!(
            "restore app dir from backup {} failed",
            backup_dir.display()
        ))
    }
}

impl Drop for AppDirBackup {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        let Some(backup_dir) = &self.backup_dir else {
            return;
        };
        if let Err(err) = self.restore(backup_dir) {
            eprintln!(
                "{} {:#}",
                tr(
                    "Restore app dir failed, backup kept at:",
                    "还原 app 目录失败，备份保留在:"
                )
                .dark_red()
                .bold(),
                err
            );
        }
    }
}

/// 删除 app 目录前先移除 current 链接，防止其绝对路径目标被误删
fn remove_app_dir(app_dir: &Path) -> anyhow::Result<()> {
    let current = app_dir.join("current");
    if read_dir_link(&current).is_some() {
        remove_dir_link(&current)?;
    }
    fs::remove_dir_all(app_dir)?;
    Ok(())
}

#[cfg(test)]
mod test_install_transaction {
    use super::*;

    fn write_file(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn failed_upgrade_restores_previous_current_and_removes_new_version() {
        let temp = tempfile::tempdir().unwrap();
        let app_dir = temp.path().join("demo");
        let old = app_dir.join("1.0");
        let new = app_dir.join("2.0");
        let current = app_dir.join("current");
        write_file(&old.join("demo.exe"), "old");
        create_dir_link(&old, &current).unwrap();

        {
            let _tx = InstallTransaction::begin(&current, &new);
            write_file(&new.join("demo.exe"), "new");
            remove_dir_link(&current).unwrap();
            create_dir_link(&new, &current).unwrap();
        }

        assert_eq!(read_dir_link(&current), Some(old.clone()));
        assert_eq!(fs::read_to_string(current.join("demo.exe")).unwrap(), "old");
        assert!(!new.exists());
    }

    #[test]
    fn failed_fresh_install_removes_current_and_version_dir() {
        let temp = tempfile::tempdir().unwrap();
        let app_dir = temp.path().join("demo");
        let version = app_dir.join("1.0");
        let current = app_dir.join("current");

        {
            let _tx = InstallTransaction::begin(&current, &version);
            write_file(&version.join("demo.exe"), "new");
            create_dir_link(&version, &current).unwrap();
        }

        assert!(fs::symlink_metadata(&current).is_err());
        assert!(!version.exists());
    }

    #[test]
    fn committed_transaction_keeps_new_state() {
        let temp = tempfile::tempdir().unwrap();
        let app_dir = temp.path().join("demo");
        let old = app_dir.join("1.0");
        let new = app_dir.join("2.0");
        let current = app_dir.join("current");
        write_file(&old.join("demo.exe"), "old");
        create_dir_link(&old, &current).unwrap();

        let tx = InstallTransaction::begin(&current, &new);
        write_file(&new.join("demo.exe"), "new");
        remove_dir_link(&current).unwrap();
        create_dir_link(&new, &current).unwrap();
        tx.commit();

        assert_eq!(read_dir_link(&current), Some(new.clone()));
        assert!(old.exists());
    }

    #[test]
    fn failed_reinstall_keeps_existing_version_dir() {
        let temp = tempfile::tempdir().unwrap();
        let app_dir = temp.path().join("demo");
        let version = app_dir.join("1.0");
        let current = app_dir.join("current");
        write_file(&version.join("demo.exe"), "old");
        create_dir_link(&version, &current).unwrap();

        {
            let _tx = InstallTransaction::begin(&current, &version);
        }

        assert_eq!(read_dir_link(&current), Some(version.clone()));
        assert!(version.join("demo.exe").exists());
    }

    #[test]
    fn version_dir_mismatch_disables_version_cleanup() {
        let temp = tempfile::tempdir().unwrap();
        let expected = temp.path().join("demo").join("nightly-20261029");
        let actual = temp.path().join("demo").join("nightly");

        {
            let mut tx = InstallTransaction::begin(temp.path().join("demo/current"), &expected);
            tx.verify_version_dir(actual.to_str().unwrap());
            write_file(&expected.join("demo.exe"), "unknown");
        }

        assert!(expected.exists());
    }

    #[test]
    fn failed_force_install_restores_app_dir_from_backup() {
        let temp = tempfile::tempdir().unwrap();
        let app_dir = temp.path().join("apps").join("demo");
        let backup_root = temp.path().join("hp-backup");
        let old = app_dir.join("1.0");
        let current = app_dir.join("current");
        write_file(&old.join("demo.exe"), "old");
        create_dir_link(&old, &current).unwrap();

        {
            let _backup = AppDirBackup::create(&app_dir, &backup_root, "demo").unwrap();
            assert!(!app_dir.exists());
            write_file(&app_dir.join("1.0").join("demo.exe"), "partial");
        }

        assert_eq!(fs::read_to_string(current.join("demo.exe")).unwrap(), "old");
        assert_eq!(fs::read_dir(&backup_root).unwrap().count(), 0);
    }

    #[test]
    fn committed_force_install_removes_backup_without_touching_new_install() {
        let temp = tempfile::tempdir().unwrap();
        let app_dir = temp.path().join("apps").join("demo");
        let backup_root = temp.path().join("hp-backup");
        let version = app_dir.join("1.0");
        let current = app_dir.join("current");
        write_file(&version.join("demo.exe"), "old");
        create_dir_link(&version, &current).unwrap();

        let backup = AppDirBackup::create(&app_dir, &backup_root, "demo").unwrap();
        // 新安装重建同一路径，备份中的 current 链接此时指向新安装
        write_file(&version.join("demo.exe"), "new");
        create_dir_link(&version, &current).unwrap();
        backup.commit();

        assert_eq!(fs::read_to_string(current.join("demo.exe")).unwrap(), "new");
        assert_eq!(fs::read_dir(&backup_root).unwrap().count(), 0);
    }
}
