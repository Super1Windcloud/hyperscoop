use crate::manifest::install_manifest::InstallManifest;
use anyhow::{bail, Result};

mod installer;
use crate::init_env::get_app_current_dir;
use crate::manifest::manifest_deserialize::*;
use crate::update::update::check_bucket_update_status;
use crate::utils::system::get_system_default_arch;
pub use installer::*;

pub struct ArchStruct {}
pub async fn install_app_from_local_manifest_file(
    manifest_path: &String,
    arch: Option<String>,
) -> Result<()> {
    log::info!("install from local manifest file {}", manifest_path);
    let mut install_arch = String::new();
    check_bucket_update_status();
    if arch.is_some() {
        let arch = arch.unwrap();
        if (arch != "64bit" && arch != "32bit" && arch != "arm64") {
            bail!("选择安装的架构错误 ,(64bit,32bit,arm64)")
        };
        install_arch = arch
    } else if arch.is_none() {
        install_arch = get_system_default_arch()?;
    }
    let content = std::fs::read_to_string(manifest_path)?;
    let mut serde_obj: InstallManifest = serde_json::from_str(&content)?;
    let name = &serde_obj
        .set_name(manifest_path)
        .get_name()
        .unwrap_or(String::new());
    if name.is_empty() {
        bail!("manifest file name is empty")
    }
    let version = serde_obj.version.clone().unwrap_or(String::new());
    if version.is_empty() {
        bail!("manifest file version is empty")
    }
    let suggest = serde_obj.suggest.clone().unwrap_or(ManifestObj::Null);
    let notes = serde_obj.notes.clone().unwrap_or(Default::default());
    let env_set = serde_obj.env_set.clone().unwrap_or(ManifestObj::Null);
    let env_add_path = serde_obj
        .env_add_path
        .clone()
        .unwrap_or(StringArrayOrString::Null);

    if !env_set.is_null() {
        handle_env_set(&env_set, &serde_obj.clone())?;
    };
    if env_add_path != StringArrayOrString::Null {
        let app_current_dir = get_app_current_dir(name.clone());
        handle_env_add_path(env_add_path, app_current_dir)?;
    }

    if !suggest.is_null() {
        show_suggest(&suggest)?;
    }
    if notes.clone() != StringArrayOrString::default() {
        show_notes(&notes)?;
    }
    Ok(())
}

pub async fn install_from_specific_bucket(bucket_name: &str, app_name: &str) -> Result<()> {
    log::info!("install from specific bucket from {}", bucket_name);
    Ok(())
}

pub async fn install_app_specific_version(app_name: &str, app_version: &str) -> Result<()> {
    log::info!("install from app specific version {}", app_version);
    Ok(())
}

pub async fn install_app(app_name: &str) -> Result<()> {
    log::info!("install from app {}", app_name);
    Ok(())
}
