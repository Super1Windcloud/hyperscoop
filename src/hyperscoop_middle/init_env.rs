use anyhow::Error;
use command_util_lib::HyperScoop;
use command_util_lib::init_hyperscoop;
#[allow(dead_code, unused)]
pub fn init_scoop_env_path() -> Result<HyperScoop, Error> {
    init_hyperscoop()
}
