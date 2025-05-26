use crate::init_env::{
    get_cache_dir_path, get_cache_dir_path_global, get_shims_root_dir, get_shims_root_dir_global,
};
use crate::install::InstallOptions::NoUseDownloadCache;
use crate::install::{install_app, ArchiveFormat, InstallOptions};
use crate::manifest::manifest_deserialize::StringArrayOrString;
use crate::utils::system::{is_broken_symlink, kill_processes_using_app};
use anyhow::{bail, Context};
use crossterm::style::Stylize;
use std::env;
use std::fs::File;
use std::io::Write;
use std::os::windows::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use which::which;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SevenZipStruct<'a> {
    archive_format: Box<[ArchiveFormat]>,
    archive_cache_files_path: Vec<String>,
    archive_names: Box<[String]>,
    app_name: &'a str,
    app_version: &'a str,
    target_dir: String,
    apps_root_dir: String,
    app_manifest_path: String,
    target_alias_name: Box<[String]>,
    options: &'a [InstallOptions<'a>],
    cache_root_dir: String,
    final_cache_file_name: Box<[String]>,
}

impl<'a> SevenZipStruct<'a> {
    pub fn get_app_manifest_path(&self) -> &str {
        &self.app_manifest_path
    }

    pub fn set_final_cache_file_name(&mut self, final_cache_file_name: &[String]) {
        self.final_cache_file_name = final_cache_file_name.to_vec().into_boxed_slice();
    }

    pub fn get_final_cache_file_name(&self) -> Vec<String> {
        self.final_cache_file_name.clone().to_vec()
    }

    pub fn get_cache_root_dir(&self) -> &str {
        &self.cache_root_dir
    }

    pub fn set_cache_root_dir(&mut self) {
        let global = if self.get_options().contains(&InstallOptions::Global) {
            true
        } else {
            false
        };
        let dir = if global {
            get_cache_dir_path_global()
        } else {
            get_cache_dir_path()
        };
        self.cache_root_dir = dir
    }
    pub fn get_options(&self) -> &[InstallOptions<'a>] {
        self.options
    }

    pub fn set_options(&mut self, options: &'a [InstallOptions<'a>]) {
        self.options = options;
    }
    pub fn get_target_alias_name(&self) -> &[String] {
        self.target_alias_name.as_ref()
    }

    pub fn set_target_alias_name(&mut self, target_alias_name: Vec<String>) {
        self.target_alias_name = target_alias_name.into_boxed_slice();
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
        self.set_cache_root_dir();
    }

    pub fn new() -> Self {
        Self {
            archive_format: Box::new([]),
            archive_cache_files_path: Vec::new(),
            archive_names: Box::new([]),
            app_name: "",
            app_version: "",
            target_dir: String::new(),
            apps_root_dir: "".into(),
            app_manifest_path: "".to_string(),
            target_alias_name: Box::new([]),
            options: &[],
            cache_root_dir: "".to_string(),
            final_cache_file_name: Box::new([]),
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
        if Path::new(&current).exists() {
            std::fs::remove_dir_all(&current).context("remove current dir failed at line 112")?;
        }

        if is_broken_symlink(&current)? {
            log::debug!("{} is a broken symlink, removing it", &current);
            std::fs::remove_dir_all(&current) // can't use remove_file here
                .context("remove current dir failed at line 116")?;
        }
        fs::symlink_dir(target, &current)
            .context("create current dir symlink failed at line 120")?;

        println!(
            "{} {} => {}",
            "Linking".dark_blue().bold(),
            &current.dark_green().bold(),
            target.dark_green().bold()
        );
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

    pub fn get_archive_cache_files_path(&self) -> Vec<String> {
        self.archive_cache_files_path.clone()
    }

    pub fn set_archive_cache_files_path(&mut self, path: Vec<String>) {
        self.archive_cache_files_path = path
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

    pub fn output_current_exe(&self, path: PathBuf, shim_root_dir: &str) -> anyhow::Result<String> {
        let parent = path.parent().unwrap();
        if parent.to_str().unwrap() != shim_root_dir {
            return Ok(path.to_str().unwrap().trim().to_string());
        }

        let path = path.to_str().unwrap();

        let splits = path.split(".").collect::<Vec<&str>>();
        if splits.len() != 2 {
            bail!("{path} is not a valid path")
        }
        let prefix = splits[0];
        let suffix = splits[1];
        if suffix == "exe" || suffix == "com" {
            let shim_file = format!("{}.shim", prefix);
            if !Path::new(&shim_file).exists() {
                bail!("{shim_file} is not exists")
            }
            let content = std::fs::read_to_string(shim_file)
                .context("failed to read shim file at line 203")?;
            let first_line = content.lines().next().unwrap();
            let content = first_line.replace("path = ", "").replace("\"", "");
            Ok(content.trim().to_string())
        } else if suffix == "cmd" || suffix == "bat" || suffix == "ps1" {
            let cmd_file = format!("{}.cmd", prefix);
            if !Path::new(&cmd_file).exists() {
                bail!("{cmd_file} is not exists")
            }
            let content = self.extract_rem_comments(cmd_file.as_str());
            Ok(content.trim().to_string())
        } else {
            eprintln!("Unknown suffix: {}", suffix);
            bail!("{path} is not a valid path")
        }
    }

    fn extract_rem_comments(&self, file_path: &str) -> String {
        let content = std::fs::read_to_string(file_path).expect("Failed to read file");
        content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("@rem") {
                    Some(trimmed[4..].trim_start().to_string()) // 提取 "@rem" 后的内容
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn load_7z_to_temp_dir(&self) -> anyhow::Result<String> {
        const _7ZIP_EXE: &[u8] = include_bytes!("../../../../resources/7z.exe");
        const _7ZIP_DLL: &[u8] = include_bytes!("../../../../resources/7z.dll");
        let seven_zip = which("7z").ok();
        let exe_path = if seven_zip.is_some() {
            seven_zip.clone().unwrap().to_str().unwrap().to_string()
        } else {
            self.get_temp_7z_path()
        };
        let global = self.get_options().contains(&InstallOptions::Global);
        let dll_path = if seven_zip.is_some() && !global {
            let seven_zip_dir = seven_zip.unwrap();
            let shim_root_dir = if self.get_options().contains(&InstallOptions::Global) {
                get_shims_root_dir_global()
            } else {
                get_shims_root_dir()
            };
            let exe_current =
                self.output_current_exe(seven_zip_dir.clone(), shim_root_dir.as_str())?;
            let parent = Path::new(&exe_current).parent().unwrap();

            let dll_path = format!("{}\\7z.dll", parent.to_str().unwrap());

            if !Path::new(&dll_path).exists() {
                bail!("7z.dll not found in 7zip dir , please install 7zip and try again.")
            }
            dll_path
        } else {
            self.get_temp_7z_dll_path()
        };
        if Path::new(&exe_path).exists() && Path::new(&dll_path).exists() {
            log::debug!("7z.exe {} ,7z.dll {}", &exe_path, &dll_path);
            return Ok(exe_path);
        }
        let mut exe_file =
            File::create(&exe_path).context(format!("failed to create 7z.exe {}", &exe_path))?;
        let mut dll_file =
            File::create(&dll_path).context(format!("failed to create 7z.dll {}", &dll_path))?;
        exe_file
            .write_all(_7ZIP_EXE)
            .context(format!("failed to write 7z.exe to {}", &exe_path))?;
        exe_file.flush()?;
        exe_file.sync_all()?;
        dll_file
            .write_all(_7ZIP_DLL)
            .context(format!("failed to write 7z.dll to {}", &dll_path))?;
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
            std::fs::create_dir_all(target_dir)
                .context("Failed to create target version directory at line 303")?;
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
                    } else if *archive_format == ArchiveFormat::INNO {
                        println!("✅");

                        let output = Command::new("innounp").output();
                        if output.is_err() {
                            install_app("innounp", vec![].as_ref())
                                .expect("Failed to install innounp");
                        } else {
                            let output = output.unwrap();
                            if !output.status.success() {
                                install_app("innounp", vec![].as_ref())
                                    .expect("Failed to install innounp");
                            }
                        }
                        self.invoke_innounp_extract(extract_to.as_str(), path.as_str())
                            .expect("Failed to extract inno archive");
                        let child_dir = format!("{}\\{}", extract_to, extract_dir);

                        self.move_child_dir_to_root(&child_dir, target_dir)
                            .expect("Failed to move child dir to root");
                        Ok(())
                    } else if *archive_format == ArchiveFormat::MSI {
                        println!("✅");

                        let output = Command::new("lessmsi").arg("h").output();
                        if output.is_err() {
                            install_app("lessmsi", vec![].as_ref())
                                .expect("Failed to install lessmsi");
                        } else {
                            let output = output.unwrap();
                            if !output.status.success() {
                                install_app("lessmsi", vec![].as_ref())
                                    .expect("Failed to install lessmsi");
                            }
                        }

                        self.invoke_lessmsi_extract(extract_to.as_str(), path.as_str())
                            .expect("Failed to extract msi archive");
                        let child_dir = format!("{}\\{}", extract_to, extract_dir);

                        self.move_child_dir_to_root(&child_dir, target_dir)
                            .expect("Failed to move child dir to root");
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
                            for entry in std::fs::read_dir(&child_dir)
                                .context(format!("Failed to read child directory {}", &child_dir))?
                            {
                                let entry = entry?;
                                let from = entry.path();
                                let file_name = entry.file_name();
                                let to = Path::new(&extract_to).join(file_name);
                                std::fs::rename(&from, &to).context(format!(
                                    "Failed to move file {} to {}",
                                    from.display(),
                                    to.display()
                                ))?;
                            }
                            std::fs::remove_dir_all(&child_dir)
                                .context(format!("Failed to remove old child {}", &child_dir))?;
                            println!("✅");
                            Ok(())
                        }
                    }
                },
            );
        if result.is_err() {
            bail!("Failed to extract archive: {}", result.unwrap_err());
        }
        Ok(())
    }

    pub fn invoke_lessmsi_extract(&self, target_dir: &str, msi_file: &str) -> anyhow::Result<()> {
        let core_script = include_str!("../../../../asset_scripts/core.ps1");
        let decompress_script = include_str!("../../../../asset_scripts/decompress.ps1");
        let temp = env::temp_dir();
        let core_path = temp.join("core.ps1");
        let decompress_path = temp.join("decompress.ps1");
        let temp_str = temp.to_str().unwrap();
        if !core_path.exists() {
            std::fs::write(&core_path, core_script).context(format!(
                "Failed to write core script: {}",
                core_path.display()
            ))?;
        }
        if !decompress_path.exists() {
            std::fs::write(&decompress_path, decompress_script).context(format!(
                "Failed to write decompress script: {}",
                decompress_path.display()
            ))?;
        }
        let include_header = format!(
            r#". "{temp_str}core.ps1";
. "{temp_str}decompress.ps1";

Expand-MsiArchive  "{msi_file}" "{target_dir}"  -Removal
 "#
        );
        let output = Command::new("PowerShell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(include_header)
            .output()
            .expect("Failed to execute PowerShell MSI Extract");

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            bail!("PowerShell MSI Extract failed: {}", error)
        }

        Ok(())
    }

    pub fn invoke_innounp_extract(&self, target_dir: &str, inno_file: &str) -> anyhow::Result<()> {
        let core_script = include_str!("../../../../asset_scripts/core.ps1");
        let decompress_script = include_str!("../../../../asset_scripts/decompress.ps1");
        let temp = env::temp_dir();
        let core_path = temp.join("core.ps1");
        let decompress_path = temp.join("decompress.ps1");
        let temp_str = temp.to_str().unwrap();
        if !core_path.exists() {
            std::fs::write(&core_path, core_script).context(format!(
                "Failed to write core script: {} at line 477",
                core_path.display()
            ))?;
        }
        if !decompress_path.exists() {
            std::fs::write(&decompress_path, decompress_script).context(format!(
                "Failed to write decompress script: {} at line 481",
                decompress_path.display()
            ))?;
        }
        let include_header = format!(
            r#". "{temp_str}core.ps1";
. "{temp_str}decompress.ps1";

Expand-InnoArchive "{inno_file}" "{target_dir}"  -Removal
 "#
        );
        let output = Command::new("PowerShell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(include_header)
            .output()
            .expect("Failed to execute PowerShell Inno Extract");

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            bail!("PowerShell Inno Extract failed: {}", error)
        }

        Ok(())
    }

    pub fn link_current(&self) -> anyhow::Result<()> {
        let result = self.link_current_target_version_dir();
        if result.is_err() {
            eprintln!(
                "Error Link: {}",
                result.err().unwrap().to_string().dark_red().bold()
            );
            let current = self.get_target_app_current_dir();
            let app_name = self.get_app_name();
            if Path::new(&current).exists() {
                kill_processes_using_app(app_name);
            }
            self.link_current_target_version_dir()?;
        }
        if self.options.contains(&NoUseDownloadCache) {
            let cache_file_path = self
                .get_final_cache_file_name()
                .iter()
                .map(|name| format!("{}\\{}", self.get_cache_root_dir(), name))
                .collect::<Vec<String>>();
            cache_file_path.iter().for_each(|path| {
                if Path::new(path).exists() {
                    std::fs::remove_file(path).expect("failed to remove cache file at line 568");
                }
            });
        }
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
            std::fs::create_dir_all(target_dir)
                .context("Failed to create target version directory at line 527")?;
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
                    } else if *archive_format == ArchiveFormat::INNO {
                        println!("✅");
                        let output = Command::new("innounp").output();
                        if output.is_err() {
                            install_app("innounp", vec![].as_ref())
                                .expect("Failed to install innounp");
                        } else {
                            let output = output.unwrap();
                            if !output.status.success() {
                                install_app("innounp", vec![].as_ref())
                                    .expect("Failed to install innounp");
                            }
                        }

                        self.invoke_innounp_extract(target_dir, path.as_str())
                            .expect("Failed to extract inno archive");
                        let child_dir = format!("{}\\{}", target_dir, child_dir);
                        self.move_child_dir_to_root(&child_dir, target_dir)
                            .expect("Failed to move child dir to root");
                        Ok(())
                    } else if *archive_format == ArchiveFormat::MSI {
                        println!("✅");

                        let output = Command::new("lessmsi").arg("h").output();
                        if output.is_err() {
                            install_app("lessmsi", vec![].as_ref())
                                .expect("Failed to install lessmsi");
                        } else {
                            let output = output.unwrap();
                            if !output.status.success() {
                                install_app("lessmsi", vec![].as_ref())
                                    .expect("Failed to install lessmsi");
                            }
                        }

                        self.invoke_lessmsi_extract(target_dir, path.as_str())
                            .expect("Failed to extract msi archive");
                        let child_dir = format!("{}\\{}", target_dir, child_dir);
                        self.move_child_dir_to_root(&child_dir, target_dir)
                            .expect("Failed to move child dir to root");
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
                            .output()
                            .expect("Failed to extract archive");

                        if !output.status.success() {
                            let error = String::from_utf8_lossy(&output.stderr);
                            bail!("7z command failed: {}", error)
                        } else {
                            let child_dir = format!("{}\\{}", target_dir, child_dir);
                            self.move_child_dir_to_root(&child_dir, target_dir)
                                .expect("Failed to move child dir to root");
                            println!("✅");

                            Ok(())
                        }
                    }
                },
            );

        if result.is_err() {
            bail!("Failed to extract archive: {}", result.unwrap_err());
        }

        Ok(())
    }

    pub fn move_child_dir_to_root(&self, child_dir: &str, target_dir: &str) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(&child_dir)
            .context("Failed to read moved child directory at line 630")?
        {
            let entry = entry?;
            let from = entry.path();
            let file_name = entry.file_name();
            let to = Path::new(&target_dir).join(file_name);
            std::fs::rename(from.as_path(), to.as_path()).context(format!(
                "Failed to move file {} to {} at line 636",
                from.display(),
                to.display()
            ))?;
        }
        std::fs::remove_dir_all(&child_dir)
            .context("Failed to remove old child directory at line 640")?; // 清理原来的空目录
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
        let _7z: String = self.load_7z_to_temp_dir().expect("Failed to load 7z.exe");
        if !self.target_is_valid() {
            bail!("Target directory is not in scoop child tree.")
        }
        let archive_formats = self.get_archive_format();
        let target_alias_names = self.get_target_alias_name();
        log::debug!("7z exe is entirely exists");
        if target_dir.is_none() {
            let target_dir = self.get_target_app_version_dir();
            if !Path::new(target_dir).exists() {
                std::fs::create_dir_all(target_dir).expect("Failed to create target directory");
            };
            log::debug!("created target dir is {}", target_dir);
            println!(
                "{}",
                format!("Extracting to {}", target_dir).dark_blue().bold()
            );

            let result = archive_items
                .iter()
                .zip(archive_paths)
                .zip(archive_formats)
                .zip(target_alias_names)
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
                    } else if *archive_format == ArchiveFormat::INNO {
                        println!("✅");

                        let output = Command::new("innounp").output();
                        if output.is_err() {
                            install_app("innounp", vec![].as_ref())
                                .expect("Failed to install innounp");
                        } else {
                            let output = output.unwrap();
                            if !output.status.success() {
                                install_app("innounp", vec![].as_ref())
                                    .expect("Failed to install innounp");
                            }
                        }
                        self.invoke_innounp_extract(target_dir, path.as_str())
                            .expect("Failed to extract inno archive");

                        Ok(())
                    } else if *archive_format == ArchiveFormat::MSI {
                        println!("✅");

                        let output = Command::new("lessmsi").arg("h").output();
                        if output.is_err() {
                            install_app("lessmsi", vec![].as_ref())
                                .expect("Failed to lessmsi innonounp");
                        } else {
                            let output = output.unwrap();
                            if !output.status.success() {
                                install_app("lessmsi", vec![].as_ref())
                                    .expect("Failed to install lessmsi");
                            }
                        }
                        self.invoke_lessmsi_extract(target_dir, path.as_str())
                            .expect("Failed to extract msi archive");

                        Ok(())
                    } else {
                        log::debug!("file is archive , invoke external command");
                        let target = format!("-o{}", target_dir);
                        let output = Command::new(&_7z)
                            .arg("x")
                            .arg(path)
                            .arg(target)
                            .arg("-aoa") // *!自动覆盖同名文件
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
            self.extract_archive_to_target_dir(None)
                .expect("extract archive to target directory");
            Ok(())
        } else if extract_dir.is_none() && extract_to.is_some() {
            let target_dir = self.get_target_app_version_dir();
            let extract_to = extract_to.unwrap();

            let target = match extract_to {
                StringArrayOrString::Null => None,
                StringArrayOrString::StringArray(extract_to) => Some(extract_to),
                StringArrayOrString::String(extract_to) => Some(Vec::from([extract_to])),
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
                StringArrayOrString::StringArray(extract_to) => extract_to,
                StringArrayOrString::String(extract_to) => Vec::from([extract_to]),
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
    fn test_extract_7z() {
        let _zip = SevenZipStruct::new();
    }

}
