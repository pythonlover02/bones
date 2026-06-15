use crate::consts::REGISTRY;
use crate::consts::EffectDef;
use crate::config::Settings;

pub(crate) fn enabled(s: &Settings, name: &str) -> bool {
    match s.effects.get(name) {
        Some(v) => *v,
        None => false,
    }
}

pub(crate) fn any_effect_enabled(s: &Settings) -> bool {
    REGISTRY.iter().any(|e| enabled(s, e.name))
}

pub(crate) fn emit_defines(s: &Settings, reg: &[EffectDef]) -> String {
    reg.iter()
        .filter(|e| enabled(s, e.name))
        .map(|e| format!("#define ENABLE_{}\n", e.name.to_uppercase()))
        .collect()
}
