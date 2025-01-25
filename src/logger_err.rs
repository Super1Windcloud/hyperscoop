use env_logger;
#[allow(unused_imports)]
use log::{debug, error, info, trace};
pub fn init_logger() {
    // 初始化日志
    if cfg!(debug_assertions) {
        // dev 模式：启用所有日志级别
        std::env::set_var("RUST_LOG", "trace");
    } else {
        // release 模式：仅启用 error 级别
        std::env::set_var("RUST_LOG", "error");
    }

    env_logger::init();
}
