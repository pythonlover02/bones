use crate::config::Settings;
use crate::consts::EffectDef;
use crate::consts::EffectKind;
use crate::consts::FragDef;
use crate::consts::RoleDef;
use crate::consts::GRADED_ROLES;
use crate::consts::GRADE_CLAMP;
use crate::consts::GRADE_CLOSE;
use crate::consts::GRADE_FRAGS;
use crate::consts::GRADE_FRAME_FRAGS;
use crate::consts::GRADE_HARDWARE_FRAGS;
use crate::consts::GRADE_OPEN;
use crate::consts::HISTORY_FRAG;
use crate::consts::HISTORY_HUD_MASK;
use crate::consts::HUD_BOX_FRAG;
use crate::consts::MAIN_CLOSE;
use crate::consts::MAIN_FETCH;
use crate::consts::MAIN_FETCH_CHROMATIC;
use crate::consts::MAIN_GRADE;
use crate::consts::MAIN_OPEN;
use crate::consts::NOISE_FRAGS;
use crate::consts::OVERLAY_FRAGS;
use crate::consts::PRE_TEMPORAL_FRAGS;
use crate::consts::SPATIAL_FRAGS;
use crate::consts::SPATIAL_TAIL_FRAGS;
use crate::consts::STABILIZER_FRAG;
use crate::consts::TAP_ROLES;
use crate::consts::TEMPORAL_FRAGS;
use crate::consts::UV_WARP_FRAGS;

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

fn role_active(s: &Settings, role: &RoleDef) -> bool {
    role.consumers.iter().any(|n| enabled(s, n))
}

fn enabled_frags<'a>(s: &Settings, frags: &'a [FragDef]) -> Vec<&'a str> {
    frags
        .iter()
        .filter(|f| enabled(s, f.name))
        .map(|f| f.src)
        .collect()
}

fn active_roles<'a>(s: &Settings, roles: &'a [&'a RoleDef]) -> Vec<&'a str> {
    roles
        .iter()
        .filter(|r| role_active(s, r))
        .map(|r| r.src)
        .collect()
}

fn fetch_frag(s: &Settings) -> &'static str {
    match enabled(s, "chromatic_aberration") {
        true => MAIN_FETCH_CHROMATIC,
        false => MAIN_FETCH,
    }
}

fn hud_box_frags(s: &Settings) -> Vec<&'static str> {
    match enabled(s, "fps_hud") {
        true => vec![HUD_BOX_FRAG],
        false => Vec::new(),
    }
}

fn history_frags(s: &Settings, reg: &[EffectDef]) -> Vec<&'static str> {
    match (temporal_enabled(s, reg), enabled(s, "fps_hud")) {
        (false, _) => Vec::new(),
        (true, false) => vec![HISTORY_FRAG],
        (true, true) => vec![HISTORY_FRAG, HISTORY_HUD_MASK],
    }
}

fn stabilizer_frags(s: &Settings, reg: &[EffectDef]) -> Vec<&'static str> {
    match temporal_enabled(s, reg) {
        true => vec![STABILIZER_FRAG],
        false => Vec::new(),
    }
}

fn grade_fn_frags(s: &Settings) -> Vec<&'static str> {
    [
        vec![GRADE_OPEN],
        enabled_frags(s, &GRADE_FRAGS),
        enabled_frags(s, &GRADE_FRAME_FRAGS),
        vec![GRADE_CLAMP],
        enabled_frags(s, &GRADE_HARDWARE_FRAGS),
        vec![GRADE_CLOSE],
    ]
    .concat()
}

fn main_frags(s: &Settings, reg: &[EffectDef]) -> Vec<&'static str> {
    [
        vec![MAIN_OPEN],
        enabled_frags(s, &UV_WARP_FRAGS),
        vec![fetch_frag(s)],
        active_roles(s, &TAP_ROLES),
        hud_box_frags(s),
        history_frags(s, reg),
        enabled_frags(s, &SPATIAL_FRAGS),
        enabled_frags(s, &SPATIAL_TAIL_FRAGS),
        vec![MAIN_GRADE],
        active_roles(s, &GRADED_ROLES),
        enabled_frags(s, &PRE_TEMPORAL_FRAGS),
        enabled_frags(s, &TEMPORAL_FRAGS),
        stabilizer_frags(s, reg),
        enabled_frags(s, &NOISE_FRAGS),
        enabled_frags(s, &OVERLAY_FRAGS),
        vec![MAIN_CLOSE],
    ]
    .concat()
}

pub(crate) fn shader_body_frags(s: &Settings, reg: &[EffectDef]) -> Vec<&'static str> {
    [grade_fn_frags(s), main_frags(s, reg)].concat()
}
