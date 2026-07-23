#![allow(non_snake_case)]

mod consts;
mod logging;
mod util;
mod timing;
mod env;
mod config;
mod effect;
mod shader;
mod watch;
mod launcher;
mod vulkan;

pub use env::process_args;
pub use launcher::run_launcher;
