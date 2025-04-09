use env_logger;
use env_logger::init;
use crate::Cli;

pub fn init_logger(x: &Cli) {
    if cfg!(debug_assertions) || x.debug {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "error");
    }
    init();
}
