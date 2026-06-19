use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use std::sync::Once;

use crate::consts::CONFIG_SEP;
use crate::consts::ENV_CONFIG;
use crate::consts::ENV_CONFIG_NAME;
use crate::consts::EffectDef;
use crate::consts::HEAD;
use crate::consts::HOT_RELOAD_KEY;
use crate::consts::REGISTRY;
use crate::logging::{LogLevel, log_at, init_log_level};
use crate::shader::{build_shaders, store_shaders};
use crate::watch::setup_watch;

#[derive(Clone)]
pub(crate) struct Settings {
    pub(crate) effects: HashMap<String, bool>,
    pub(crate) hot_reload: bool,
}

static SETTINGS: RwLock<Option<Settings>> = RwLock::new(None);

fn toml_bool(v: &toml::Value) -> Option<bool> {
    match v {
        toml::Value::Boolean(b) => Some(*b),
        _ => None,
    }
}

fn name_is_known(raw: &str, reg: &[EffectDef]) -> bool {
    raw == HOT_RELOAD_KEY || reg.iter().any(|e| e.name == raw)
}

fn log_unknown_effect(name: &str) {
    log_at(LogLevel::Warn, &format!("unknown effect '{}' in config, ignoring", name));
}

fn log_non_bool_effect(name: &str) {
    log_at(LogLevel::Warn, &format!("non-boolean value for '{}' in config, ignoring", name));
}

fn validated_toml_effect(name: &str, v: &toml::Value, reg: &[EffectDef]) -> Option<(String, bool)> {
    match (name_is_known(name, reg), toml_bool(v)) {
        (true, Some(b)) => Some((name.to_string(), b)),
        (true, None) => {
            log_non_bool_effect(name);
            None
        }
        (false, _) => {
            log_unknown_effect(name);
            None
        }
    }
}

fn section_effects(v: &toml::Value, reg: &[EffectDef]) -> Vec<(String, bool)> {
    match v.as_table() {
        Some(sec) => sec.iter().filter_map(|(k, v2)| validated_toml_effect(k, v2, reg)).collect(),
        None => Vec::new(),
    }
}

fn effects_of(doc: &toml::Value, reg: &[EffectDef]) -> HashMap<String, bool> {
    match doc.as_table() {
        Some(t) => t.iter().flat_map(|(_, v)| section_effects(v, reg)).collect(),
        None => HashMap::new(),
    }
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

fn effects_from_list(text: &str, reg: &[EffectDef]) -> HashMap<String, bool> {
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
    let hot = effects.get(HOT_RELOAD_KEY).copied().unwrap_or(true);
    Settings { hot_reload: hot, effects }
}

pub(crate) fn settings_from_env(text: &str, reg: &[EffectDef]) -> Settings {
    Settings { hot_reload: false, effects: effects_from_list(text, reg) }
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
    PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
}

pub(crate) fn config_dir() -> PathBuf {
    home_dir().join(".config").join("bones")
}

pub(crate) fn profile_name() -> String {
    sanitize_name(&env::var(ENV_CONFIG_NAME).unwrap_or_else(|_| "bones".into()))
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
    match env::var(ENV_CONFIG) {
        Ok(text) => settings_from_env(&text, reg),
        Err(_) => read_config(&config_path(&profile_name()), reg),
    }
}

static INIT: Once = Once::new();

pub(crate) fn ensure_settings() -> Settings {
    INIT.call_once(|| {
        load_settings();
    });
    SETTINGS.read().ok().and_then(|g| g.clone()).unwrap_or_else(|| {
        load_settings()
    })
}

pub(crate) fn load_settings() -> Settings {
    init_log_level();
    let s = resolve_settings(&REGISTRY);
    let (gl, spv) = build_shaders(&s, &REGISTRY);
    store_shaders(gl, spv);
    store_settings(s.clone());
    setup_watch(&s);
    log_at(LogLevel::Info, "settings loaded");
    s
}

pub(crate) fn store_settings(s: Settings) {
    match SETTINGS.write() {
        Ok(mut g) => *g = Some(s),
        Err(_) => (),
    }
}
