pub mod buckets;
pub mod cat;
pub mod init_env;
pub mod list;
pub mod merge;
pub mod search;
pub mod utils;
use crate::init_env::HyperScoopGlobal;
pub use init_env::HyperScoop;
pub use list::{display_app_info, list_specific_installed_apps};
pub use std::process::exit;

pub mod cache;
pub mod export;
pub mod home;
pub mod info;
pub mod manifest;

pub mod config;
pub mod import;
pub mod install;
pub mod reset;
pub mod shim;
pub mod uninstall;
pub mod update;

pub fn init_hyperscoop() -> Result<HyperScoop, anyhow::Error> {
    let hyperscoop = HyperScoop::new();
    Ok(hyperscoop)
}

pub fn init_hyperscoop_global() -> Result<HyperScoopGlobal, anyhow::Error> {
    let hyperscoop = HyperScoopGlobal::new();
    Ok(hyperscoop)
}
