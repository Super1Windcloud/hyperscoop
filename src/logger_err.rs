use crate::Cli;
use command_util_lib::utils::system::{is_admin, request_admin};
use crossterm::style::{force_color_output, Stylize};
use env_logger;
use env_logger::init;
use std::env;

pub unsafe fn init_logger(x: &Cli) {
    if (cfg!(debug_assertions) || x.debug) && !x.error {
        env::set_var("RUST_LOG", "debug");
    } else {
        env::set_var("RUST_LOG", "error");
    }
    init();
}

pub fn init_color_output(no_color: bool) {
    if no_color {
        force_color_output(false);
    }
}

// 主函数入口调用提权, 管理员进程会自动闪退不知道为什么
#[allow(dead_code)]
pub fn invoke_admin_process() -> anyhow::Result<()> {
    if !is_admin()? {
        let args = env::args().skip(1).collect::<Vec<String>>();
        let args_str = args.join(" ");
        log::warn!(
            "Global command arguments: {}",
            args_str.clone().dark_yellow()
        );
        request_admin(args_str.as_str())?;
        return Ok(());
    }

    Ok(())
}
