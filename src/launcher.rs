use std::fs;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use crate::config::config_dir;
use crate::config::config_path;
use crate::config::sanitize_name;
use crate::consts::ARCH_SO;
use crate::consts::FLATPAK_CMD;
use crate::consts::FLATPAK_INJECT_ABS;
use crate::consts::FLATPAK_METADATA_KEY;
use crate::consts::FLATPAK_RUN;
use crate::consts::HEAD;
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
    match is_flatpak_run(cmd) {
        true => exec_flatpak(cmd, &profile),
        false => exec_native(cmd, &profile),
    }
}

fn is_flatpak_cmd(name: &str) -> bool {
    name == FLATPAK_CMD || name.ends_with("/flatpak")
}

fn is_flatpak_run(cmd: &[String]) -> bool {
    cmd.first().map(|s| is_flatpak_cmd(s)).unwrap_or(false)
        && cmd.iter().any(|a| a == FLATPAK_RUN)
}

fn flatpak_run_pos(cmd: &[String]) -> Option<usize> {
    cmd.iter().position(|a| a == FLATPAK_RUN)
}

fn flatpak_app_id(cmd: &[String]) -> Option<&str> {
    flatpak_run_pos(cmd)
        .and_then(|pos| cmd[pos + 1..].iter().find(|a| !a.starts_with('-')))
        .map(|s| s.as_str())
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

fn flatpak_trailing_args(cmd: &[String]) -> Vec<String> {
    flatpak_run_pos(cmd)
        .and_then(|pos| {
            cmd[pos + 1..]
                .iter()
                .position(|a| !a.starts_with('-'))
                .map(|app_rel| cmd[pos + 1 + app_rel + 1..].to_vec())
        })
        .unwrap_or_default()
}

fn parse_metadata_command(text: &str) -> Option<String> {
    text.lines()
        .find(|l| l.starts_with(FLATPAK_METADATA_KEY))
        .map(|l| l[FLATPAK_METADATA_KEY.len()..].trim().to_string())
}

fn call_flatpak_metadata(app_id: &str) -> Option<String> {
    Command::new(FLATPAK_CMD)
        .args(["info", "--show-metadata", app_id])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|text| parse_metadata_command(&text))
}

fn config_dir_str() -> String {
    config_dir().to_string_lossy().into_owned()
}

fn build_flatpak_cmd(
    profile: &str,
    app_id: &str,
    app_cmd: &str,
    user_flags: &[String],
    trailing: &[String],
) -> Vec<String> {
    let mut args = vec![
        FLATPAK_RUN.to_string(),
        format!("--command={}", FLATPAK_INJECT_ABS),
        format!("--env=BONES_CONFIG_NAME={}", profile),
        format!("--filesystem={}:rw", config_dir_str()),
    ];
    args.extend(user_flags.iter().cloned());
    args.push(app_id.to_string());
    args.push(app_cmd.to_string());
    args.extend(trailing.iter().cloned());
    args
}

fn exec_flatpak(cmd: &[String], profile: &str) -> i32 {
    match flatpak_app_id(cmd) {
        None => {
            log_at(LogLevel::Error, "flatpak run: could not find app ID in arguments");
            1
        }
        Some(app_id) => {
            let app_cmd = call_flatpak_metadata(app_id)
                .unwrap_or_else(|| app_id.to_string());
            let args = build_flatpak_cmd(
                profile,
                app_id,
                &app_cmd,
                &flatpak_user_flags(cmd),
                &flatpak_trailing_args(cmd),
            );
            let err = Command::new(FLATPAK_CMD).args(&args).exec();
            log_at(LogLevel::Error, &format!("exec failed: {}", err));
            127
        }
    }
}

fn exec_native(cmd: &[String], profile: &str) -> i32 {
    stage_runtime();
    let rt = config_dir().join("runtime");
    let preload = locate_lib(&lib_search_paths())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| rt.join(ARCH_SO).to_string_lossy().into_owned());
    exec_child(cmd, profile, &preload, &rt)
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
