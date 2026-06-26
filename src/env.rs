use std::env;

use crate::consts::ENV_COMPUTE;
use crate::consts::ENV_COMPUTE_X;
use crate::consts::ENV_COMPUTE_Y;
use crate::consts::ENV_CONFIG;
use crate::consts::ENV_CONFIG_NAME;
use crate::consts::ENV_LOG;
use crate::consts::ENV_OPT_DYNREN;
use crate::consts::ENV_OPT_FP16;
use crate::consts::ENV_OPT_PUSHDESC;
use crate::consts::ENV_OPT_SUBGROUP;
use crate::consts::ENV_OPT_ASYNC_COMPUTE;
use crate::consts::ENV_OPT_SUBGROUP_EXT_TYPES;
use crate::consts::ENV_OPT_SUBGROUP_UNIFORM_FLOW;
use crate::consts::ENV_OPT_SYNC2;
use crate::consts::ENV_PRELOAD;
use crate::consts::ENV_RES_SCALE;
use crate::consts::ENV_VK_ADD_LAYER_PATH;
use crate::consts::ENV_VK_INSTANCE_LAYERS;
use crate::consts::RES_SCALE_MIN;

pub(crate) const ENV_BYPASS_KEYS: [&str; 13] = [
    ENV_CONFIG,
    ENV_RES_SCALE,
    ENV_OPT_FP16,
    ENV_OPT_DYNREN,
    ENV_OPT_PUSHDESC,
    ENV_OPT_SUBGROUP,
    ENV_OPT_SYNC2,
    ENV_OPT_SUBGROUP_EXT_TYPES,
    ENV_OPT_SUBGROUP_UNIFORM_FLOW,
    ENV_OPT_ASYNC_COMPUTE,
    ENV_COMPUTE,
    ENV_COMPUTE_X,
    ENV_COMPUTE_Y,
];

fn parse_bool(s: &str) -> Option<bool> {
    match s {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}

fn parse_uint(s: &str) -> Option<u32> {
    s.parse::<u32>().ok()
}

fn parse_float(s: &str) -> Option<f32> {
    s.parse::<f32>().ok()
}

fn read_var(key: &str) -> Option<String> {
    env::var(key).ok().filter(|v| !v.is_empty())
}

fn key_is_active(key: &str) -> bool {
    read_var(key).is_some()
}

pub(crate) fn env_bypass_active() -> bool {
    ENV_BYPASS_KEYS.iter().any(|k| key_is_active(k))
}

pub(crate) fn env_bool(key: &str, default: bool) -> bool {
    read_var(key).and_then(|v| parse_bool(&v)).unwrap_or(default)
}

pub(crate) fn env_uint(key: &str, default: u32) -> u32 {
    read_var(key).and_then(|v| parse_uint(&v)).unwrap_or(default)
}

pub(crate) fn env_res_scale(default: f32) -> f32 {
    read_var(ENV_RES_SCALE)
        .and_then(|v| parse_float(&v))
        .map(|v| v.max(RES_SCALE_MIN))
        .unwrap_or(default)
}

pub(crate) fn env_string(key: &str, default: &str) -> String {
    read_var(key).unwrap_or_else(|| default.into())
}

pub(crate) fn env_opt_string(key: &str) -> Option<String> {
    read_var(key)
}

pub(crate) fn env_log_level() -> String {
    env_string(ENV_LOG, "warn")
}

pub(crate) fn env_config_name() -> String {
    env_string(ENV_CONFIG_NAME, "bones")
}

pub(crate) fn env_config_inline() -> Option<String> {
    read_var(ENV_CONFIG)
}

pub(crate) fn env_preload() -> Option<String> {
    env_opt_string(ENV_PRELOAD)
}

pub(crate) fn env_vk_add_layer_path() -> Option<String> {
    env_opt_string(ENV_VK_ADD_LAYER_PATH)
}

pub(crate) fn env_vk_instance_layers() -> Option<String> {
    env_opt_string(ENV_VK_INSTANCE_LAYERS)
}

pub(crate) fn env_home() -> String {
    env::var("HOME").unwrap_or_else(|_| "/tmp".into())
}

pub fn process_args() -> Vec<String> {
    env::args().skip(1).collect()
}
