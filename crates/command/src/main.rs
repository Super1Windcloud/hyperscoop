mod init_env;
use anyhow;
use command_util_lib::init_hyperscoop;
use init_env::HyperScoop;
use std::env;
mod buckets;
fn main() {
  let hyperscoop = init_hyperscoop().expect("Failed to initialize hyperscoop");
  let bucket = buckets::Buckets::new();
}
