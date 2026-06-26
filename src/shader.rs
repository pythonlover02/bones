use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use std::sync::RwLock;

use crate::config::Settings;
use crate::consts::COMPUTE_X_DEFAULT;
use crate::consts::COMPUTE_Y_DEFAULT;
use crate::consts::EffectDef;
use crate::consts::UBER_SRC;
use crate::consts::VERT_SRC;
use crate::effect::emit_defines;
use crate::logging::log_at;
use crate::logging::LogLevel;

pub(crate) struct ShaderArtifacts {
    pub(crate) frag: Vec<u32>,
    pub(crate) comp: Vec<u32>,
    pub(crate) compute_x: u32,
    pub(crate) compute_y: u32,
}

static SHADER_FRAG_SPV: RwLock<Option<Vec<u32>>> = RwLock::new(None);
static SHADER_COMP_SPV: RwLock<Option<Vec<u32>>> = RwLock::new(None);
static SHADER_WG: RwLock<(u32, u32)> = RwLock::new((COMPUTE_X_DEFAULT, COMPUTE_Y_DEFAULT));
static VERT_SPV: RwLock<Option<Vec<u32>>> = RwLock::new(None);
static WG_LIMITS: RwLock<(u32, u32, u32)> = RwLock::new((u32::MAX, u32::MAX, u32::MAX));
static SUBGROUP_CAPS: RwLock<(bool, bool)> = RwLock::new((false, false));
pub(crate) static GENERATION: AtomicI32 = AtomicI32::new(0);

fn split_first_line(src: &str) -> (&str, &str) {
    let nl = src.find('\n').unwrap_or(src.len());
    (&src[..nl], src.get(nl + 1..).unwrap_or(""))
}

fn read_wg_limits() -> (u32, u32, u32) {
    WG_LIMITS.read().map(|g| *g).unwrap_or((u32::MAX, u32::MAX, u32::MAX))
}

fn fallback_wg() -> (u32, u32) {
    (COMPUTE_X_DEFAULT, COMPUTE_Y_DEFAULT)
}

fn clamp_wg_axes(req_x: u32, req_y: u32, max_x: u32, max_y: u32) -> (u32, u32) {
    (req_x.clamp(1, max_x.max(1)), req_y.clamp(1, max_y.max(1)))
}

fn wg_within_invocation_limit(x: u32, y: u32, max_inv: u32) -> bool {
    (x as u64).saturating_mul(y as u64) <= max_inv.max(1) as u64
}

fn log_wg_fallback(x: u32, y: u32, max_inv: u32) {
    log_at(
        LogLevel::Warn,
        &format!(
            "compute workgroup {}x{} exceeds device max invocations {}, falling back to {}x{}",
            x, y, max_inv, COMPUTE_X_DEFAULT, COMPUTE_Y_DEFAULT
        ),
    );
}

fn maybe_log_wg_fallback(x: u32, y: u32, max_inv: u32) {
    match wg_within_invocation_limit(x, y, max_inv) {
        true => (),
        false => log_wg_fallback(x, y, max_inv),
    }
}

fn pick_wg(x: u32, y: u32, max_inv: u32) -> (u32, u32) {
    match wg_within_invocation_limit(x, y, max_inv) {
        true => (x, y),
        false => fallback_wg(),
    }
}

pub(crate) fn effective_workgroup(req_x: u32, req_y: u32) -> (u32, u32) {
    let (max_x, max_y, max_inv) = read_wg_limits();
    let (cx, cy) = clamp_wg_axes(req_x, req_y, max_x, max_y);
    maybe_log_wg_fallback(cx, cy, max_inv);
    pick_wg(cx, cy, max_inv)
}

pub(crate) fn set_wg_limits(max_x: u32, max_y: u32, max_inv: u32) {
    match WG_LIMITS.write() {
        Ok(mut g) => {
            let (cur_x, cur_y, cur_inv) = *g;
            *g = (
                cur_x.min(max_x.max(1)),
                cur_y.min(max_y.max(1)),
                cur_inv.min(max_inv.max(1)),
            );
        }
        Err(_) => (),
    }
}

pub(crate) fn set_subgroup_caps(ext_types: bool, uniform_flow: bool) {
    match SUBGROUP_CAPS.write() {
        Ok(mut g) => *g = (ext_types, uniform_flow),
        Err(_) => (),
    }
}

fn subgroup_define_block(ext_types: bool, uniform_flow: bool) -> String {
    let mut out = String::new();
    match ext_types {
        true => out.push_str("#define BONES_HAS_SUBGROUP_EXT_TYPES\n"),
        false => (),
    }
    match uniform_flow {
        true => out.push_str("#define BONES_HAS_SUBGROUP_UNIFORM_FLOW\n"),
        false => (),
    }
    out
}

fn current_subgroup_caps() -> (bool, bool) {
    SUBGROUP_CAPS.read().map(|g| *g).unwrap_or((false, false))
}

fn assemble_frag_source(s: &Settings, reg: &[EffectDef]) -> String {
    let (ver, rest) = split_first_line(UBER_SRC);
    let (ext, uni) = current_subgroup_caps();
    format!(
        "{}\n{}{}{}",
        ver,
        subgroup_define_block(ext, uni),
        emit_defines(s, reg),
        rest
    )
}

fn assemble_comp_source(s: &Settings, reg: &[EffectDef], wg_x: u32, wg_y: u32) -> String {
    let (ver, rest) = split_first_line(UBER_SRC);
    let (ext, uni) = current_subgroup_caps();
    format!(
        "{}\n#define COMPUTE_PATH\n#define LOCAL_SIZE_X {}\n#define LOCAL_SIZE_Y {}\n{}{}{}",
        ver,
        wg_x,
        wg_y,
        subgroup_define_block(ext, uni),
        emit_defines(s, reg),
        rest
    )
}

fn new_compiler() -> Result<shaderc::Compiler, ()> {
    shaderc::Compiler::new().ok_or_else(|| log_at(LogLevel::Error, "shaderc compiler init failed"))
}

fn new_compile_options() -> Result<shaderc::CompileOptions<'static>, ()> {
    shaderc::CompileOptions::new()
        .ok_or_else(|| log_at(LogLevel::Error, "shaderc options init failed"))
}

fn run_compile(
    c: &shaderc::Compiler,
    o: &shaderc::CompileOptions,
    src: &str,
    kind: shaderc::ShaderKind,
    name: &str,
) -> Result<Vec<u32>, ()> {
    c.compile_into_spirv(src, kind, name, "main", Some(o))
        .map(|a| a.as_binary().to_vec())
        .map_err(|e| log_at(LogLevel::Error, &format!("spirv compile failed ({}): {}", name, e)))
}

fn compile_spirv(src: &str, kind: shaderc::ShaderKind, name: &str) -> Result<Vec<u32>, ()> {
    new_compiler()
        .and_then(|c| new_compile_options().and_then(|o| run_compile(&c, &o, src, kind, name)))
}

fn cached_vert_spirv() -> Option<Vec<u32>> {
    VERT_SPV.read().ok().and_then(|g| g.clone())
}

fn store_vert_spirv(v: Vec<u32>) {
    match VERT_SPV.write() {
        Ok(mut g) => *g = Some(v),
        Err(_) => (),
    }
}

fn compile_and_cache_vert() -> Result<Vec<u32>, ()> {
    compile_spirv(VERT_SRC, shaderc::ShaderKind::Vertex, "bones.vert").map(|v| {
        store_vert_spirv(v.clone());
        v
    })
}

pub(crate) fn compile_vert_spirv() -> Result<Vec<u32>, ()> {
    match cached_vert_spirv() {
        Some(v) => Ok(v),
        None => compile_and_cache_vert(),
    }
}

fn compile_frag_spirv(src: &str) -> Vec<u32> {
    compile_spirv(src, shaderc::ShaderKind::Fragment, "bones.frag").unwrap_or_default()
}

fn compile_comp_spirv(src: &str) -> Vec<u32> {
    compile_spirv(src, shaderc::ShaderKind::Compute, "bones.comp").unwrap_or_default()
}

fn maybe_compile_comp(s: &Settings, reg: &[EffectDef], wg_x: u32, wg_y: u32) -> Vec<u32> {
    match s.compute {
        true => compile_comp_spirv(&assemble_comp_source(s, reg, wg_x, wg_y)),
        false => Vec::new(),
    }
}

pub(crate) fn build_shaders(s: &Settings, reg: &[EffectDef]) -> ShaderArtifacts {
    let (wg_x, wg_y) = effective_workgroup(s.compute_x, s.compute_y);
    ShaderArtifacts {
        frag: compile_frag_spirv(&assemble_frag_source(s, reg)),
        comp: maybe_compile_comp(s, reg, wg_x, wg_y),
        compute_x: wg_x,
        compute_y: wg_y,
    }
}

fn store_spv(slot: &RwLock<Option<Vec<u32>>>, spv: Vec<u32>) {
    match slot.write() {
        Ok(mut g) => *g = Some(spv),
        Err(_) => (),
    }
}

fn store_wg(x: u32, y: u32) {
    match SHADER_WG.write() {
        Ok(mut g) => *g = (x, y),
        Err(_) => (),
    }
}

pub(crate) fn store_shaders(a: ShaderArtifacts) {
    store_spv(&SHADER_FRAG_SPV, a.frag);
    store_spv(&SHADER_COMP_SPV, a.comp);
    store_wg(a.compute_x, a.compute_y);
    GENERATION.fetch_add(1, Ordering::Relaxed);
}

fn read_spv(slot: &RwLock<Option<Vec<u32>>>) -> Vec<u32> {
    slot.read().ok().and_then(|g| g.clone()).unwrap_or_default()
}

pub(crate) fn current_frag_spv() -> Vec<u32> {
    read_spv(&SHADER_FRAG_SPV)
}

pub(crate) fn current_comp_spv() -> Vec<u32> {
    read_spv(&SHADER_COMP_SPV)
}

pub(crate) fn current_wg() -> (u32, u32) {
    SHADER_WG
        .read()
        .map(|g| *g)
        .unwrap_or((COMPUTE_X_DEFAULT, COMPUTE_Y_DEFAULT))
}
