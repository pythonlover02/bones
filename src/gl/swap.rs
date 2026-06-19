use std::ffi::c_void;
use std::ptr;

use crate::config::ensure_settings;
use crate::config::Settings;
use crate::consts::EffectDef;
use crate::consts::REGISTRY;
use crate::effect::any_effect_enabled;
use crate::interpose::call_egl_current_ctx;
use crate::interpose::call_egl_surface_size;
use crate::interpose::call_glx_current_ctx;
use crate::interpose::call_glx_drawable_size;
use crate::interpose::call_real_egl_create_context;
use crate::interpose::call_real_egl_destroy_context;
use crate::interpose::call_real_egl_swap;
use crate::interpose::call_real_egl_swap_damage_ext;
use crate::interpose::call_real_egl_swap_damage_khr;
use crate::interpose::call_real_egl_terminate;
use crate::interpose::call_real_glx_destroy_context;
use crate::interpose::call_real_glx_swap;
use crate::watch::maybe_reload;

use super::context::ctx_ready;
use super::context::ctx_remove;
use super::context::ctx_store;
use super::context::ctx_take;
use super::context::destroy_ctx_state;
use super::context::display_ctx_add;
use super::context::display_ctx_drain;
use super::context::display_ctx_remove;
use super::context::ensure_ctx_caps;
use super::postfx::draw_postfx_gl;
use super::postfx::restore_gl_state;
use super::postfx::save_gl_state;

fn ctx_key() -> u64 {
    let g = call_glx_current_ctx();
    match g.is_null() {
        true => call_egl_current_ctx() as u64,
        false => g as u64,
    }
}

fn want_postfx(ctx: u64, s: &Settings, reg: &[EffectDef]) -> bool {
    ctx != 0 && any_effect_enabled(s, reg)
}

fn run_postfx_at(key: u64, w: i32, h: i32) {
    let mut st = ctx_take(key);
    ensure_ctx_caps(&mut st);
    match ctx_ready(&mut st, w, h) {
        true => {
            let saved = save_gl_state();
            draw_postfx_gl(&st, w, h);
            restore_gl_state(&saved);
        }
        false => (),
    }
    ctx_store(key, st);
}

fn run_postfx_glx(dpy: *mut c_void, drawable: libc::c_ulong) {
    let (w, h) = call_glx_drawable_size(dpy, drawable);
    run_postfx_at(ctx_key(), w, h);
    call_real_glx_swap(dpy, drawable);
}

fn run_postfx_egl(dpy: *mut c_void, surf: *mut c_void) {
    let (w, h) = call_egl_surface_size(dpy, surf);
    run_postfx_at(call_egl_current_ctx() as u64, w, h);
}

#[no_mangle]
pub unsafe extern "C" fn glXSwapBuffers(dpy: *mut c_void, drawable: libc::c_ulong) {
    maybe_reload();
    match want_postfx(ctx_key(), &ensure_settings(), &REGISTRY) {
        true => run_postfx_glx(dpy, drawable),
        false => call_real_glx_swap(dpy, drawable),
    }
}

#[no_mangle]
pub unsafe extern "C" fn eglSwapBuffers(dpy: *mut c_void, surface: *mut c_void) -> u32 {
    maybe_reload();
    match want_postfx(call_egl_current_ctx() as u64, &ensure_settings(), &REGISTRY) {
        true => { run_postfx_egl(dpy, surface); call_real_egl_swap(dpy, surface) }
        false => call_real_egl_swap(dpy, surface),
    }
}

#[no_mangle]
pub unsafe extern "C" fn eglSwapBuffersWithDamageEXT(
    dpy: *mut c_void, surf: *mut c_void, rects: *const i32, n: i32,
) -> u32 {
    maybe_reload();
    match want_postfx(call_egl_current_ctx() as u64, &ensure_settings(), &REGISTRY) {
        true => { run_postfx_egl(dpy, surf); call_real_egl_swap_damage_ext(dpy, surf, ptr::null(), 0) }
        false => call_real_egl_swap_damage_ext(dpy, surf, rects, n),
    }
}

#[no_mangle]
pub unsafe extern "C" fn eglSwapBuffersWithDamageKHR(
    dpy: *mut c_void, surf: *mut c_void, rects: *const i32, n: i32,
) -> u32 {
    maybe_reload();
    match want_postfx(call_egl_current_ctx() as u64, &ensure_settings(), &REGISTRY) {
        true => { run_postfx_egl(dpy, surf); call_real_egl_swap_damage_khr(dpy, surf, ptr::null(), 0) }
        false => call_real_egl_swap_damage_khr(dpy, surf, rects, n),
    }
}

#[no_mangle]
pub unsafe extern "C" fn glXDestroyContext(dpy: *mut c_void, ctx: *mut c_void) {
    ctx_remove(ctx as u64).into_iter().for_each(destroy_ctx_state);
    call_real_glx_destroy_context(dpy, ctx);
}

#[no_mangle]
pub unsafe extern "C" fn eglCreateContext(dpy: *mut c_void, config: *mut c_void, share: *mut c_void, attribs: *const i32) -> *mut c_void {
    let ctx = call_real_egl_create_context(dpy, config, share, attribs);
    match ctx.is_null() {
        true => ctx,
        false => {
            display_ctx_add(dpy as u64, ctx as u64);
            ctx
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn eglDestroyContext(dpy: *mut c_void, ctx: *mut c_void) -> u32 {
    display_ctx_remove(dpy as u64, ctx as u64);
    ctx_remove(ctx as u64).into_iter().for_each(destroy_ctx_state);
    call_real_egl_destroy_context(dpy, ctx)
}

#[no_mangle]
pub unsafe extern "C" fn eglTerminate(dpy: *mut c_void) -> u32 {
    display_ctx_drain(dpy as u64).into_iter()
        .filter_map(ctx_remove)
        .for_each(destroy_ctx_state);
    call_real_egl_terminate(dpy)
}
