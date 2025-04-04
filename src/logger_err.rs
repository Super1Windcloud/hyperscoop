use env_logger;
use env_logger::init;
#[allow(unused_imports)]
use log::{debug, error, info, trace};
use crate::Cli;

pub fn init_logger(x: &Cli) {
    // 初始化日志
    if cfg!(debug_assertions) || x.debug {
        // dev 模式：启用所有日志级别
        std::env::set_var("RUST_LOG", "trace");
    } else {
        // release 模式：仅启用 error 级别
        std::env::set_var("RUST_LOG", "error");
    }
    init();
}
