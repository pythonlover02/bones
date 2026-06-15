use std::fs;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use crate::config::config_dir;
use crate::config::config_path;
use crate::config::sanitize_name;
use crate::consts::HEAD;
use crate::consts::ARCH_SO;
use crate::consts::LAYER_NAME;
use crate::consts::MANIFEST_JSON;
use crate::consts::USAGE;
use crate::logging::init_log_level;
use crate::logging::log_at;
use crate::logging::LogLevel;

struct StagePlan {
    src: Option<PathBuf>,
    dest: PathBuf,
    manifest: String,
    need_copy: bool,
}

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
    stage_runtime();
    let rt = config_dir().join("runtime");
    let preload = locate_lib(&lib_search_paths())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| rt.join(ARCH_SO).to_string_lossy().into_owned());
    exec_child(cmd, &profile, &preload, &rt)
}

fn needs_preload(preload: &str) -> bool {
    !preload.is_empty()
}

fn exec_child(full: &[String], profile: &str, preload: &str, rt: &PathBuf) -> i32 {
    let mut c = Command::new(&full[0]);
    c.args(&full[1..]);
    c.env("BONES_CONFIG_NAME", profile);
    c.env("VK_LAYER_PATH", rt);
    c.env("VK_INSTANCE_LAYERS", LAYER_NAME);
    match needs_preload(preload) {
        true => { c.env("LD_PRELOAD", preload); }
        false => (),
    };
    let err = c.exec();
    log_at(LogLevel::Error, &format!("exec failed: {}", err));
    127
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

fn lib_search_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/local/lib/libbones.so"),
        PathBuf::from("/usr/local/lib32/libbones.so"),
        PathBuf::from("/usr/lib/libbones.so"),
        PathBuf::from("/usr/lib32/libbones.so"),
        PathBuf::from("target/release/libbones.so"),
    ]
}

fn locate_lib(search: &[PathBuf]) -> Result<PathBuf, ()> {
    first_existing(search).map(|p| p.canonicalize().unwrap_or(p)).ok_or(())
}

fn first_existing(search: &[PathBuf]) -> Option<PathBuf> {
    search.iter().find(|p| p.exists()).cloned()
}

fn src_newer(src: &Option<PathBuf>, dest: &PathBuf) -> bool {
    let src_t = src.as_ref().and_then(|p| fs::metadata(p).ok()).and_then(|m| m.modified().ok());
    let dst_t = fs::metadata(dest).ok().and_then(|m| m.modified().ok());
    match (src_t, dst_t) {
        (Some(s), Some(d)) => s > d,
        (Some(_), None) => true,
        (None, _) => false,
    }
}

fn manifest_json(abs_lib: &str) -> String {
    MANIFEST_JSON.replace(
        "\"library_path\": \"libbones.so\"",
        &format!("\"library_path\": \"{}\"", abs_lib),
    )
}

fn so_path(dest_dir: &PathBuf, arch: &str) -> PathBuf {
    dest_dir.join(arch)
}

fn plan_stage(search: &[PathBuf], dest_dir: &PathBuf, arch: &str) -> StagePlan {
    let src = first_existing(search);
    let dest = so_path(dest_dir, arch);
    StagePlan {
        manifest: manifest_json(&dest_dir.join("libbones.so").to_string_lossy()),
        need_copy: src_newer(&src, &dest),
        src,
        dest,
    }
}

fn stage_runtime() {
    let rt = config_dir().join("runtime");
    let _ = fs::create_dir_all(&rt);
    let plan = plan_stage(&lib_search_paths(), &rt, ARCH_SO);
    match (plan.need_copy, &plan.src) {
        (true, Some(src)) => {
            let _ = fs::copy(src, &plan.dest);
            let _ = fs::copy(src, rt.join("libbones.so"));
        }
        (_, _) => (),
    }
    let _ = fs::write(rt.join("VkLayer_bones.json"), &plan.manifest);
}
