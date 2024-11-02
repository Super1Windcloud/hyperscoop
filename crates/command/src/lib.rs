pub mod buckets;
pub mod init_env;
mod test;

use anyhow;
pub use init_env::HyperScoop;

pub fn init_hyperscoop() -> Result<HyperScoop, anyhow::Error> {
  let hyperscoop = init_env::HyperScoop::new();
  Ok(hyperscoop)
}
