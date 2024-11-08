#![deny(clippy::shadow)]
pub mod buckets;
pub mod init_env;
pub mod utils;
pub mod merge;
pub mod list;
pub mod search;

pub use list::{display_app_info, list_specific_installed_apps};
pub use std::process::exit;
use anyhow;
pub use init_env::HyperScoop;
// pub use manifest::search_manifest;
pub mod manifest;


pub fn init_hyperscoop() -> Result<HyperScoop, anyhow::Error> {
  let hyperscoop = init_env::HyperScoop::new();
  Ok(hyperscoop)
}
