use crate::init_env::{
    get_app_current_dir, get_app_current_dir_global, get_app_version_dir,
    get_app_version_dir_global, get_persist_dir_path, get_persist_dir_path_global,
    get_psmodules_root_dir, get_psmodules_root_global_dir, get_shims_root_dir,
    get_shims_root_dir_global,
};
use crate::install::{
    DownloadManager, InstallOptions, create_default_shim_name_file, install_app,
    install_from_specific_bucket,
};
use crate::manifest::install_manifest::{SuggestObj, SuggestObjValue};
use crate::manifest::manifest_deserialize::{
    ArchitectureObject, PSModuleStruct, StringArrayOrString, StringOrArrayOrDoubleDimensionArray,
};
use crate::utils::system::{
    get_system_default_arch, get_system_env_str, get_system_env_var, get_user_env_str,
    get_user_env_var, set_global_env_var, set_user_env_var,
};
use anyhow::{Context, bail};
use crossterm::style::Stylize;
use regex::Regex;
use std::os::windows::fs;
use std::path::Path;
use which::which;

pub fn show_suggest(suggest: &SuggestObj) -> anyhow::Result<()> {
    println!(
        "{}",
        "建议安装以下依赖包 :".to_string().dark_yellow().bold()
    );

    for item in suggest {
        let name = item.0;
        let value = item.1;
        match value {
            SuggestObjValue::Null => {}
            SuggestObjValue::String(value) => {
                println!(
                    "{}",
                    format!("{} : {}", name, value)
                        .dark_grey()
                        .bold()
                        .to_string()
                );
            }
            SuggestObjValue::StringArray(arr) => {
                println!(
                    "{}",
                    format!("{} : {:?}", name, arr)
                        .dark_grey()
                        .bold()
                        .to_string()
                );
            }
        }
    }
    Ok(())
}

pub fn show_notes(notes: StringArrayOrString) -> anyhow::Result<()> {
    match notes {
        StringArrayOrString::StringArray(notes) => {
            println!("{}", "Notes : ".to_string().dark_cyan().bold());
            println!("{}", "_____ : ".to_string().dark_cyan().bold());
            for note in notes {
                println!(" {}", note.clone().dark_grey().bold());
            }
        }
        StringArrayOrString::String(note) => {
            println!("Notes : {}", note.clone().dark_grey().bold());
        }
        StringArrayOrString::Null => {}
    }
    Ok(())
}

pub fn handle_depends(depends: String, options: &[InstallOptions<'_>]) -> anyhow::Result<()> {
    if depends.contains('/') {
        let arr = depends.split('/').collect::<Vec<&str>>();
        if arr.len() != 2 {
            bail!("manifest depends format error")
        }
        let bucket = arr[0].to_string();
        let app_name = arr[1].to_string();
        install_from_specific_bucket(&bucket, &app_name, options)?;
    } else {
        install_app(&depends, options)?;
    }
    Ok(())
}
pub fn handle_arch(arch: &[InstallOptions]) -> anyhow::Result<String> {
    if arch.contains(&InstallOptions::ArchOptions("64bit"))
        || arch.contains(&InstallOptions::ArchOptions("32bit"))
        || arch.contains(&InstallOptions::ArchOptions("arm64"))
    {
        let option_arch = arch
            .iter()
            .map(|option| match option {
                InstallOptions::ArchOptions(arch) => arch,
                _ => "",
            })
            .collect::<Vec<&str>>();
        let option_arch = option_arch[0];
        if option_arch != "64bit" && option_arch != "32bit" && option_arch != "arm64" {
            bail!("选择安装的架构错误 ,(64bit,32bit,arm64)")
        };
        Ok(option_arch.parse()?)
    } else {
        let system_arch = get_system_default_arch()?;
        if system_arch.is_empty() {
            bail!("获取系统默认架构失败")
        }
        Ok(system_arch)
    }
}

pub fn add_scoop_shim_root_dir_to_env_path(options: &Box<[InstallOptions]>) -> anyhow::Result<()> {
    let origin = if options.contains(&InstallOptions::Global) {
        get_system_env_str()
    } else {
        get_user_env_str()
    };

    let scoop_shim_root_dir = if options.contains(&InstallOptions::Global) {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };
    if !origin.contains(&scoop_shim_root_dir) {
        if options.contains(&InstallOptions::Global) {
            let path = origin + ";" + &scoop_shim_root_dir;
            set_global_env_var("Path", path.as_str())?
        } else {
            let path = origin + ";" + &scoop_shim_root_dir;
            set_user_env_var("Path", path.as_str())?
        }
        eprint!("not have  shim root ");
        Ok(())
    } else {
        Ok(())
    }
}

pub fn install_psmodule(
    global: bool,
    psmodule: PSModuleStruct,
    app_name: &str,
    version: &str,
) -> anyhow::Result<()> {
    let psmodule_root_dir = if global {
        get_psmodules_root_global_dir()
    } else {
        get_psmodules_root_dir()
    };
    ensure_in_psmodulepath(&psmodule_root_dir, global)?;
    let app_version_dir = if global {
        get_app_version_dir_global(app_name, version)
    } else {
        get_app_version_dir(app_name, version)
    };
    let module_name = psmodule.name;
    if module_name.is_empty() {
        bail!("module name cannot be empty, manifest format error");
    }
    let link_dir = format!("{}\\{}", psmodule_root_dir, module_name);
    if Path::new(&link_dir).exists() {
        eprintln!(
            "{}",
            format!("{link_dir} is already exists. It will be replaced.")
                .dark_grey()
                .bold()
                .to_string()
        );
        std::fs::remove_dir_all(&link_dir).context(format!(
            "remove old module dir link failed {} at line 178",
            link_dir
        ))?;
    }
    fs::symlink_dir(&app_version_dir, &link_dir)
        .context("Create ps module dir symlink failed at line 180")?;
    println!(
        "{}  {} => {}",
        "Linking".dark_blue().bold(),
        link_dir.dark_green().bold(),
        app_version_dir.dark_green().bold()
    );
    Ok(())
}

fn ensure_in_psmodulepath(psmodule_root_dir: &str, global: bool) -> anyhow::Result<()> {
    let path = if global {
        get_system_env_var("PSModulePath")?
    } else {
        get_user_env_var("PSModulePath")?
    };

    let path = if path.is_empty() && !global {
        let home = std::env::var("USERPROFILE")?;
        format!("{home}\\Documents\\WindowsPowerShell\\Modules")
    } else {
        path
    };
    if path.contains(&psmodule_root_dir) {
        return Ok(());
    }
    let re = Regex::new(&regex::escape(&psmodule_root_dir))?;
    if !re.is_match(&path) {
        println!(
            "Adding {} to {} PowerShell module path.",
            psmodule_root_dir,
            if global { "global" } else { "your" }
        );
        if global {
            set_global_env_var("PSModulePath", &format!("{psmodule_root_dir}\\{path}"))?;
        } else {
            set_user_env_var("PSModulePath", &format!("{psmodule_root_dir}\\{path}"))?;
        }
    }
    Ok(())
}

pub fn create_persist_data_link(
    persist: Option<StringOrArrayOrDoubleDimensionArray>,
    options: &[InstallOptions],
    app_name: &str,
) -> anyhow::Result<()> {
    if persist.is_none() {
        return Ok(());
    }
    let persist = persist.unwrap();
    let global = if options.contains(&InstallOptions::Global) {
        true
    } else {
        false
    };
    match persist {
        StringOrArrayOrDoubleDimensionArray::StringArray(arr) => {
            for item in arr {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                start_create_file_and_dir_link(global, item, app_name, item)?;
            }
        }
        StringOrArrayOrDoubleDimensionArray::Null => {}
        StringOrArrayOrDoubleDimensionArray::String(persist_dir) => {
            let persist_dir = persist_dir.trim();
            if persist_dir.is_empty() {
                return Ok(());
            }
            start_create_file_and_dir_link(global, persist_dir, app_name, persist_dir)?;
        }
        StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(double_arr) => {
            let len = double_arr.len();
            if len == 0 {
                return Ok(());
            }
            for item in double_arr {
                if item.len() == 0 {
                    bail!("persist array format error");
                } else if item.len() == 1 {
                    let [persist, source] =
                        [item.get(0).unwrap().trim(), item.get(0).unwrap().trim()];
                    start_create_file_and_dir_link(global, persist, app_name, source)?;
                } else if item.len() == 2 {
                    let [source, persist] =
                        [item.get(0).unwrap().trim(), item.get(1).unwrap().trim()];
                    start_create_file_and_dir_link(global, persist, app_name, source)?;
                }
            }
        }
        StringOrArrayOrDoubleDimensionArray::NestedStringArray(nested_str_arr) => {
            for item in nested_str_arr {
                match item {
                    StringOrArrayOrDoubleDimensionArray::StringArray(arr) => {
                        let len = arr.len();
                        if len == 0 {
                            continue;
                        } else if len == 1 {
                            let [persist, source] =
                                [arr.get(0).unwrap().trim(), arr.get(0).unwrap().trim()];
                            start_create_file_and_dir_link(global, persist, app_name, source)?;
                        } else if len == 2 {
                            let [source, persist] =
                                [arr.get(0).unwrap().trim(), arr.get(1).unwrap().trim()];
                            start_create_file_and_dir_link(global, persist, app_name, source)?;
                        }
                    }
                    StringOrArrayOrDoubleDimensionArray::String(persist) => {
                        let [persist_target, source] = [persist.trim(), persist.trim()];
                        if persist_target.is_empty() {
                            continue;
                        }
                        start_create_file_and_dir_link(global, persist_target, app_name, source)?;
                    }
                    StringOrArrayOrDoubleDimensionArray::Null => {}
                    StringOrArrayOrDoubleDimensionArray::DoubleDimensionArray(_) => {}
                    StringOrArrayOrDoubleDimensionArray::NestedStringArray(_) => {}
                }
            }
        }
    }
    Ok(())
}

pub fn ensure_directory(target: &str) -> anyhow::Result<()> {
    let path = Path::new(target);
    if !path.exists() {
        std::fs::create_dir_all(path).context(format!(
            "create target directory failed {} at line 307",
            target
        ))?;
    } else if !path.is_dir() {
        // 如果路径存在但不是目录，返回错误
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Path exists but is not a directory",
        )
        .into());
    }

    Ok(())
}

pub fn start_create_file_and_dir_link(
    global: bool,
    persist_dir: &str,
    app_name: &str,
    source_dir: &str,
) -> anyhow::Result<()> {
    let persist_root_dir = if global {
        get_persist_dir_path_global()
    } else {
        get_persist_dir_path()
    };
    let app_current_dir = if global {
        get_app_current_dir_global(app_name)
    } else {
        get_app_current_dir(app_name)
    };
    let target_persist_dir = format!("{persist_root_dir}\\{app_name}\\{persist_dir}");
    let source_dir = format!("{app_current_dir}\\{source_dir}");

    if Path::new(&target_persist_dir).exists() {
        if Path::new(&source_dir).exists() {
            std::fs::rename(&source_dir, format!("{source_dir}.original")).context(format!(
                "rename old source dir failed {} at line 343",
                source_dir
            ))?;
        }
    } else if Path::new(&source_dir).exists() {
        let parent = Path::new(&target_persist_dir).parent().unwrap();
        if !parent.exists() {
            std::fs::create_dir_all(parent).context(format!(
                "create parent directory failed {} at line 351",
                parent.display()
            ))?;
        }
        std::fs::rename(&source_dir, &target_persist_dir)
            .context(format!("move source dir failed {} at line 355", source_dir))?
    } else {
        ensure_directory(&target_persist_dir)?;
    }

    // !create persist data link
    if Path::new(&target_persist_dir).is_dir() {
        fs::symlink_dir(&target_persist_dir, &source_dir).context(format!(
            "create target persisted dir failed {} at line 362",
            target_persist_dir
        ))?;
    } else {
        std::fs::hard_link(&target_persist_dir, &source_dir).context(format!(
            "create target persisted hard file failed {} at line 365",
            target_persist_dir
        ))?;
    }

    Ok(())
}

pub fn install_app_from_url(
    download_url: &Path,
    options: &[InstallOptions<'_>],
    app_alias: Option<String>,
) -> anyhow::Result<()> {
    log::info!("Installing app from url: {}", download_url.display());
    let shim_root = if options.contains(&InstallOptions::Global) {
        get_shims_root_dir_global()
    } else {
        get_shims_root_dir()
    };

    let suffix = download_url
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap();
    log::debug!("suffix is : {}", suffix);
    if suffix.is_empty() {
        bail!("url suffix is empty");
    } else if suffix == "exe" {
        let aria2 = which("aria2c").ok();
        if aria2.is_none() {
            install_app("aria2", options)?;
        } else {
            let download_manager = DownloadManager::new(
                options,
                download_url.to_str().unwrap(),
                Some(download_url.to_str().unwrap()),
            );
            download_manager.start_download()?;
            let exe_name =
                download_manager.copy_file_to_app_dir_from_remote_url(app_alias.clone(), "exe")?;
            download_manager.link_current_from_remote_url()?;
            let app_name = download_manager.get_download_app_name();
            create_default_shim_name_file(exe_name, shim_root.as_str(), app_name, options)?;
        }
    } else if suffix == "bat" || suffix == "cmd" {
        let download_manager = DownloadManager::new(
            options,
            download_url.to_str().unwrap(),
            Some(download_url.to_str().unwrap()),
        );
        download_manager.start_download()?;
        let bat_name = if suffix == "bat" {
            download_manager.copy_file_to_app_dir_from_remote_url(app_alias.clone(), "bat")?
        } else {
            download_manager.copy_file_to_app_dir_from_remote_url(app_alias.clone(), "cmd")?
        };
        download_manager.link_current_from_remote_url()?;
        let app_name = download_manager.get_download_app_name();
        create_default_shim_name_file(bat_name, shim_root.as_str(), app_name, options)?;
    } else if suffix == "ps1" {
        let download_manager = DownloadManager::new(
            options,
            download_url.to_str().unwrap(),
            Some(download_url.to_str().unwrap()),
        );
        download_manager.start_download()?;
        let ps1_name =
            download_manager.copy_file_to_app_dir_from_remote_url(app_alias.clone(), "ps1")?;
        download_manager.link_current_from_remote_url()?;
        let app_name = download_manager.get_download_app_name();
        create_default_shim_name_file(ps1_name, shim_root.as_str(), app_name, options)?;
    } else {
        bail!("Unsupported file type: {}", suffix);
    }
    Ok(())
}

pub fn validate_hash_exists(
    hash: Option<StringArrayOrString>,
    architecture: Option<ArchitectureObject>,
) -> anyhow::Result<bool> {
    if hash.is_some() {
        Ok(true)
    } else if architecture.is_some() {
        let architecture = architecture.unwrap();
        let arch = get_system_default_arch()?;
        let result = architecture.get_specific_architecture(arch.as_str());
        if result.is_some() {
            let result = result.unwrap();
            let hash = result.hash.clone();
            if hash.is_some() { Ok(true) } else { Ok(false) }
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

mod test_installer {
    #[allow(unused)]
    use super::*;

    #[test]
    fn output_env_path() {
        let options = vec![].into_boxed_slice();
        add_scoop_shim_root_dir_to_env_path(&options).unwrap();
    }

    #[test]
    fn test_symbolic_meta() {
        let info = std::fs::read_link(r"A:\Scoop\apps\motrix\current").unwrap();
        println!("{:?}", info.display());
    }

    #[test]
    fn test_path_format() {
        let path = r"A:\Scoop\apps\nodejs\current\.";
        let path = Path::new(path).canonicalize().unwrap();
        println!("{:?}", path.display());
    }
}
