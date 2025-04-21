use crate::install::ArchiveFormat;
use crate::manifest::manifest_deserialize::StringArrayOrString;
use anyhow::bail;
use crossterm::style::Stylize;
use std::borrow::Cow;
use std::env;
use std::fs::File;
use std::io::Write;
use std::os::windows::fs;
use std::path::Path;
use std::process::{Command, Stdio};
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SevenZipStruct<'a> {
    archive_format: Box<[ArchiveFormat]>,
    archive_cache_files_path: Cow<'a, [&'a str]>,
    archive_names: Box<[String]>,
    app_name: &'a str,
    app_version: &'a str,
    target_dir: String,
    apps_root_dir: String,
    app_manifest_path: String,
    target_alias_name: &'a [&'a str],
}

impl<'a> SevenZipStruct<'a> {
    pub fn get_app_manifest_path(&self) -> &str {
        &self.app_manifest_path
    }

    pub fn get_target_alias_name(&self) -> &[&str] {
        self.target_alias_name
    }

    pub fn set_target_alias_name(&mut self, target_alias_name: &'a [&'a str]) {
        self.target_alias_name = target_alias_name;
    }

    pub fn set_app_manifest_path(&mut self, app_manifest_path: &str) {
        self.app_manifest_path = app_manifest_path.to_string();
    }
  
    pub fn get_apps_root_dir(&self) -> &str {
        &self.apps_root_dir
    }
  
    pub fn set_apps_root_dir(&mut self, dir: String) {
        self.apps_root_dir = dir
    }
  
    pub fn init(&mut self) {
        self.set_target_app_version_dir();
    }
  
    pub fn new() -> Self {
        Self {
            archive_format: Box::new([]),
            archive_cache_files_path: Cow::from(&[]),
            archive_names: Box::new([]),
            app_name: "",
            app_version: "",
            target_dir: String::new(),
            apps_root_dir: "".into(),
            app_manifest_path: "".to_string(),
            target_alias_name: &[],
        }
    }
  
    pub fn set_target_app_version_dir(&mut self) {
        let dir = self.get_apps_root_dir();
        let dir = format!(
            "{}\\{}\\{}",
            dir,
            self.get_app_name(),
            self.get_app_version()
        );
        self.target_dir = dir
    }
  
    pub fn get_target_app_version_dir(&self) -> &str {
        &self.target_dir
    }
  
    pub fn get_target_app_root_dir(&self) -> String {
        let dur = self.get_apps_root_dir();
        let dir = format!("{}\\{}", dur, self.get_app_name());
        dir
    }
  
    pub fn get_target_app_current_dir(&self) -> String {
        let app_root = self.get_target_app_root_dir();
        format!("{}\\current", app_root)
    }
  
    pub fn link_current_target_version_dir(&self) -> anyhow::Result<()> {
        let target = self.get_target_app_version_dir();
        let current = self.get_target_app_current_dir();
        fs::symlink_dir(target, &current)?; 
        println!("{} {} => {}", "Linking".dark_blue().bold(),&current.dark_green().bold() , target.dark_green().bold());
        Ok(())
    }
  
    pub fn get_app_name(&self) -> &'a str {
        self.app_name
    }
  
    pub fn get_app_version(&self) -> &'a str {
        self.app_version
    }
  
    pub fn set_app_name(&mut self, name: &'a str) {
        self.app_name = name;
    }
  
    pub fn set_app_version(&mut self, version: &'a str) {
        self.app_version = version;
    }
  
    pub fn get_archive_names(&self) -> Box<[String]> {
        self.archive_names.clone()
    }
  
    pub fn set_archive_names(&mut self, name: &[String]) {
        self.archive_names = Box::from(name.to_vec());
    }
  
    pub fn get_archive_format(&self) -> &[ArchiveFormat] {
        &self.archive_format
    }
  
    pub fn get_archive_cache_files_path(&self) -> Cow<'a, [&'a str]> {
        self.archive_cache_files_path.clone()
    }
  
    pub fn set_archive_cache_files_path(&mut self, path: &'a [&'a str]) {
        self.archive_cache_files_path = Cow::Borrowed(path);
    }
  
    pub fn set_archive_format(&mut self, format: &[ArchiveFormat]) {
        self.archive_format = format.to_vec().into_boxed_slice();
    }
  

    pub fn get_temp_7z_path(&self) -> String {
        let temp_dir = env::temp_dir();
        let exe_path = temp_dir.join("7z.exe");
        exe_path.to_str().unwrap().to_string()
    }
  
    pub fn get_temp_7z_dll_path(&self) -> String {
        let temp_dir = env::temp_dir();
        let exe_path = temp_dir.join("7z.dll");
        let str = exe_path.to_str().unwrap();
        str.to_string()
    }
  
    pub fn load_7z_to_temp_dir(&self) -> anyhow::Result<String> {
        const _7ZIP_EXE: &[u8] = include_bytes!("../../../../resources/7z.exe");
        const _7ZIP_DLL: &[u8] = include_bytes!("../../../../resources/7z.dll");
        let exe_path = self.get_temp_7z_path();
        let dll_path = self.get_temp_7z_dll_path();
        if Path::new(&exe_path).exists() && Path::new(&dll_path).exists() {
            return Ok(exe_path);
        }
        let mut exe_file = File::create(&exe_path)?;
        let mut dll_file = File::create(dll_path)?;
        exe_file.write_all(_7ZIP_EXE)?;
        exe_file.flush()?;
        exe_file.sync_all()?;
        dll_file.write_all(_7ZIP_DLL)?;
        dll_file.flush()?;
        dll_file.sync_all()?;
        drop(exe_file);
        drop(dll_file);
        Ok(exe_path)
    }

    pub fn extract_archive_child_to_target_child_dir(
        &self,
        extract_dir: Vec<String>,
        extract_to: Vec<String>,
    ) -> anyhow::Result<()> {
        let archive_items = self.get_archive_names();
        let archive_paths = self.get_archive_cache_files_path().to_vec();
        if archive_items.is_empty() || archive_paths.is_empty() {
            bail!("No archive files found.");
        }
        let _7z: String = self.load_7z_to_temp_dir()?;
        if !self.target_is_valid() {
            bail!("Target directory is not in scoop child tree.")
        }

        let target_dir = self.get_target_app_version_dir();
        if !Path::new(target_dir).exists() {
            std::fs::create_dir_all(target_dir)?;
        }
        let extract_to_dir = extract_to
            .iter()
            .map(|path| format!("{}\\{}", target_dir, path))
            .collect::<Vec<_>>();
        extract_to.iter().for_each(|path| {
            if !Path::new(path).exists() {
                std::fs::create_dir_all(path).unwrap();
            }
        });
        let result = archive_items
            .iter()
            .zip(archive_paths)
            .zip(extract_to_dir)
            .zip(extract_dir)
            .zip(self.get_archive_format())
            .zip(self.get_target_alias_name())
            .try_for_each(
                |(((((archive_name, path), extract_to), extract_dir), archive_format), alias)| {
                    print!(
                        "{}  {}......",
                        "Extracting archive".dark_blue().bold(),
                        archive_name.clone().dark_cyan().bold()
                    );
                    if *archive_format == ArchiveFormat::EXE
                        || *archive_format == ArchiveFormat::Other
                    {
                        let target_dir = if alias.is_empty() {
                            format!("{}\\{}", target_dir, archive_name)
                        } else {
                            format!("{}\\{}", target_dir, alias)
                        };
                        std::fs::copy(path, target_dir).expect("Failed to copy archive");
                        println!("✅");
                        Ok(())
                    } else {
                        let target = format!("-o{}", extract_to);
                        let output = Command::new(&_7z)
                            .arg("x")
                            .arg(path)
                            .arg(target)
                            .arg("-aoa") // *!自动覆盖同名文件
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .output()?;
                        if !output.status.success() {
                            let error = String::from_utf8_lossy(&output.stderr);
                            bail!("7z command failed: {}", error)
                        } else {
                            let child_dir = format!("{}\\{}", extract_to, extract_dir);
                            // log::debug!("child dir: {}", child_dir);
                            for entry in std::fs::read_dir(&child_dir)? {
                                let entry = entry?;
                                let from = entry.path();
                                let file_name = entry.file_name();
                                let to = Path::new(&extract_to).join(file_name);
                                std::fs::rename(from, to)?;
                            }
                            std::fs::remove_dir_all(&child_dir)?;
                            println!("✅");
                            Ok(())
                        }
                    }
                },
            );
        if result.is_err() {
            bail!("Failed to extract archive: {}", result.unwrap_err());
        }
        self.link_current_target_version_dir()?;
        Ok(())
    }

    pub fn extract_archive_child_to_target_dir(
        &self,
        archive_child_dir: Vec<String>,
    ) -> anyhow::Result<()> {
        let archive_items = self.get_archive_names();
        let archive_paths = self.get_archive_cache_files_path().to_vec();
        if archive_items.is_empty() || archive_paths.is_empty() {
            bail!("No archive files found.");
        }
        let _7z: String = self.load_7z_to_temp_dir()?;
        if !self.target_is_valid() {
            bail!("Target directory is not in scoop child tree.")
        }

        let target_dir = self.get_target_app_version_dir();
        if !Path::new(target_dir).exists() {
            std::fs::create_dir_all(target_dir)?;
        }
        let result = archive_items
            .iter()
            .zip(archive_paths)
            .zip(archive_child_dir)
            .zip(self.get_archive_format())
            .zip(self.get_target_alias_name())
            .try_for_each(
                |((((archive_name, path), child_dir), archive_format), alias)| {
                    print!(
                        "{}  {}......",
                        "Extracting archive".dark_blue().bold(),
                        archive_name.clone().dark_cyan().bold()
                    );
                    if *archive_format == ArchiveFormat::EXE
                        || *archive_format == ArchiveFormat::Other
                    {
                        let target_dir = if alias.is_empty() {
                            format!("{}\\{}", target_dir, archive_name)
                        } else {
                            format!("{}\\{}", target_dir, alias)
                        };
                        std::fs::copy(path, target_dir).expect("Failed to copy archive");
                        println!("✅");
                        Ok(())
                    } else {
                        let target = format!("-o{}", target_dir);
                        let child_dir = format!("{}\\{}", target_dir, child_dir);
                        let output = Command::new(&_7z)
                            .arg("x")
                            .arg(path)
                            .arg(target)
                            .arg("-aoa") // *!自动覆盖同名文件
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .output()
                            .expect("Failed to extract archive");
                        if !output.status.success() {
                            let error = String::from_utf8_lossy(&output.stderr);
                            bail!("7z command failed: {}", error)
                        } else {
                            for entry in std::fs::read_dir(&child_dir)? {
                                let entry = entry?;
                                let from = entry.path();
                                let file_name = entry.file_name();
                                let to = Path::new(&target_dir).join(file_name);
                                std::fs::rename(from, to)?;
                            }
                            std::fs::remove_dir_all(&child_dir)?; // 清理原来的空目录
                            println!("✅");
                            Ok(())
                        }
                    }
                },
            );

        if result.is_err() {
            bail!("Failed to extract archive: {}", result.unwrap_err());
        }
        self.link_current_target_version_dir()?;

        Ok(())
    }

    pub fn extract_archive_to_target_dir(
        &self,
        target_dir: Option<Vec<String>>,
    ) -> anyhow::Result<()> {
        let archive_items = self.get_archive_names();
        let archive_paths = self.get_archive_cache_files_path().to_vec();
        if archive_items.is_empty() || archive_paths.is_empty() {
            bail!("No archive files found.");
        }
        let _7z: String = self.load_7z_to_temp_dir()?;
        if !self.target_is_valid() {
            bail!("Target directory is not in scoop child tree.")
        }

        if target_dir.is_none() {
            let target_dir = self.get_target_app_version_dir();
            if !Path::new(target_dir).exists() {
                std::fs::create_dir_all(target_dir)?;
            }
            let result = archive_items
                .iter()
                .zip(archive_paths)
                .zip(self.get_archive_format())
                .zip(self.get_target_alias_name())
                .try_for_each(|(((archive_name, path), archive_format), target_alias)| {
                    print!(
                        "{}  {}......",
                        "Extracting archive".dark_blue().bold(),
                        archive_name.clone().dark_cyan().bold()
                    );
                    if *archive_format == ArchiveFormat::EXE
                        || *archive_format == ArchiveFormat::Other
                    {
                        // 复制exe到别名路径
                        let target_dir = if target_alias.is_empty() {
                            format!("{}\\{}", target_dir, archive_name)
                        } else {
                            format!("{}\\{}", target_dir, target_alias)
                        }; 
                        // println!("target alias dir {target_dir}"); 
                        std::fs::copy(path, target_dir).expect("Failed to copy archive");
                        println!("✅");
                        Ok(())
                    } else {
                        let target = format!("-o{}", target_dir);

                        let output = Command::new(&_7z)
                            .arg("x")
                            .arg(path)
                            .arg(target)
                            .arg("-aoa") // *!自动覆盖同名文件
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .output()?;
                        if !output.status.success() {
                            let error = String::from_utf8_lossy(&output.stderr);
                            bail!("7z command failed: {}", error)
                        } else {
                            println!("✅");
                            Ok(())
                        }
                    }
                });
            if result.is_err() {
                bail!("Failed to extract archive: {}", result.unwrap_err());
            }
            self.link_current_target_version_dir()?;
        } else {
            let target_dirs = target_dir.unwrap();
            target_dirs.iter().for_each(|target_dir| {
                if !Path::new(target_dir).exists() {
                    std::fs::create_dir_all(target_dir).unwrap();
                }
            });
            let result = archive_items
                .iter()
                .zip(archive_paths)
                .zip(target_dirs)
                .try_for_each(|((archive_name, archive_path), dest)| {
                    print!(
                        "{}  {}......",
                        "Extracting archive".dark_blue().bold(),
                        archive_name.clone().dark_cyan().bold()
                    );
                    let target = format!("-o{}", dest);
                    let output = Command::new(&_7z)
                        .arg("x")
                        .arg(archive_path)
                        .arg(target)
                        .arg("-aoa") // *!自动覆盖同名文件
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()?;
                    if !output.status.success() {
                        let error = String::from_utf8_lossy(&output.stderr);
                        bail!("7z command failed: {}", error)
                    } else {
                        println!("✅");
                        Ok(())
                    }
                });
            if result.is_err() {
                bail!("Failed to extract archive: {}", result.unwrap_err());
            }
            self.link_current_target_version_dir()?;
        };

        Ok(())
    }

    #[doc(hidden)]
    pub fn invoke_7z_command(
        &self,
        extract_dir: Option<StringArrayOrString>,
        extract_to: Option<StringArrayOrString>,
    ) -> anyhow::Result<()> {
        if extract_dir.is_none() && extract_to.is_none() {
            self.extract_archive_to_target_dir(None)?;
            Ok(())
        } else if extract_dir.is_none() && extract_to.is_some() {
            let target_dir = self.get_target_app_version_dir();
            let extract_to = extract_to.unwrap();

            let target = match extract_to {
                StringArrayOrString::Null => None,
                StringArrayOrString::StringArray(extract_dir) => Some(extract_dir),
                StringArrayOrString::String(extract_dir) => Some(Vec::from([extract_dir])),
            };
            if target.is_some() {
                let target = target.unwrap();

                if !target.is_empty() {
                    let target_dir = target
                        .iter()
                        .map(|path| {
                            let path = format!("{}\\{}", target_dir, path);
                            path
                        })
                        .collect::<Vec<_>>();
                    self.extract_archive_to_target_dir(Some(target_dir))?
                }
            }
            Ok(())
        } else if extract_dir.is_some() && extract_to.is_none() {
            let extract_dir = extract_dir.unwrap();
            let extract_dir = match extract_dir {
                StringArrayOrString::StringArray(extract_dir) => extract_dir,
                StringArrayOrString::String(extract_dir) => Vec::from([extract_dir]),
                StringArrayOrString::Null => Vec::new(),
            };
            if !extract_dir.is_empty() {
                self.extract_archive_child_to_target_dir(extract_dir)?;
                Ok(())
            } else {
                bail!("Parse Error : extract_dir is empty ")
            }
        } else if extract_dir.is_some() && extract_to.is_some() {
            let extract_dir = extract_dir.unwrap();
            let extract_dir = match extract_dir {
                StringArrayOrString::StringArray(extract_dir) => extract_dir,
                StringArrayOrString::String(extract_dir) => Vec::from([extract_dir]),
                StringArrayOrString::Null => Vec::new(),
            };
            let extract_to = extract_to.unwrap();
            let extract_to = match extract_to {
                StringArrayOrString::Null => vec![],
                StringArrayOrString::StringArray(extract_dir) => extract_dir,
                StringArrayOrString::String(extract_dir) => Vec::from([extract_dir]),
            };

            if extract_dir.is_empty() || extract_to.is_empty() {
                eprintln!("Empty Error ,check dir {}", self.get_app_manifest_path());
                bail!("Parse Error : extract_dir or extract_to is empty ")
            } else {
                self.extract_archive_child_to_target_child_dir(extract_dir, extract_to)?;
                Ok(())
            }
        } else {
            bail!("Invalid arguments.")
        }
    }

    pub fn target_is_valid(&self) -> bool {
        let target_dir = self.get_target_app_root_dir();
        let path = Path::new(&target_dir);
        let parent = path.parent().unwrap();
        let parent_name = parent.file_name().unwrap().to_str().unwrap().to_lowercase();
        if parent_name != "apps" {
            false
        } else {
            true
        }
    }
}

mod test_7z {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    fn test_invoke_7z() {
        let _zip = SevenZipStruct::new();
    }
}
