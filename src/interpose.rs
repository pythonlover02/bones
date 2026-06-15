use std::ffi::c_char;
use std::ffi::c_void;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::sync::OnceLock;

use crate::consts::DLSYM_VERSION;
use crate::consts::EGL_HEIGHT;
use crate::consts::EGL_WIDTH;
use crate::consts::GLX_HEIGHT;
use crate::consts::GLX_WIDTH;
use crate::util::cstr_to_str;

pub(crate) type DlsymFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void;
pub(crate) type GlxSwapFn = unsafe extern "C" fn(*mut c_void, libc::c_ulong);
pub(crate) type EglSwapFn = unsafe extern "C" fn(*mut c_void, *mut c_void) -> u32;
pub(crate) type EglSwapDamageFn = unsafe extern "C" fn(*mut c_void, *mut c_void, *const i32, i32) -> u32;
pub(crate) type GpaFn = unsafe extern "C" fn(*const c_char) -> *mut c_void;
type CurrentFn = unsafe extern "C" fn() -> *mut c_void;
pub(crate) type GlxQueryDrawableFn = unsafe extern "C" fn(*mut c_void, libc::c_ulong, i32, *mut u32);
pub(crate) type EglQuerySurfaceFn = unsafe extern "C" fn(*mut c_void, *mut c_void, i32, *mut i32) -> u32;

#[derive(Default)]
pub(crate) struct Reals {
    pub(crate) glx_swap: OnceLock<GlxSwapFn>,
    pub(crate) egl_swap: OnceLock<EglSwapFn>,
    pub(crate) egl_swap_damage_ext: OnceLock<EglSwapDamageFn>,
    pub(crate) egl_swap_damage_khr: OnceLock<EglSwapDamageFn>,
    pub(crate) glx_gpa: OnceLock<GpaFn>,
    pub(crate) egl_gpa: OnceLock<GpaFn>,
    glx_get_current: OnceLock<CurrentFn>,
    egl_get_current: OnceLock<CurrentFn>,
    pub(crate) glx_query_drawable: OnceLock<GlxQueryDrawableFn>,
    pub(crate) egl_query_surface: OnceLock<EglQuerySurfaceFn>,
    pub(crate) gl_fns: OnceLock<crate::gl::fns::GlFns>,
}

static REAL_DLSYM: OnceLock<DlsymFn> = OnceLock::new();
static REALS: OnceLock<Reals> = OnceLock::new();

fn real_dlsym() -> DlsymFn {
    *REAL_DLSYM.get_or_init(|| unsafe {
        let p = libc::dlvsym(
            libc::RTLD_NEXT,
            b"dlsym\0".as_ptr() as *const c_char,
            DLSYM_VERSION.as_ptr() as *const c_char,
        );
        let q = match p.is_null() {
            true => libc::dlsym(libc::RTLD_NEXT, b"dlsym\0".as_ptr() as *const c_char),
            false => p,
        };
        mem::transmute::<*mut c_void, DlsymFn>(q)
    })
}

pub(crate) fn call_real_dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void {
    unsafe { (real_dlsym())(handle, name) }
}

pub(crate) fn call_dlopen(name: &str) -> *mut c_void {
    let c = CString::new(name).unwrap_or_default();
    unsafe { libc::dlopen(c.as_ptr(), libc::RTLD_LAZY | libc::RTLD_LOCAL) }
}

pub(crate) fn lib_sym(lib: &str, sym: &str) -> *mut c_void {
    let h = call_dlopen(lib);
    let c = CString::new(sym).unwrap_or_default();
    match h.is_null() {
        true => ptr::null_mut(),
        false => call_real_dlsym(h, c.as_ptr()),
    }
}

pub(crate) fn reals() -> &'static Reals {
    REALS.get_or_init(Reals::default)
}

pub(crate) fn real_glx_swap() -> GlxSwapFn {
    *reals().glx_swap.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, GlxSwapFn>(lib_sym("libGLX.so.0", "glXSwapBuffers"))
    })
}

pub(crate) fn real_egl_swap() -> EglSwapFn {
    *reals().egl_swap.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, EglSwapFn>(lib_sym("libEGL.so.1", "eglSwapBuffers"))
    })
}

pub(crate) fn real_egl_swap_damage_ext() -> EglSwapDamageFn {
    *reals().egl_swap_damage_ext.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, EglSwapDamageFn>(lib_sym("libEGL.so.1", "eglSwapBuffersWithDamageEXT"))
    })
}

pub(crate) fn real_egl_swap_damage_khr() -> EglSwapDamageFn {
    *reals().egl_swap_damage_khr.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, EglSwapDamageFn>(lib_sym("libEGL.so.1", "eglSwapBuffersWithDamageKHR"))
    })
}

pub(crate) fn real_glx_gpa() -> GpaFn {
    *reals().glx_gpa.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, GpaFn>(lib_sym("libGLX.so.0", "glXGetProcAddressARB"))
    })
}

pub(crate) fn real_egl_gpa() -> GpaFn {
    *reals().egl_gpa.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, GpaFn>(lib_sym("libEGL.so.1", "eglGetProcAddress"))
    })
}

pub(crate) fn call_real_glx_gpa(name: *const c_char) -> *mut c_void {
    unsafe { (real_glx_gpa())(name) }
}

pub(crate) fn call_real_egl_gpa(name: *const c_char) -> *mut c_void {
    unsafe { (real_egl_gpa())(name) }
}

pub(crate) fn call_real_glx_swap(dpy: *mut c_void, drawable: libc::c_ulong) {
    unsafe { (real_glx_swap())(dpy, drawable) }
}

pub(crate) fn call_real_egl_swap(dpy: *mut c_void, surface: *mut c_void) -> u32 {
    unsafe { (real_egl_swap())(dpy, surface) }
}

pub(crate) fn call_real_egl_swap_damage_ext(dpy: *mut c_void, surf: *mut c_void, rects: *const i32, n: i32) -> u32 {
    unsafe { (real_egl_swap_damage_ext())(dpy, surf, rects, n) }
}

pub(crate) fn call_real_egl_swap_damage_khr(dpy: *mut c_void, surf: *mut c_void, rects: *const i32, n: i32) -> u32 {
    unsafe { (real_egl_swap_damage_khr())(dpy, surf, rects, n) }
}

pub(crate) fn gl_lookup(name: &str) -> *mut c_void {
    let c = CString::new(name).unwrap_or_default();
    let p = call_real_glx_gpa(c.as_ptr());
    match p.is_null() {
        true => {
            let q = call_real_egl_gpa(c.as_ptr());
            match q.is_null() {
                true => lib_sym("libGL.so.1", name),
                false => q,
            }
        }
        false => p,
    }
}

pub(crate) fn call_glx_current_ctx() -> *mut c_void {
    let f = *reals().glx_get_current.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, CurrentFn>(lib_sym("libGLX.so.0", "glXGetCurrentContext"))
    });
    unsafe { f() }
}

pub(crate) fn call_egl_current_ctx() -> *mut c_void {
    let f = *reals().egl_get_current.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, CurrentFn>(lib_sym("libEGL.so.1", "eglGetCurrentContext"))
    });
    unsafe { f() }
}

pub(crate) fn call_glx_drawable_size(dpy: *mut c_void, drawable: libc::c_ulong) -> (i32, i32) {
    let f = *reals().glx_query_drawable.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, GlxQueryDrawableFn>(lib_sym("libGLX.so.0", "glXQueryDrawable"))
    });
    let mut w: u32 = 0;
    let mut h: u32 = 0;
    unsafe {
        f(dpy, drawable, GLX_WIDTH, &mut w);
        f(dpy, drawable, GLX_HEIGHT, &mut h);
    }
    (w as i32, h as i32)
}

pub(crate) fn call_egl_surface_size(dpy: *mut c_void, surf: *mut c_void) -> (i32, i32) {
    let f = *reals().egl_query_surface.get_or_init(|| unsafe {
        mem::transmute::<*mut c_void, EglQuerySurfaceFn>(lib_sym("libEGL.so.1", "eglQuerySurface"))
    });
    let mut w: i32 = 0;
    let mut h: i32 = 0;
    unsafe {
        f(dpy, surf, EGL_WIDTH, &mut w);
        f(dpy, surf, EGL_HEIGHT, &mut h);
    }
    (w, h)
}

pub(crate) fn hooked_symbol(name: &str) -> Option<*mut c_void> {
    use crate::gl::swap;
    match name {
        "glXSwapBuffers" => Some(swap::glXSwapBuffers as *mut c_void),
        "eglSwapBuffers" => Some(swap::eglSwapBuffers as *mut c_void),
        "eglSwapBuffersWithDamageEXT" => Some(swap::eglSwapBuffersWithDamageEXT as *mut c_void),
        "eglSwapBuffersWithDamageKHR" => Some(swap::eglSwapBuffersWithDamageKHR as *mut c_void),
        "glXGetProcAddress" => Some(glXGetProcAddress as *mut c_void),
        "glXGetProcAddressARB" => Some(glXGetProcAddressARB as *mut c_void),
        "eglGetProcAddress" => Some(eglGetProcAddress as *mut c_void),
        _ => None,
    }
}

#[no_mangle]
pub unsafe extern "C" fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void {
    match hooked_symbol(cstr_to_str(name)) {
        Some(p) => p,
        None => call_real_dlsym(handle, name),
    }
}

#[no_mangle]
pub unsafe extern "C" fn glXGetProcAddress(name: *const c_char) -> *mut c_void {
    match hooked_symbol(cstr_to_str(name)) {
        Some(p) => p,
        None => call_real_glx_gpa(name),
    }
}

#[no_mangle]
pub unsafe extern "C" fn glXGetProcAddressARB(name: *const c_char) -> *mut c_void {
    match hooked_symbol(cstr_to_str(name)) {
        Some(p) => p,
        None => call_real_glx_gpa(name),
    }
}

#[no_mangle]
pub unsafe extern "C" fn eglGetProcAddress(name: *const c_char) -> *mut c_void {
    match hooked_symbol(cstr_to_str(name)) {
        Some(p) => p,
        None => call_real_egl_gpa(name),
    }
}
