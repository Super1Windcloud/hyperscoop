use crate::manifest::install_manifest::InstallManifest;
use anyhow::{bail, Context, Result};
use crossterm::style::Stylize;
use rayon::prelude::*;
use std::path::Path;

pub mod installer;
use crate::init_env::{
    check_bucket_whether_exists, get_app_current_dir, get_app_dir, get_app_dir_global,
    get_special_bucket_all_manifest_path, get_special_bucket_all_manifest_path_global,
    get_special_version_all_manifest_path, get_special_version_all_manifest_path_global,
};
use crate::manifest::manifest_deserialize::*;
pub use installer::*;
pub mod check;
pub mod shim_and_shortcuts;
pub use check::*;
pub use shim_and_shortcuts::*;
pub mod cli_options_store;
pub use cli_options_store::*;
pub mod aria2;
pub use aria2::*;
pub mod sevenzip;
pub use sevenzip::*;
pub mod download;
pub mod parse_lifecycle_scripts;
pub use parse_lifecycle_scripts::*;
pub mod env_operate;
use crate::list::VersionJSON;
use crate::manifest::manifest::{
    get_best_manifest_from_local_bucket, get_best_manifest_from_local_bucket_global,
    get_latest_manifest_from_local_bucket, get_latest_manifest_from_local_bucket_global,
};
use crate::utils::system::{ensure_persist_permission, kill_processes_using_app};
use crate::utils::utility::{nightly_version, validate_version};
pub use download::*;
pub use env_operate::*;

/// 下载, 解压, preinstall, create_shim_shortcut, postinstall
pub fn install_app_from_local_manifest_file<P: AsRef<Path>>(
    manifest_path: P,
    options: Vec<InstallOptions<'_>>,
    bucket_source: Option<&str>,
) -> Result<()> {
    let options: Box<[InstallOptions]> = options.into_boxed_slice();
    add_scoop_shim_root_dir_to_env_path(&options).expect("add scoop shim root to env path failed");

    let manifest_path = manifest_path.as_ref().to_str().unwrap();
    let install_arch = handle_arch(&options)?;
    log::info!("install arch: {}", install_arch);
    let content = std::fs::read_to_string(&manifest_path).context(format!(
        "read manifest file '{}' failed at line 51",
        manifest_path
    ))?;
    let mut serde_obj: InstallManifest = serde_json::from_str(&content).context(format!(
        "deserialize manifest file '{}' failed at line 53",
        manifest_path
    ))?;
    let app_name = serde_obj
        .set_name(&manifest_path)
        .get_name()
        .unwrap_or(String::new());
    let obj_copy = serde_obj.clone();
    if app_name.is_empty() {
        bail!("manifest file name is empty")
    }
    let version = &serde_obj.version.unwrap_or(String::new());
    if version.is_empty() {
        bail!("manifest file version is empty")
    }
    if options.contains(&InstallOptions::ForceInstallOverride) {
        let special_app_dir = if options.contains(&InstallOptions::Global) {
            get_app_dir_global(&app_name)
        } else {
            get_app_dir(&app_name)
        };

        if Path::new(&special_app_dir).exists() && app_name.to_lowercase() != "hp" {
            let result = std::fs::remove_dir_all(&special_app_dir);
            match result {
                Ok(_) => {
                    log::debug!("remove app dir success!");
                }
                Err(_) => {
                    log::debug!("kill {app_name}  process");
                    kill_processes_using_app(&app_name);
                    std::fs::remove_dir_all(&special_app_dir).context(format!(
                        "remove app dir '{}' failed at line 83",
                        special_app_dir
                    ))?;
                }
            }
        }
    }
    validate_version(version)?;

    let version = if version == "nightly" {
        &nightly_version()?
    } else {
        version
    };

    let options = if version == "nightly" {
        options
            .to_vec()
            .into_iter()
            .chain(vec![InstallOptions::SkipDownloadHashCheck])
            .chain(vec![InstallOptions::CurrentInstallApp {
                app_name: app_name.to_string(),
                app_version: version.to_string(),
            }])
            .collect::<Box<[InstallOptions]>>()
    } else {
        options
            .to_vec()
            .into_iter()
            .chain(vec![InstallOptions::CurrentInstallApp {
                app_name: app_name.to_string(),
                app_version: version.to_string(),
            }]) 
            .collect::<Box<[InstallOptions]>>()
    };

    let result = if !options.contains(&InstallOptions::ForceDownloadNoInstallOverrideCache)
        && !options.contains(&InstallOptions::OnlyDownloadNoInstall)
        && !options.contains(&InstallOptions::ForceInstallOverride)
        && !options.contains(&InstallOptions::UpdateTransaction)
        && !options.contains(&InstallOptions::InstallSpecialVersionApp)
    {
        match check_before_install(&app_name, &version, &options) {
            Ok(result) => result,
            Err(e) => {
                eprintln!(
                    "{}",
                    format!("check before install failed, Error message:\n{}", e)
                        .dark_red()
                        .bold()
                );
                let special_app_dir = if options.contains(&InstallOptions::Global) {
                    get_app_dir_global(&app_name)
                } else {
                    get_app_dir(&app_name)
                };
                log::debug!("remove app dir: {}", special_app_dir);
                if Path::new(&special_app_dir).exists() && app_name.to_lowercase() != "hp" {
                    let result = std::fs::remove_dir_all(&special_app_dir);
                    match result {
                        Ok(_) => {
                            log::debug!("remove app dir success!");
                        }
                        Err(_) => {
                            log::debug!("kill {app_name}  process");
                            kill_processes_using_app(&app_name);
                            std::fs::remove_dir_all(&special_app_dir).context(format!(
                                "After kill process ,still remove app dir '{}' failed at line 142",
                                special_app_dir
                            ))?;
                        }
                    }
                }
                0
            }
        }
    } else {
        0
    };
    if result != 0 {
        return Ok(());
    };
    let end_message = if bucket_source.is_none() {
        format!("from manifest file '{}'", manifest_path)
    } else {
        format!("from bucket '{}'", bucket_source.clone().unwrap())
    };

    println!(
        "{}",
        format!("Installing '{app_name}' ({version}) [{install_arch}] {end_message}")
            .bold()
            .dark_green()
    );

    let depends = serde_obj.depends;
    let suggest = serde_obj.suggest;
    let notes = serde_obj.notes;
    let env_set = serde_obj.env_set;
    let env_add_path = serde_obj.env_add_path;
    // let url = serde_obj.url;
    // let hash = serde_obj.hash;
    // let installer = serde_obj.installer;
    // let shortcuts = serde_obj.shortcuts;
    let architecture = serde_obj.architecture;
    // let bin = serde_obj.bin;
    let extract_dir = serde_obj.extract_dir;
    let extract_to = serde_obj.extract_to;
    // let innosetup = serde_obj.innosetup;
    let persist = serde_obj.persist;
    let psmodule = serde_obj.psmodule;
    // let pre_install = serde_obj.pre_install;
    // let post_install = serde_obj.post_install;

    if !depends.is_none() && !options.contains(&InstallOptions::NoAutoDownloadDepends) {
        let depend = depends.unwrap();
        match depend {
            StringArrayOrString::StringArray(array) => {
                for depend in array {
                    handle_depends(depend, options.as_ref())?;
                }
            }
            StringArrayOrString::String(_depend) => {
                handle_depends(_depend, options.as_ref())?;
            }
            StringArrayOrString::Null => {}
        }
    }

    //   **invoke aria2  to  download  file to cache
    let download_manager = DownloadManager::new(&options, &manifest_path, bucket_source);
    download_manager.start_download()?;
    if !options.contains(&InstallOptions::SkipDownloadHashCheck) {
        download_manager.check_cache_file_hash()?
    }
    if options.contains(&InstallOptions::OnlyDownloadNoInstall) {
        return Ok(());
    }
    //  * 提取 cache 中的zip 到 app dir
    let senvenzip =
        download_manager.invoke_7z_extract(extract_dir, extract_to, architecture.clone())?;
    senvenzip.link_current()?;
    // !  parse    pre_install
    parse_lifecycle_scripts(
        LifecycleScripts::PreInstall,
        manifest_path,
        &options,
        &app_name,
        None,
    )
    .expect("parse pre_install failed");

    // !  parse    manifest installer
    parse_lifecycle_scripts(
        LifecycleScripts::Installer,
        manifest_path,
        &options,
        &app_name,
        None,
    )
    .expect("parse installer scripts failed");

    //  ? linking   app current dir to app version dir
    // senvenzip.link_current()?;
    //*create_shims
    //*create_startmenu_shortcuts
    create_shim_or_shortcuts(manifest_path, &app_name, &options)
        .expect("create shim or shortcuts failed");

    // * install_psmodule
    if psmodule.is_some() {
        let psmodule = psmodule.unwrap();
        let global = if options.contains(&InstallOptions::Global) {
            true
        } else {
            false
        };
        install_psmodule(global, psmodule, &app_name, version).expect("install_psmodule failed");
    }
    if !env_set.is_none() {
        handle_env_set(env_set.unwrap(), obj_copy, &options)?;
    };
    if env_add_path.is_some() {
        let env_add_path = env_add_path.unwrap();
        if env_add_path != StringArrayOrString::Null {
            let app_current_dir = get_app_current_dir(&app_name);
            handle_env_add_path(env_add_path, app_current_dir, &options)?;
        }
    }
    // ! linking  persist_data  链接 Persist 目录
    create_persist_data_link(persist.clone(), &options, &app_name)
        .expect("create persist link failed");

    //*persist_permission  主要用于 设置文件系统权限，确保特定用户（通常是 "Users" 组）对某个目录具有写入权限。
    if persist.is_some() {
        let global = if options.contains(&InstallOptions::Global) {
            true
        } else {
            false
        };
        if global {
            ensure_persist_permission().expect("persist dir check failed");
        }
    }
    // !   parse post_install
    parse_lifecycle_scripts(
        LifecycleScripts::PostInstall,
        &manifest_path,
        &options,
        &app_name,
        None,
    )
    .expect("parse post_install failed");
    //*  save  install.json , manifest.json  to app version dir
    download_manager
        .save_install_info()
        .expect("save install info failed");
    if !suggest.is_none() {
        show_suggest(&suggest.unwrap())?;
    }
    println!(
        "{}",
        format!("'{app_name}' ({version}) was installed successfully!")
            .dark_green()
            .bold()
    );
    if notes.is_some() {
        let notes = notes.unwrap();
        if notes != StringArrayOrString::Null {
            show_notes(notes)?;
        }
    }
    Ok(())
}

pub fn install_from_specific_bucket(
    bucket_name: &str,
    app_name: &str,
    options: &[InstallOptions<'_>],
) -> Result<()> {
    log::info!("install from specific bucket from {}", bucket_name);

    if !check_bucket_whether_exists(bucket_name, options)? {
        bail!("bucket '{}' not exists,please check it!", bucket_name)
    }
    let all_manifests = if options.contains(&InstallOptions::Global) {
        get_special_bucket_all_manifest_path_global(bucket_name)?
    } else {
        get_special_bucket_all_manifest_path(bucket_name)?
    };
    if all_manifests.is_empty() {
        bail!("No manifest found for '{app_name}' please check it!");
    }

    let manifest_path = all_manifests.par_iter().find_any(|path| {
        let file_name = path.file_stem().unwrap().to_str().unwrap();
        file_name.to_lowercase() == app_name.to_lowercase()
    });
    if manifest_path.is_none() {
        bail!("No manifest found for '{app_name}' please check it!");
    }
    let manifest_path = manifest_path.unwrap();

    log::debug!("manifest path: {}", manifest_path.display());
    let options = options
        .to_vec()
        .into_iter()
        .chain([InstallOptions::InstallSpecialBucketApp])
        .collect();

    install_app_from_local_manifest_file(manifest_path, options, Some(bucket_name))?;
    Ok(())
}

pub async fn install_app_specific_version(
    app_name: &str,
    app_version: &str,
    options: &Vec<InstallOptions<'_>>,
) -> Result<()> {
    let all_manifests = if options.contains(&InstallOptions::Global) {
        get_special_version_all_manifest_path_global()?
    } else {
        get_special_version_all_manifest_path()?
    };
    if all_manifests.is_empty() {
        bail!("No manifest found for '{app_name}' please check it!");
    }
    let manifest_paths = all_manifests
        .par_iter()
        .filter_map(|path| {
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            if file_name.to_lowercase() == app_name.to_lowercase() {
                Some(path.as_path())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if manifest_paths.is_empty() {
        bail!("App '{}' not found,check it!", app_name)
    }

    let special_version_manifests = manifest_paths
        .into_iter()
        .filter_map(|path| {
            let content = std::fs::read_to_string(path)
                .expect(format!("文件编码格式错误, 检查{}", path.display()).as_ref());
            let version: VersionJSON = serde_json::from_str(&content)
                .expect(format!("JSON格式错误, 检查{}", path.display()).as_str());
            let version = version
                .version
                .expect(format!("Version为空,检查{}", path.display()).as_ref());
            if !version.is_empty() && version.to_lowercase() == app_version {
                Some(path.to_str().unwrap().to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let special_version_manifest = if special_version_manifests.iter().any(|path| {
        let path = Path::new(path);
        let parent = path.parent().unwrap().parent().unwrap();
        let bucket_name = parent.file_name().unwrap().to_str().unwrap();
        bucket_name.contains("main")
    }) {
        special_version_manifests.iter().find(|path| {
            let path = Path::new(path);
            let parent = path.parent().unwrap().parent().unwrap();
            let bucket_name = parent.file_name().unwrap().to_str().unwrap();
            bucket_name.contains("main")
        })
    } else if special_version_manifests.iter().any(|path| {
        let path = Path::new(path);
        let parent = path.parent().unwrap().parent().unwrap();
        let bucket_name = parent.file_name().unwrap().to_str().unwrap();
        bucket_name.contains("extras")
    }) {
        special_version_manifests.iter().find(|path| {
            let path = Path::new(path);
            let parent = path.parent().unwrap().parent().unwrap();
            let bucket_name = parent.file_name().unwrap().to_str().unwrap();
            bucket_name.contains("extras")
        })
    } else if special_version_manifests.iter().any(|path| {
        let path = Path::new(path);
        let parent = path.parent().unwrap().parent().unwrap();
        let bucket_name = parent.file_name().unwrap().to_str().unwrap();
        bucket_name.contains("versions")
    }) {
        special_version_manifests.iter().find(|path| {
            let path = Path::new(path);
            let parent = path.parent().unwrap().parent().unwrap();
            let bucket_name = parent.file_name().unwrap().to_str().unwrap();
            bucket_name.contains("versions")
        })
    } else {
        special_version_manifests.get(0)
    };
    if special_version_manifest.is_none() {
        bail!(
            "app '{}' version '{}' not found ,check it!",
            app_name,
            app_version
        )
    }
    let special_version_manifest = special_version_manifest.unwrap();
    let source_bucket = (|| {
        let path = Path::new(special_version_manifest);
        let parent = path.parent().unwrap().parent().unwrap();
        parent.file_name().unwrap().to_str().unwrap()
    })();
    log::info!("manifest path: {}", special_version_manifest);

    let options = options
        .to_vec()
        .into_iter()
        .chain([InstallOptions::InstallSpecialVersionApp])
        .collect();
    install_app_from_local_manifest_file(special_version_manifest, options, Some(source_bucket))?;
    Ok(())
}

pub fn install_app(app_name: &str, options: &[InstallOptions<'_>]) -> Result<()> {
    log::info!("install from app {}", app_name);
    if app_name.to_lowercase() == "hp" {
        bail!("Update self please use `hp u hp` or `hp u -f -k hp`")
    }
    let manifest_path = if options.contains(&InstallOptions::Global) {
        if options.contains(&InstallOptions::UpdateTransaction) {
            let result = get_latest_manifest_from_local_bucket_global(app_name);
            if result.is_err() {
                let result = get_latest_manifest_from_local_bucket(app_name)?;
                result
            } else {
                result?
            }
        } else {
            let result = get_best_manifest_from_local_bucket_global(app_name);
            if result.is_err() {
                get_best_manifest_from_local_bucket(app_name)?
            } else {
                result?
            }
        }
    } else if options.contains(&InstallOptions::UpdateTransaction) {
        get_latest_manifest_from_local_bucket(app_name)?
    } else {
        get_best_manifest_from_local_bucket(app_name)?
    };

    let duplicate = manifest_path.clone();
    if !duplicate.exists() {
        bail!("No app manifest found for '{app_name}', please check it!")
    }
    log::info!("manifest path: {}", manifest_path.display());
    let source_bucket = (|| {
        let parent = duplicate.parent().unwrap().parent().unwrap();
        parent.file_name().unwrap().to_str().unwrap()
    })();

    install_app_from_local_manifest_file(manifest_path, options.to_vec(), Some(source_bucket))?;

    Ok(())
}

pub async fn install_and_replace_hp(options: &[InstallOptions<'_>]) -> Result<String> {
    let app_name = "hp";

    let manifest_path = if options.contains(&InstallOptions::Global) {
        get_best_manifest_from_local_bucket_global(app_name)?
    } else {
        get_best_manifest_from_local_bucket(app_name)?
    };

    let duplicate = manifest_path.clone();
    if !duplicate.exists() {
        bail!("No app manifest found for '{app_name}', please check it!")
    }
    log::info!("manifest path: {}", manifest_path.display());
    let source_bucket = (|| {
        let parent = duplicate.parent().unwrap().parent().unwrap();
        parent.file_name().unwrap().to_str().unwrap()
    })();
    let version: VersionJSON = serde_json::from_str(
        &std::fs::read_to_string(&manifest_path)
            .context("failed to read hp manifest version from local bucket")?,
    )
    .expect("JSON格式错误,检查hp.json");

    install_app_from_local_manifest_file(
        manifest_path.as_path(),
        options.to_vec(),
        Some(source_bucket),
    )?;
    if version.version.is_none() {
        bail!("hp version is empty")
    }
    Ok(version.version.unwrap())
}
