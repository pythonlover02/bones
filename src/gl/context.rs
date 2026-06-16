use std::collections::HashMap;
use std::ffi::c_void;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

use crate::consts::*;
use crate::consts::VERT_SRC;
use crate::logging::log_at;
use crate::logging::LogLevel;
use crate::shader::current_gl_shader;
use crate::shader::GENERATION;
use crate::util::cstr_to_str;

use super::fns::gl_fns;

#[derive(Default, Clone, Copy)]
pub(crate) struct UniformLocations {
    pub(crate) input: i32,
    pub(crate) history: i32,
    pub(crate) resolution: i32,
    pub(crate) time: i32,
    pub(crate) fps: i32,
}

#[derive(Default, Clone, Copy)]
pub(crate) struct CtxState {
    pub(crate) fbo_checked: bool,
    pub(crate) fbo_supported: bool,
    pub(crate) postfx_enabled: bool,
    pub(crate) program: u32,
    pub(crate) vbo: u32,
    pub(crate) vao: u32,
    pub(crate) tex_input: u32,
    pub(crate) fbo_history: u32,
    pub(crate) tex_history: u32,
    pub(crate) locs: UniformLocations,
    pub(crate) w: i32,
    pub(crate) h: i32,
    pub(crate) gen: i32,
}

static CTXS: Mutex<Option<HashMap<u64, CtxState>>> = Mutex::new(None);

pub(crate) fn ctx_take(key: u64) -> CtxState {
    CTXS.lock()
        .ok()
        .and_then(|mut g| g.get_or_insert_with(HashMap::new).remove(&key))
        .unwrap_or_default()
}

pub(crate) fn ctx_store(key: u64, st: CtxState) {
    match CTXS.lock() {
        Ok(mut g) => {
            g.get_or_insert_with(HashMap::new).insert(key, st);
        }
        Err(_) => (),
    }
}

fn compile_gl_shader(kind: u32, src: &str) -> Option<u32> {
    let f = gl_fns();
    let sh = unsafe { (f.create_shader)(kind) };
    let c = CString::new(src).unwrap_or_default();
    let p = c.as_ptr();
    let mut ok: i32 = 0;
    unsafe {
        (f.shader_source)(sh, 1, &p, ptr::null());
        (f.compile_shader)(sh);
        (f.get_shaderiv)(sh, GL_COMPILE_STATUS, &mut ok);
    }
    match ok {
        0 => {
            unsafe { (f.delete_shader)(sh) };
            log_at(LogLevel::Error, "gl shader compile failed");
            None
        }
        _ => Some(sh),
    }
}

fn link_gl_program(vs: u32, fs: u32) -> Option<u32> {
    let f = gl_fns();
    let prog = unsafe { (f.create_program)() };
    let attr = CString::new("a_pos").unwrap_or_default();
    let mut ok: i32 = 0;
    unsafe {
        (f.attach_shader)(prog, vs);
        (f.attach_shader)(prog, fs);
        (f.bind_attrib_location)(prog, 0, attr.as_ptr());
        (f.link_program)(prog);
        (f.get_programiv)(prog, GL_LINK_STATUS, &mut ok);
        (f.delete_shader)(vs);
        (f.delete_shader)(fs);
    }
    match ok {
        0 => {
            unsafe { (f.delete_program)(prog) };
            log_at(LogLevel::Error, "gl program link failed");
            None
        }
        _ => Some(prog),
    }
}

fn uniform_loc(prog: u32, name: &str) -> i32 {
    let c = CString::new(name).unwrap_or_default();
    unsafe { (gl_fns().get_uniform_location)(prog, c.as_ptr()) }
}

fn build_gl_program(gl_src: &str) -> Option<(u32, UniformLocations)> {
    compile_gl_shader(GL_VERTEX_SHADER, VERT_SRC)
        .and_then(|vs| compile_gl_shader(GL_FRAGMENT_SHADER, gl_src).and_then(|fs| link_gl_program(vs, fs)))
        .map(|prog| {
            (
                prog,
                UniformLocations {
                    input: uniform_loc(prog, "u_input"),
                    history: uniform_loc(prog, "u_history"),
                    resolution: uniform_loc(prog, "u_resolution"),
                    time: uniform_loc(prog, "u_time"),
                    fps: uniform_loc(prog, "u_fps"),
                },
            )
        })
}

fn gl_ver_major(ver: &str) -> u8 {
    ver.split(' ')
        .next()
        .and_then(|t| t.split('.').next())
        .and_then(|maj| maj.parse().ok())
        .unwrap_or(0)
}

fn gl_has_fbo() -> bool {
    let ver = cstr_to_str(unsafe { (gl_fns().get_string)(GL_VERSION) });
    ver.starts_with("OpenGL ES") || gl_ver_major(ver) >= 3
}

fn gl_vao_ptr_valid() -> bool {
    gl_fns().gen_vertex_arrays as usize != 0
        && gl_fns().bind_vertex_array as usize != 0
}

pub(crate) fn alloc_tex(w: i32, h: i32) -> u32 {
    let f = gl_fns();
    let mut t: u32 = 0;
    unsafe {
        (f.gen_textures)(1, &mut t);
        (f.bind_texture)(GL_TEXTURE_2D, t);
        (f.tex_parameteri)(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        (f.tex_parameteri)(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
        (f.tex_parameteri)(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
        (f.tex_parameteri)(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
        (f.tex_image_2d)(GL_TEXTURE_2D, 0, GL_RGBA8, w, h, 0, GL_RGBA, GL_UNSIGNED_BYTE, ptr::null());
    }
    t
}

pub(crate) fn destroy_targets(st: &mut CtxState) {
    let f = gl_fns();
    let texs = [st.tex_input, st.tex_history];
    let fbos = [st.fbo_history];
    unsafe {
        (f.delete_textures)(2, texs.as_ptr());
        (f.delete_framebuffers)(1, fbos.as_ptr());
    }
    st.tex_input = 0;
    st.tex_history = 0;
    st.fbo_history = 0;
}

fn drain_gl_errors() {
    let f = gl_fns();
    std::iter::from_fn(|| match unsafe { (f.get_error)() } {
        0 => None,
        _ => Some(()),
    })
    .for_each(|_| ());
}

pub(crate) fn alloc_targets(st: &mut CtxState, w: i32, h: i32) -> bool {
    drain_gl_errors();
    let f = gl_fns();
    st.tex_input = alloc_tex(w, h);
    st.tex_history = alloc_tex(w, h);
    let mut fbo: u32 = 0;
    unsafe {
        (f.gen_framebuffers)(1, &mut fbo);
        (f.bind_framebuffer)(GL_FRAMEBUFFER, fbo);
        (f.framebuffer_texture_2d)(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, st.tex_history, 0);
        (f.clear_color)(0.0, 0.0, 0.0, 1.0);
        (f.clear)(GL_COLOR_BUFFER_BIT);
        (f.bind_framebuffer)(GL_FRAMEBUFFER, 0);
    }
    st.fbo_history = fbo;
    st.w = w;
    st.h = h;
    let err = unsafe { (f.get_error)() };
    match err {
        0 => true,
        _ => {
            log_at(LogLevel::Warn, "gl target allocation failed, will retry next frame");
            destroy_targets(st);
            false
        }
    }
}

fn call_create_vbo() -> (u32, u32) {
    let f = gl_fns();
    let mut prev_vao: i32 = 0;
    let mut vao: u32 = 0;
    let mut vbo: u32 = 0;
    unsafe {
        (f.get_integerv)(GL_VERTEX_ARRAY_BINDING, &mut prev_vao);
        (f.gen_vertex_arrays)(1, &mut vao);
        (f.bind_vertex_array)(vao);
        (f.gen_buffers)(1, &mut vbo);
        (f.bind_buffer)(GL_ARRAY_BUFFER, vbo);
        (f.buffer_data)(
            GL_ARRAY_BUFFER,
            (TRI_VERTS.len() * mem::size_of::<f32>()) as isize,
            TRI_VERTS.as_ptr() as *const c_void,
            GL_STATIC_DRAW,
        );
        (f.vertex_attrib_pointer)(0, VBO_COMPONENTS, GL_FLOAT, 0, 0, ptr::null());
        (f.enable_vertex_attrib_array)(0);
        (f.bind_vertex_array)(prev_vao as u32);
    }
    (vbo, vao)
}

fn fbo_status_message(supported: bool) -> &'static str {
    match supported {
        true => "GL_EXT_framebuffer_object present, postfx enabled",
        false => "GL_EXT_framebuffer_object missing, postfx disabled for context",
    }
}

fn check_fbo_caps(st: &mut CtxState) {
    st.fbo_checked = true;
    st.fbo_supported = gl_has_fbo() && gl_vao_ptr_valid();
    st.postfx_enabled = st.fbo_supported;
    log_at(LogLevel::Info, fbo_status_message(st.fbo_supported));
}

pub(crate) fn ensure_ctx_caps(st: &mut CtxState) {
    match st.fbo_checked {
        true => (),
        false => check_fbo_caps(st),
    }
}

fn needs_rebuild(program: u32, st_gen: i32, cur_gen: i32) -> bool {
    program == 0 || st_gen != cur_gen
}

pub(crate) fn ensure_ctx_program(st: &mut CtxState) {
    let gen = GENERATION.load(Ordering::Relaxed);
    match needs_rebuild(st.program, st.gen, gen) {
        true => rebuild_ctx_program(st, gen),
        false => (),
    }
}

fn call_delete_program_if_exists(prog: u32) {
    match prog {
        0 => (),
        p => unsafe { (gl_fns().delete_program)(p) },
    }
}

fn needs_vbo(vbo: u32) -> bool {
    vbo == 0
}

fn rebuild_ctx_program(st: &mut CtxState, gen: i32) {
    match build_gl_program(&current_gl_shader()) {
        Some((prog, locs)) => {
            call_delete_program_if_exists(st.program);
            st.program = prog;
            st.locs = locs;
            st.gen = gen;
        }
        None => {
            log_at(LogLevel::Error, "gl program compile failed, keeping previous shader");
            st.gen = gen;
        }
    }
    match needs_vbo(st.vbo) {
        true => { let (vbo, vao) = call_create_vbo(); st.vbo = vbo; st.vao = vao; }
        false => (),
    }
}

pub(crate) fn ensure_ctx_targets(st: &mut CtxState, w: i32, h: i32) -> bool {
    match (st.tex_input != 0, st.w == w && st.h == h) {
        (true, true) => true,
        (true, false) => {
            destroy_targets(st);
            alloc_targets(st, w, h)
        }
        (false, _) => alloc_targets(st, w, h),
    }
}

pub(crate) fn ctx_ready(st: &mut CtxState, w: i32, h: i32) -> bool {
    match st.postfx_enabled && w > 0 && h > 0 {
        true => {
            ensure_ctx_program(st);
            st.program != 0 && ensure_ctx_targets(st, w, h)
        }
        false => false,
    }
}
