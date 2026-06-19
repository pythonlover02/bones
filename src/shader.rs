use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use std::sync::RwLock;

use crate::config::default_settings;
use crate::config::Settings;
use crate::consts::VERT_VK_SRC;
use crate::consts::VK_HEADER;
use crate::consts::UBER_SRC;
use crate::consts::EffectDef;
use crate::consts::REGISTRY;
use crate::effect::emit_defines;
use crate::logging::log_at;
use crate::logging::LogLevel;

static SHADER_GL: RwLock<Option<String>> = RwLock::new(None);
static SHADER_SPV: RwLock<Option<Vec<u32>>> = RwLock::new(None);
static VERT_SPV: RwLock<Option<Vec<u32>>> = RwLock::new(None);
pub(crate) static GENERATION: AtomicI32 = AtomicI32::new(0);

enum LineClass {
    Version,
    UniformInput,
    UniformHistory,
    UniformResolution,
    UniformTime,
    UniformFps,
    FragOut,
    Other,
}

fn split_first_line(src: &str) -> (&str, &str) {
    let nl = src.find('\n').unwrap_or(src.len());
    (&src[..nl], src.get(nl + 1..).unwrap_or(""))
}

pub(crate) fn build_gl_source(s: &Settings, reg: &[EffectDef]) -> String {
    let (ver, rest) = split_first_line(UBER_SRC);
    format!("{}\n{}{}", ver, emit_defines(s, reg), rest)
}

fn is_version_line(t: &str) -> bool {
    t.starts_with("#version")
}

fn classify(line: &str) -> LineClass {
    let t = line.trim();
    match t {
        _ if is_version_line(t) => LineClass::Version,
        "uniform sampler2D u_input;" => LineClass::UniformInput,
        "uniform sampler2D u_history;" => LineClass::UniformHistory,
        "uniform vec2 u_resolution;" => LineClass::UniformResolution,
        "uniform float u_time;" => LineClass::UniformTime,
        "uniform float u_fps;" => LineClass::UniformFps,
        "out vec4 frag_out;" => LineClass::FragOut,
        _ => LineClass::Other,
    }
}

fn rewrite_line(line: &str) -> String {
    match classify(line) {
        LineClass::Version => "#version 450".into(),
        LineClass::UniformInput => "layout(set=0, binding=0) uniform sampler2D u_input;".into(),
        LineClass::UniformHistory => "layout(set=0, binding=1) uniform sampler2D u_history;".into(),
        LineClass::UniformResolution => String::new(),
        LineClass::UniformTime => String::new(),
        LineClass::UniformFps => String::new(),
        LineClass::FragOut => String::new(),
        LineClass::Other => line.into(),
    }
}

fn insert_after_version(body: &str, header: &str) -> String {
    let (ver, rest) = split_first_line(body);
    format!("{}\n{}\n{}", ver, header, rest)
}

fn rewrite_gl_to_vk(gl: &str) -> String {
    let body = gl.lines().map(rewrite_line).collect::<Vec<_>>().join("\n");
    insert_after_version(&body, VK_HEADER)
}

fn new_compiler() -> Result<shaderc::Compiler, ()> {
    shaderc::Compiler::new().ok_or_else(|| log_at(LogLevel::Error, "shaderc compiler init failed"))
}

fn new_compile_options() -> Result<shaderc::CompileOptions<'static>, ()> {
    shaderc::CompileOptions::new().ok_or_else(|| log_at(LogLevel::Error, "shaderc options init failed"))
}

fn run_compile(
    c: &shaderc::Compiler,
    o: &shaderc::CompileOptions,
    vk_src: &str,
    kind: shaderc::ShaderKind,
) -> Result<Vec<u32>, ()> {
    c.compile_into_spirv(vk_src, kind, "bones.frag", "main", Some(o))
        .map(|a| a.as_binary().to_vec())
        .map_err(|e| log_at(LogLevel::Error, &format!("spirv compile failed: {}", e)))
}

fn compile_vk_spirv(vk_src: &str, kind: shaderc::ShaderKind) -> Result<Vec<u32>, ()> {
    new_compiler().and_then(|c| new_compile_options().and_then(|o| run_compile(&c, &o, vk_src, kind)))
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
    compile_vk_spirv(VERT_VK_SRC, shaderc::ShaderKind::Vertex)
        .map(|v| { store_vert_spirv(v.clone()); v })
}

pub(crate) fn compile_vert_spirv() -> Result<Vec<u32>, ()> {
    match cached_vert_spirv() {
        Some(v) => Ok(v),
        None => compile_and_cache_vert(),
    }
}

pub(crate) fn compile_frag_spirv(vk_src: &str) -> Result<Vec<u32>, ()> {
    compile_vk_spirv(vk_src, shaderc::ShaderKind::Fragment)
}

pub(crate) fn build_shaders(s: &Settings, reg: &[EffectDef]) -> (String, Vec<u32>) {
    let gl = build_gl_source(s, reg);
    let spv = compile_frag_spirv(&rewrite_gl_to_vk(&gl)).unwrap_or_default();
    (gl, spv)
}

fn store_gl_shader(gl: String) {
    match SHADER_GL.write() {
        Ok(mut g) => *g = Some(gl),
        Err(_) => (),
    }
}

fn store_spv_shader(spv: Vec<u32>) {
    match SHADER_SPV.write() {
        Ok(mut g) => *g = Some(spv),
        Err(_) => (),
    }
}

pub(crate) fn store_shaders(gl: String, spv: Vec<u32>) {
    store_gl_shader(gl);
    store_spv_shader(spv);
    GENERATION.fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn current_gl_shader() -> String {
    SHADER_GL
        .read()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| build_gl_source(&default_settings(), &REGISTRY))
}

pub(crate) fn current_spv() -> Vec<u32> {
    SHADER_SPV.read().ok().and_then(|g| g.clone()).unwrap_or_default()
}
