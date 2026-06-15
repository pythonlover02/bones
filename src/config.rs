use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::consts::HEAD;
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

fn toml_bool(v: &toml::Value) -> bool {
    match v {
        toml::Value::Boolean(b) => *b,
        _ => false,
    }
}

fn effects_of(doc: &toml::Value) -> HashMap<String, bool> {
    match doc.as_table() {
        Some(t) => t
            .iter()
            .flat_map(|(_, v)| match v.as_table() {
                Some(sec) => sec.iter().map(|(k, v2)| (k.clone(), toml_bool(v2))).collect::<Vec<_>>(),
                None => Vec::new(),
            })
            .collect(),
        None => HashMap::new(),
    }
}

pub(crate) fn parse_settings(text: &str) -> Settings {
    let doc = text.parse::<toml::Value>().unwrap_or(toml::Value::Table(toml::map::Map::new()));
    let effects = effects_of(&doc);
    let hot = effects.get("hot_reload").copied().unwrap_or(true);
    Settings { hot_reload: hot, effects }
}

pub(crate) fn default_settings() -> Settings {
    parse_settings(HEAD)
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
    sanitize_name(&env::var("BONES_CONFIG_NAME").unwrap_or_else(|_| "bones".into()))
}

pub(crate) fn config_path(name: &str) -> PathBuf {
    config_dir().join(format!("{}-config.toml", name))
}

pub(crate) fn read_config(path: &PathBuf) -> Settings {
    fs::read_to_string(path)
        .map(|t| parse_settings(&t))
        .unwrap_or_else(|_| default_settings())
}

pub(crate) fn ensure_settings() -> Settings {
    let have = SETTINGS.read().ok().and_then(|g| g.clone());
    match have {
        Some(s) => s,
        None => load_settings(),
    }
}

pub(crate) fn load_settings() -> Settings {
    init_log_level();
    let s = read_config(&config_path(&profile_name()));
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
