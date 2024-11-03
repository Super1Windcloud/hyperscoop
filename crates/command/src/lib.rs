pub mod buckets;
pub mod init_env;
mod test;
mod utils;
pub mod merge;
pub use std::process::exit;
use anyhow;
pub use init_env::HyperScoop;

pub fn init_hyperscoop() -> Result<HyperScoop, anyhow::Error> {
  let hyperscoop = init_env::HyperScoop::new();
  Ok(hyperscoop)
}
