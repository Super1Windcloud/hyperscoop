use env_logger;
use env_logger::init;
use crate::Cli;
use  crossterm::style::force_color_output;
pub fn init_logger(x: &Cli) {
    if cfg!(debug_assertions) || x.debug {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "error");
    }
    init(); 
 
}

pub fn init_color_output(no_color : bool) {
   if  no_color {
      force_color_output(false); 
   }
}