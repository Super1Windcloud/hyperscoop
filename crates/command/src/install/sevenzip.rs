use crate::install::{ArchiveFormat, InstallOptions};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use anyhow::bail;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive] 
pub struct SevenZipStruct<'a> {
    archive_format: ArchiveFormat,
    archive_cache_dir: &'a str,
    archive_name: &'a str,
    app_name: &'a str,
    app_version: &'a str,
    target_dir: String,
    install_options: &'a [InstallOptions],
}

impl<'a> SevenZipStruct<'a> {
    pub fn new(options: &'a [InstallOptions]) -> Self {
        Self {
            archive_format: ArchiveFormat::SevenZip,
            archive_cache_dir: "",
            archive_name: "",
            app_name: "",
            app_version: "",
            target_dir: String::new(),
            install_options: options,
        }
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
    pub fn get_archive_name(&self) -> &'a str {
        self.archive_name
    }
    pub fn set_archive_name(&mut self, name: &'a str) {
        self.archive_name = name;
    }

    pub fn get_archive_format(&self) -> &ArchiveFormat {
        &self.archive_format
    }
    pub fn get_archive_dir(&self) -> &str {
        &self.archive_cache_dir
    }
    pub fn set_archive_dir(&mut self, path: &'a str) {
        self.archive_cache_dir = path;
    }
    pub fn set_archive_format(&mut self, format: ArchiveFormat) {
        self.archive_format = format;
    }
    pub fn get_temp_7z_path(&self) -> String   {
        let temp_dir = env::temp_dir();
        let exe_path = temp_dir.join("7z.exe");
        exe_path.to_str().unwrap() .to_string()
    }
    pub fn get_temp_7z_dll_path(&self) -> String   {
        let temp_dir = env::temp_dir();
        let exe_path = temp_dir.join("7z.dll");
        let str =exe_path.to_str().unwrap();  
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
    #[must_use]  
    #[doc(hidden)]
    pub fn  invoke_7z_command (&self, command: &str) -> anyhow::Result<()> {
         let _7z = self.load_7z_to_temp_dir()? ;
         let  output =Command::new(_7z).arg(command.trim().to_lowercase()).output()?;
         if!output.status.success() {
            let  error = String::from_utf8_lossy(&output.stderr);
             bail!("7z command failed: {}", error)
         }
         let  output = String::from_utf8_lossy(&output.stdout);
          println!("{}", output);
        Ok(())
  }
}

mod test_7z {
    #[allow(unused_imports)] 
    use super::*;
    #[test]
    fn test_invoke_7z() {
        let zip = SevenZipStruct::new(&[InstallOptions::Global]);
        zip.invoke_7z_command("i  ").unwrap()
    }
}
