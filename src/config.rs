use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Once;
use std::sync::RwLock;

use crate::consts::CONFIG_SEP;
use crate::consts::COMPUTE_KEY;
use crate::consts::COMPUTE_X_DEFAULT;
use crate::consts::COMPUTE_X_KEY;
use crate::consts::COMPUTE_Y_DEFAULT;
use crate::consts::COMPUTE_Y_KEY;
use crate::consts::EffectDef;
use crate::consts::ENV_COMPUTE;
use crate::consts::ENV_COMPUTE_X;
use crate::consts::ENV_COMPUTE_Y;
use crate::consts::ENV_OPT_DYNREN;
use crate::consts::ENV_OPT_FP16;
use crate::consts::ENV_OPT_PUSHDESC;
use crate::consts::ENV_OPT_ASYNC_COMPUTE;
use crate::consts::ENV_OPT_SUBGROUP_EXT_TYPES;
use crate::consts::ENV_OPT_SUBGROUP_UNIFORM_FLOW;
use crate::consts::ENV_OPT_SYNC2;
use crate::consts::ENV_OPT_SUBGROUP;
use crate::consts::GENERAL_BOOL_KEYS;
use crate::consts::GENERAL_FLOAT_KEYS;
use crate::consts::GENERAL_UINT_KEYS;
use crate::consts::HEAD;
use crate::consts::HOT_RELOAD_KEY;
use crate::consts::OPT_DYNREN_KEY;
use crate::consts::OPT_FP16_KEY;
use crate::consts::OPT_PUSHDESC_KEY;
use crate::consts::OPT_ASYNC_COMPUTE_KEY;
use crate::consts::OPT_SUBGROUP_EXT_TYPES_KEY;
use crate::consts::OPT_SUBGROUP_UNIFORM_FLOW_KEY;
use crate::consts::OPT_SYNC2_KEY;
use crate::consts::OPT_SUBGROUP_KEY;
use crate::consts::REGISTRY;
use crate::consts::RES_SCALE_DEFAULT;
use crate::consts::RES_SCALE_KEY;
use crate::consts::RES_SCALE_MIN;
use crate::env::env_bool;
use crate::env::env_bypass_active;
use crate::env::env_config_inline;
use crate::env::env_config_name;
use crate::env::env_res_scale;
use crate::env::env_uint;
use crate::env::env_home;
use crate::logging::init_log_level;
use crate::logging::log_at;
use crate::logging::LogLevel;
use crate::watch::setup_watch;

#[derive(Clone)]
pub(crate) struct Settings {
    pub(crate) effects: HashMap<String, bool>,
    pub(crate) hot_reload: bool,
    pub(crate) res_scale: f32,
    pub(crate) opt_fp16: bool,
    pub(crate) opt_dynren: bool,
    pub(crate) opt_pushdesc: bool,
    pub(crate) opt_subgroup: bool,
    pub(crate) opt_sync2: bool,
    pub(crate) opt_subgroup_ext_types: bool,
    pub(crate) opt_subgroup_uniform_flow: bool,
    pub(crate) opt_async_compute: bool,
    pub(crate) compute: bool,
    pub(crate) compute_x: u32,
    pub(crate) compute_y: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            effects: HashMap::new(),
            hot_reload: true,
            res_scale: RES_SCALE_DEFAULT,
            opt_fp16: true,
            opt_dynren: true,
            opt_pushdesc: true,
            opt_subgroup: true,
            opt_sync2: true,
            opt_subgroup_ext_types: true,
            opt_subgroup_uniform_flow: true,
            opt_async_compute: true,
            compute: true,
            compute_x: COMPUTE_X_DEFAULT,
            compute_y: COMPUTE_Y_DEFAULT,
        }
    }
}

static SETTINGS: RwLock<Option<Settings>> = RwLock::new(None);
static INIT: Once = Once::new();

fn toml_bool(v: &toml::Value) -> Option<bool> {
    match v {
        toml::Value::Boolean(b) => Some(*b),
        _ => None,
    }
}

fn toml_float(v: &toml::Value) -> Option<f64> {
    match v {
        toml::Value::Float(f) => Some(*f),
        toml::Value::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

fn toml_uint(v: &toml::Value) -> Option<u64> {
    match v {
        toml::Value::Integer(i) => Some((*i).max(0) as u64),
        _ => None,
    }
}

fn name_is_general_bool(raw: &str) -> bool {
    GENERAL_BOOL_KEYS.contains(&raw)
}

fn name_is_general_float(raw: &str) -> bool {
    GENERAL_FLOAT_KEYS.contains(&raw)
}

fn name_is_general_uint(raw: &str) -> bool {
    GENERAL_UINT_KEYS.contains(&raw)
}

fn name_is_general(raw: &str) -> bool {
    name_is_general_bool(raw) || name_is_general_float(raw) || name_is_general_uint(raw)
}

fn name_is_known(raw: &str, reg: &[EffectDef]) -> bool {
    name_is_general(raw) || reg.iter().any(|e| e.name == raw)
}

fn log_unknown_effect(name: &str) {
    log_at(LogLevel::Warn, &format!("unknown effect '{}' in config, ignoring", name));
}

fn log_non_bool_effect(name: &str) {
    log_at(LogLevel::Warn, &format!("non-boolean value for '{}' in config, ignoring", name));
}

fn classify_toml_key(name: &str, v: &toml::Value, reg: &[EffectDef]) -> Option<(String, bool)> {
    match (name_is_known(name, reg), name_is_general(name), toml_bool(v)) {
        (true, true, _) => None,
        (true, false, Some(b)) => Some((name.to_string(), b)),
        (true, false, None) => {
            log_non_bool_effect(name);
            None
        }
        (false, _, _) => {
            log_unknown_effect(name);
            None
        }
    }
}

fn section_effects(v: &toml::Value, reg: &[EffectDef]) -> Vec<(String, bool)> {
    match v.as_table() {
        Some(sec) => sec
            .iter()
            .filter_map(|(k, v2)| classify_toml_key(k, v2, reg))
            .collect(),
        None => Vec::new(),
    }
}

fn effects_of(doc: &toml::Value, reg: &[EffectDef]) -> HashMap<String, bool> {
    match doc.as_table() {
        Some(t) => t
            .iter()
            .flat_map(|(_, v)| section_effects(v, reg))
            .collect(),
        None => HashMap::new(),
    }
}

fn general_table(doc: &toml::Value) -> Option<&toml::value::Table> {
    doc.as_table()
        .and_then(|t| t.get("general"))
        .and_then(|v| v.as_table())
}

fn parse_res_scale(doc: &toml::Value) -> f32 {
    general_table(doc)
        .and_then(|t| t.get(RES_SCALE_KEY))
        .and_then(toml_float)
        .map(|v| (v as f32).max(RES_SCALE_MIN))
        .unwrap_or(RES_SCALE_DEFAULT)
}

fn parse_general_uint(doc: &toml::Value, key: &str, default: u32) -> u32 {
    general_table(doc)
        .and_then(|t| t.get(key))
        .and_then(toml_uint)
        .map(|v| v.min(u32::MAX as u64) as u32)
        .unwrap_or(default)
}

fn parse_bool_opt(effects: &HashMap<String, bool>, key: &str, default: bool) -> bool {
    effects.get(key).copied().unwrap_or(default)
}

fn warn_unknown(raw: &str, reg: &[EffectDef]) -> Option<(String, bool)> {
    match name_is_known(raw, reg) {
        true => Some((raw.to_string(), true)),
        false => {
            log_at(LogLevel::Warn, "unknown effect in BONES_CONFIG, ignoring");
            None
        }
    }
}

pub(crate) fn effects_from_list(text: &str, reg: &[EffectDef]) -> HashMap<String, bool> {
    text.split(CONFIG_SEP)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|s| warn_unknown(s, reg))
        .collect()
}

pub(crate) fn parse_settings(text: &str, reg: &[EffectDef]) -> Settings {
    let doc = match text.parse::<toml::Value>() {
        Ok(d) => d,
        Err(e) => {
            log_at(LogLevel::Warn, &format!("config parse failed: {}, using defaults", e));
            toml::Value::Table(toml::map::Map::new())
        }
    };
    let effects = effects_of(&doc, reg);
    Settings {
        hot_reload: parse_bool_opt(&effects, HOT_RELOAD_KEY, true),
        opt_fp16: parse_bool_opt(&effects, OPT_FP16_KEY, true),
        opt_dynren: parse_bool_opt(&effects, OPT_DYNREN_KEY, true),
        opt_pushdesc: parse_bool_opt(&effects, OPT_PUSHDESC_KEY, true),
        opt_subgroup: parse_bool_opt(&effects, OPT_SUBGROUP_KEY, true),
        opt_sync2: parse_bool_opt(&effects, OPT_SYNC2_KEY, true),
        opt_subgroup_ext_types: parse_bool_opt(&effects, OPT_SUBGROUP_EXT_TYPES_KEY, true),
        opt_subgroup_uniform_flow: parse_bool_opt(&effects, OPT_SUBGROUP_UNIFORM_FLOW_KEY, true),
        opt_async_compute: parse_bool_opt(&effects, OPT_ASYNC_COMPUTE_KEY, true),
        compute: parse_bool_opt(&effects, COMPUTE_KEY, true),
        compute_x: parse_general_uint(&doc, COMPUTE_X_KEY, COMPUTE_X_DEFAULT),
        compute_y: parse_general_uint(&doc, COMPUTE_Y_KEY, COMPUTE_Y_DEFAULT),
        res_scale: parse_res_scale(&doc),
        effects,
    }
}

fn env_effects(reg: &[EffectDef]) -> HashMap<String, bool> {
    match env_config_inline() {
        Some(text) => effects_from_list(&text, reg),
        None => HashMap::new(),
    }
}

pub(crate) fn settings_from_env(reg: &[EffectDef]) -> Settings {
    Settings {
        effects: env_effects(reg),
        hot_reload: false,
        res_scale: env_res_scale(RES_SCALE_DEFAULT),
        opt_fp16: env_bool(ENV_OPT_FP16, true),
        opt_dynren: env_bool(ENV_OPT_DYNREN, true),
        opt_pushdesc: env_bool(ENV_OPT_PUSHDESC, true),
        opt_subgroup: env_bool(ENV_OPT_SUBGROUP, true),
        opt_sync2: env_bool(ENV_OPT_SYNC2, true),
        opt_subgroup_ext_types: env_bool(ENV_OPT_SUBGROUP_EXT_TYPES, true),
        opt_subgroup_uniform_flow: env_bool(ENV_OPT_SUBGROUP_UNIFORM_FLOW, true),
        opt_async_compute: env_bool(ENV_OPT_ASYNC_COMPUTE, true),
        compute: env_bool(ENV_COMPUTE, true),
        compute_x: env_uint(ENV_COMPUTE_X, COMPUTE_X_DEFAULT),
        compute_y: env_uint(ENV_COMPUTE_Y, COMPUTE_Y_DEFAULT),
    }
}

pub(crate) fn default_settings() -> Settings {
    parse_settings(HEAD, &REGISTRY)
}

fn name_is_valid(raw: &str) -> bool {
    !raw.is_empty()
        && !raw.contains('/')
        && !raw.contains('\\')
        && !raw.contains("..")
        && !raw.contains('\0')
        && raw.chars().all(|ch| ch.is_ascii_graphic())
}

pub(crate) fn sanitize_name(raw: &str) -> String {
    match name_is_valid(raw) {
        true => raw.into(),
        false => {
            log_at(LogLevel::Warn, "invalid profile name, using default profile");
            "bones".into()
        }
    }
}

pub(crate) fn home_dir() -> PathBuf {
    PathBuf::from(env_home())
}

pub(crate) fn config_dir() -> PathBuf {
    home_dir().join(".config").join("bones")
}

pub(crate) fn profile_name() -> String {
    sanitize_name(&env_config_name())
}

pub(crate) fn config_path(name: &str) -> PathBuf {
    config_dir().join(format!("{}-config.toml", name))
}

pub(crate) fn read_config(path: &PathBuf, reg: &[EffectDef]) -> Settings {
    fs::read_to_string(path)
        .map(|t| parse_settings(&t, reg))
        .unwrap_or_else(|_| default_settings())
}

pub(crate) fn resolve_settings(reg: &[EffectDef]) -> Settings {
    match env_bypass_active() {
        true => settings_from_env(reg),
        false => read_config(&config_path(&profile_name()), reg),
    }
}

pub(crate) fn store_settings(s: Settings) {
    match SETTINGS.write() {
        Ok(mut g) => *g = Some(s),
        Err(_) => (),
    }
}

pub(crate) fn load_settings() -> Settings {
    init_log_level();
    let s = resolve_settings(&REGISTRY);
    store_settings(s.clone());
    setup_watch(&s);
    log_at(LogLevel::Info, "settings loaded");
    s
}

pub(crate) fn ensure_settings() -> Settings {
    INIT.call_once(|| {
        load_settings();
    });
    SETTINGS
        .read()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| load_settings())
}
