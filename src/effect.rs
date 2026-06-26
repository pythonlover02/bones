use crate::consts::EffectDef;
use crate::consts::EffectKind;
use crate::config::Settings;

pub(crate) fn enabled(s: &Settings, name: &str) -> bool {
    match s.effects.get(name) {
        Some(v) => *v,
        None => false,
    }
}

pub(crate) fn any_effect_enabled(s: &Settings, reg: &[EffectDef]) -> bool {
    reg.iter().any(|e| enabled(s, e.name))
}

pub(crate) fn any_of_kind_enabled(s: &Settings, reg: &[EffectDef], kind: EffectKind) -> bool {
    reg.iter()
        .filter(|e| e.kind == kind)
        .any(|e| enabled(s, e.name))
}

pub(crate) fn temporal_enabled(s: &Settings, reg: &[EffectDef]) -> bool {
    any_of_kind_enabled(s, reg, EffectKind::Temporal)
}

pub(crate) fn emit_defines(s: &Settings, reg: &[EffectDef]) -> String {
    reg.iter()
        .filter(|e| enabled(s, e.name))
        .map(|e| format!("#define ENABLE_{}\n", e.name.to_uppercase()))
        .collect()
}
