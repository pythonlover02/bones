use std::env;
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
use crate::consts::ENV_SEP;
use crate::consts::ENV_VK_ADD_LAYER_PATH;
use crate::consts::ENV_VK_INSTANCE_LAYERS;
use crate::consts::FLATPAK_CMD;
use crate::consts::FLATPAK_INFO;
use crate::consts::FLATPAK_INJECT;
use crate::consts::FLATPAK_META_KEY;
use crate::consts::FLATPAK_RUN;
use crate::consts::FLATPAK_SHOW_META;
use crate::consts::HEAD;
use crate::consts::INSTALL_DIR;
use crate::consts::INSTALL_LIB;
use crate::consts::LAYER_NAME;
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

fn is_flatpak_bin(name: &str) -> bool {
    name == FLATPAK_CMD || name.ends_with("/flatpak")
}

fn is_flatpak_run(cmd: &[String]) -> bool {
    cmd.first().map(|s| is_flatpak_bin(s)).unwrap_or(false)
        && cmd.iter().any(|a| a == FLATPAK_RUN)
}

fn flatpak_run_pos(cmd: &[String]) -> Option<usize> {
    cmd.iter().position(|a| a == FLATPAK_RUN)
}

fn flatpak_user_flags(cmd: &[String]) -> Vec<String> {
    flatpak_run_pos(cmd)
        .map(|pos| {
            cmd[pos + 1..]
                .iter()
                .take_while(|a| a.starts_with('-'))
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn flatpak_app_pos(cmd: &[String]) -> Option<usize> {
    flatpak_run_pos(cmd).and_then(|pos| {
        cmd[pos + 1..]
            .iter()
            .position(|a| !a.starts_with('-'))
            .map(|rel| pos + 1 + rel)
    })
}

fn flatpak_app_id(cmd: &[String]) -> Option<String> {
    flatpak_app_pos(cmd).map(|p| cmd[p].clone())
}

fn flatpak_trailing(cmd: &[String]) -> Vec<String> {
    flatpak_app_pos(cmd)
        .map(|p| cmd.get(p + 1..).unwrap_or(&[]).to_vec())
        .unwrap_or_default()
}

fn parse_meta_command(text: &str) -> Option<String> {
    text.lines()
        .find(|l| l.starts_with(FLATPAK_META_KEY))
        .map(|l| l[FLATPAK_META_KEY.len()..].trim().to_string())
}

fn call_flatpak_meta(app_id: &str) -> Option<String> {
    Command::new(FLATPAK_CMD)
        .args([FLATPAK_INFO, FLATPAK_SHOW_META, app_id])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|t| parse_meta_command(&t))
}

fn app_command(app_id: &str) -> String {
    call_flatpak_meta(app_id).unwrap_or_else(|| app_id.to_string())
}

fn build_flatpak_args(profile: &str, app_id: &str, app_cmd: &str, flags: &[String], trailing: &[String]) -> Vec<String> {
    [
        vec![
            FLATPAK_RUN.to_string(),
            format!("--command={}", FLATPAK_INJECT),
            format!("--env={}={}", ENV_CONFIG_NAME, profile),
        ],
        flags.to_vec(),
        vec![app_id.to_string(), app_cmd.to_string()],
        trailing.to_vec(),
    ]
    .concat()
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
    match is_flatpak_run(cmd) {
        true => exec_flatpak(cmd, &profile),
        false => exec_native(cmd, &profile),
    }
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

fn exec_flatpak(cmd: &[String], profile: &str) -> i32 {
    match flatpak_app_id(cmd) {
        None => {
            log_at(LogLevel::Error, "flatpak run: no app id found");
            1
        }
        Some(app_id) => {
            let args = build_flatpak_args(
                profile,
                &app_id,
                &app_command(&app_id),
                &flatpak_user_flags(cmd),
                &flatpak_trailing(cmd),
            );
            let err = Command::new(FLATPAK_CMD).args(&args).exec();
            log_at(LogLevel::Error, &format!("exec failed: {}", err));
            127
        }
    }
}

fn val_in_list(existing: &str, val: &str, sep: char) -> bool {
    existing.split(sep).any(|s| s == val)
}

fn join_env(val: &str, prev: &str, sep: char) -> String {
    match val_in_list(prev, val, sep) {
        true => prev.to_string(),
        false => format!("{}{}{}", val, sep, prev),
    }
}

fn prepend_env(val: &str, existing: Option<String>, sep: char) -> String {
    match existing.filter(|s| !s.is_empty()) {
        Some(prev) => join_env(val, &prev, sep),
        None => val.to_string(),
    }
}

fn exec_native(cmd: &[String], profile: &str) -> i32 {
    exec_child(cmd, profile, &resolve(&lib_paths(), INSTALL_LIB), &resolve(&dir_paths(), INSTALL_DIR))
}

fn exec_child(full: &[String], profile: &str, preload: &str, layer_dir: &str) -> i32 {
    let err = Command::new(&full[0])
        .args(&full[1..])
        .env(ENV_CONFIG_NAME, profile)
        .env(ENV_PRELOAD, prepend_env(preload, env::var(ENV_PRELOAD).ok(), ENV_SEP))
        .env(ENV_VK_ADD_LAYER_PATH, prepend_env(layer_dir, env::var(ENV_VK_ADD_LAYER_PATH).ok(), ENV_SEP))
        .env(ENV_VK_INSTANCE_LAYERS, prepend_env(LAYER_NAME, env::var(ENV_VK_INSTANCE_LAYERS).ok(), ENV_SEP))
        .exec();
    log_at(LogLevel::Error, &format!("exec failed: {}", err));
    127
}
