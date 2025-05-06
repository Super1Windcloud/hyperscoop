use crate::init_env::{
    get_app_current_dir, get_app_current_dir_global, get_app_version_dir,
    get_app_version_dir_global, get_old_scoop_dir, get_persist_dir_path,
    get_persist_dir_path_global, get_psmodules_root_dir, get_psmodules_root_global_dir,
    get_scoop_cfg_path, get_shims_root_dir, get_shims_root_dir_global, init_scoop_global,
    init_user_scoop,
};
use crate::install::{
    create_default_shim_name_file, install_app, install_from_specific_bucket, DownloadManager,
    InstallOptions,
};
use crate::manifest::install_manifest::{InstallManifest, SuggestObj, SuggestObjValue};
use crate::manifest::manifest_deserialize::{
    ManifestObj, PSModuleStruct, StringArrayOrString, StringOrArrayOrDoubleDimensionArray,
};
use crate::utils::system::{
    get_system_default_arch, get_system_env_str, get_system_env_var, get_user_env_str,
    get_user_env_var, set_global_env_var, set_user_env_var,
};
use anyhow::bail;
use crossterm::style::Stylize;
use regex::Regex;
use std::os::windows::fs;
use std::path::Path;
use which::which;
use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;
use winreg::RegKey;
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

pub fn handle_depends(depends: &str, options: &[InstallOptions<'_>]) -> anyhow::Result<()> {
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

pub fn handle_env_set(
    env_set: ManifestObj,
    manifest: InstallManifest,
    options: &Box<[InstallOptions]>,
) -> anyhow::Result<()> {
    let app_name = manifest.name.unwrap_or(String::new());
    let app_version = manifest.version.unwrap_or(String::new());
    let scoop_home = if options.contains(&InstallOptions::Global) {
        init_scoop_global()
    } else {
        init_user_scoop()
    };
    let global_scoop_home = init_scoop_global();

    let app_dir = format!(
        r#"function app_dir($other_app) {{
      return  "{scoop_home}\apps\$other_app\current" ;
  }}"#
    );
    let old_scoop_dir = get_old_scoop_dir();
    let cfg_path = get_scoop_cfg_path();
    let injects_var = format!(
        r#"
      $app = "{app_name}" ;
      $version = "{app_version}" ;
      $cmd ="uninstall" ;
      $global = $false  ;
      $scoopdir ="{scoop_home}" ;
      $dir = "{scoop_home}\apps\$app\current" ;
      $globaldir  ="{global_scoop_home}";
      $oldscoopdir  = "{old_scoop_dir}" ;
      $original_dir = "{scoop_home}\apps\$app\$version";
      $modulesdir  = "{scoop_home}\modules";
      $cachedir  =  "{scoop_home}\cache";
      $bucketsdir  = "{scoop_home}\buckets";
      $persist_dir  = "{scoop_home}\persist\$app";
      $cfgpath   ="{cfg_path}" ;
  "#
    );

    if let serde_json::Value::Object(env_set) = env_set {
        for (key, env_value) in env_set {
            let mut env_value = env_value.to_string().trim().to_string();
            if env_value.is_empty() {
                continue;
            }
            if env_value.contains('/') {
                env_value = env_value.replace('/', r"\");
            }
            if env_value.contains(r"\\") {
                env_value = env_value.replace(r"\\", r"\");
            }
            let cmd = format!(
                r#"Set-ItemProperty -Path "HKCU:\Environment" -Name "{key}" -Value {env_value}"#
            );

            let output = std::process::Command::new("powershell")
                .arg("-Command")
                .arg(&app_dir)
                .arg(&injects_var)
                .arg(cmd)
                .output()?;
            if !output.status.success() {
                let error_output = String::from_utf8_lossy(&output.stderr);
                bail!(
                    "powershell failed to remove environment variable: {}",
                    error_output
                );
            }

            println!(
                "{} {}",
                "Env set successfully for".to_string().dark_green().bold(),
                key.to_string().dark_cyan().bold(),
            );
        }
    }
    Ok(())
}

pub fn handle_env_add_path(
    env_add_path: StringArrayOrString,
    app_current_dir: String,
    options: &Box<[InstallOptions]>,
) -> anyhow::Result<()> {
    let app_current_dir = app_current_dir.replace('/', r"\");
    if let StringArrayOrString::StringArray(paths) = env_add_path {
        for path in paths {
            add_bin_to_path(path.as_ref(), &app_current_dir, options)?;
        }
    } else if let StringArrayOrString::String(path) = env_add_path {
        add_bin_to_path(path.as_ref(), &app_current_dir, options)?;
    }

    Ok(())
}

pub fn add_bin_to_path(
    path: &str,
    app_current_dir: &String,
    options: &Box<[InstallOptions]>,
) -> anyhow::Result<()> {
    let path = path.replace('/', r"\");
    let path = path.replace('\\', r"\");
    let path = format!(r"{app_current_dir}\{path}");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment_key = hkcu.open_subkey("Environment")?;
    let user_path: String = environment_key.get_value("PATH")?;

    let user_path = format!("{user_path};{path}");
    log::debug!("\n 更新后的用户的 PATH: {}", user_path);

    let script =
        format!(r#"[System.Environment]::SetEnvironmentVariable("PATH","{user_path}", "Machine")"#);
    if options.contains(&InstallOptions::Global) {
        let output = std::process::Command::new("powershell")
            .arg("-Command")
            .arg(script)
            .output()?;
        if !output.status.success() {
            bail!("Failed to remove path var");
        }
        Ok(())
    } else {
        set_user_env_var("Path", &user_path)?;
        Ok(())
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
        std::fs::remove_dir_all(&link_dir)?;
    }
    fs::symlink_dir(&app_version_dir, &link_dir).expect("Create dir symlink failed");
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

pub fn ensure_directory(target: &str) -> std::io::Result<()> {
    let path = Path::new(target);
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    } else if !path.is_dir() {
        // 如果路径存在但不是目录，返回错误
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Path exists but is not a directory",
        ));
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
            std::fs::rename(&source_dir, format!("{source_dir}.original"))?
        }
    } else if Path::new(&source_dir).exists() {
        let parent = Path::new(&target_persist_dir).parent().unwrap();
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&source_dir, &target_persist_dir)?
    } else {
        ensure_directory(&target_persist_dir)?;
    }

    // create link
    if Path::new(&target_persist_dir).is_dir() {
        fs::symlink_dir(target_persist_dir, &source_dir)?;
    } else {
        std::fs::hard_link(target_persist_dir, &source_dir)?;
    }

    Ok(())
}

pub fn install_app_from_url(
    download_url: &Path,
    options: &[InstallOptions],
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
}
