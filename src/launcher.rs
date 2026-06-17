use std::fs;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use crate::config::config_dir;
use crate::config::config_path;
use crate::config::sanitize_name;
use crate::consts::DEV_DIR;
use crate::consts::DEV_LIB;
use crate::consts::ENV_CONFIG_NAME;
use crate::consts::ENV_PRELOAD;
use crate::consts::LAYER_NAME;
use crate::consts::ENV_VK_ADD_LAYER_PATH;
use crate::consts::ENV_VK_INSTANCE_LAYERS;
use crate::consts::HEAD;
use crate::consts::INSTALL_DIR;
use crate::consts::INSTALL_LIB;
use crate::consts::USAGE;
use crate::logging::init_log_level;
use crate::logging::log_at;
use crate::logging::LogLevel;

pub fn run_launcher(args: Vec<String>) -> i32 {
    init_log_level();
    match wants_help(&args) {
        true => {
            print!("{}", USAGE);
            0
        }
        false => launch(args),
    }
}

fn is_help_flag(a: &str) -> bool {
    a == "--help" || a == "-h"
}

fn wants_help(args: &[String]) -> bool {
    args.iter().take_while(|a| **a != "--").any(|a| is_help_flag(a))
}

fn split_args(args: &[String]) -> (Vec<String>, Vec<String>) {
    let pos = args.iter().position(|a| a == "--").unwrap_or(args.len());
    (args[..pos].to_vec(), args.get(pos + 1..).unwrap_or(&[]).to_vec())
}

fn lib_paths() -> [PathBuf; 2] {
    [PathBuf::from(INSTALL_LIB), PathBuf::from(DEV_LIB)]
}

fn dir_paths() -> [PathBuf; 2] {
    [PathBuf::from(INSTALL_DIR), PathBuf::from(DEV_DIR)]
}

fn first_existing(search: &[PathBuf]) -> Option<PathBuf> {
    search.iter().find(|p| p.exists()).cloned()
}

fn resolve(search: &[PathBuf], fallback: &str) -> String {
    first_existing(search)
        .map(|p| p.canonicalize().unwrap_or(p))
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| fallback.into())
}

fn launch(args: Vec<String>) -> i32 {
    let (head, cmd) = split_args(&args);
    match cmd.is_empty() {
        true => {
            print!("{}", USAGE);
            1
        }
        false => launch_cmd(&head, &cmd),
    }
}

fn launch_cmd(head: &[String], cmd: &[String]) -> i32 {
    let profile = sanitize_name(head.first().map(String::as_str).unwrap_or("bones"));
    ensure_profile_config(&profile);
    exec_child(cmd, &profile, &resolve(&lib_paths(), INSTALL_LIB), &resolve(&dir_paths(), INSTALL_DIR))
}

fn ensure_profile_config(profile: &str) {
    let path = config_path(profile);
    let _ = fs::create_dir_all(config_dir());
    match path.exists() {
        true => (),
        false => {
            let _ = fs::write(&path, HEAD);
        }
    }
}

fn exec_child(full: &[String], profile: &str, preload: &str, layer_dir: &str) -> i32 {
    let err = Command::new(&full[0])
        .args(&full[1..])
        .env(ENV_CONFIG_NAME, profile)
        .env(ENV_PRELOAD, preload)
        .env(ENV_VK_ADD_LAYER_PATH, layer_dir)
        .env(ENV_VK_INSTANCE_LAYERS, LAYER_NAME)
        .exec();
    log_at(LogLevel::Error, &format!("exec failed: {}", err));
    127
}
