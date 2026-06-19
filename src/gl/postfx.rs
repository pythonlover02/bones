use crate::consts::*;
use crate::consts::FULLSCREEN_TRI_VERTS;
use crate::timing::frame_time_fps;

use super::context::CtxState;
use super::fns::gl_fns;

#[derive(Default, Clone, Copy)]
pub(crate) struct GlState {
    pub(crate) draw_fbo: i32,
    pub(crate) read_fbo: i32,
    pub(crate) vao_binding: i32,
    pub(crate) program: i32,
    pub(crate) active_tex: i32,
    pub(crate) tex0: i32,
    pub(crate) tex1: i32,
    pub(crate) pixel_unpack: i32,
    pub(crate) array_buffer: i32,
    pub(crate) attrib0_enabled: i32,
    pub(crate) viewport: [i32; 4],
    pub(crate) scissor: [i32; 4],
    pub(crate) color_mask: [u8; 4],
    pub(crate) clear_color: [f32; 4],
    pub(crate) depth_mask: u8,
    pub(crate) stencil_mask: i32,
    pub(crate) unpack_alignment: i32,
    pub(crate) unpack_row_length: i32,
    pub(crate) blend_src_rgb: i32,
    pub(crate) blend_dst_rgb: i32,
    pub(crate) blend_src_a: i32,
    pub(crate) blend_dst_a: i32,
    pub(crate) blend_eq_rgb: i32,
    pub(crate) blend_eq_a: i32,
    pub(crate) en_depth: u8,
    pub(crate) en_blend: u8,
    pub(crate) en_cull: u8,
    pub(crate) en_scissor: u8,
    pub(crate) en_stencil: u8,
    pub(crate) en_srgb: u8,
    pub(crate) en_raster_discard: u8,
    pub(crate) en_alpha_test: u8,
    pub(crate) en_logic_op: u8,
    pub(crate) en_vp_arb: u8,
    pub(crate) en_fp_arb: u8,
    pub(crate) en_tex2d: u8,
}

pub(crate) fn save_gl_state() -> GlState {
    let f = gl_fns();
    let mut s = GlState::default();
    unsafe {
        (f.get_integerv)(GL_DRAW_FRAMEBUFFER_BINDING, &mut s.draw_fbo);
        (f.get_integerv)(GL_READ_FRAMEBUFFER_BINDING, &mut s.read_fbo);
        (f.get_integerv)(GL_CURRENT_PROGRAM, &mut s.program);
        (f.get_integerv)(GL_VERTEX_ARRAY_BINDING, &mut s.vao_binding);
        (f.get_integerv)(GL_ACTIVE_TEXTURE, &mut s.active_tex);
        (f.active_texture)(GL_TEXTURE0);
        (f.get_integerv)(GL_TEXTURE_BINDING_2D, &mut s.tex0);
        (f.active_texture)(GL_TEXTURE1);
        (f.get_integerv)(GL_TEXTURE_BINDING_2D, &mut s.tex1);
        (f.get_integerv)(GL_PIXEL_UNPACK_BUFFER_BINDING, &mut s.pixel_unpack);
        (f.get_integerv)(GL_ARRAY_BUFFER_BINDING, &mut s.array_buffer);
        (f.get_vertex_attribiv)(0, GL_VERTEX_ATTRIB_ARRAY_ENABLED, &mut s.attrib0_enabled);
        (f.get_integerv)(GL_VIEWPORT, s.viewport.as_mut_ptr());
        (f.get_integerv)(GL_SCISSOR_BOX, s.scissor.as_mut_ptr());
        (f.get_booleanv)(GL_COLOR_WRITEMASK, s.color_mask.as_mut_ptr());
        (f.get_floatv)(GL_COLOR_CLEAR_VALUE, s.clear_color.as_mut_ptr());
        (f.get_booleanv)(GL_DEPTH_WRITEMASK, &mut s.depth_mask);
        (f.get_integerv)(GL_STENCIL_WRITEMASK, &mut s.stencil_mask);
        (f.get_integerv)(GL_UNPACK_ALIGNMENT, &mut s.unpack_alignment);
        (f.get_integerv)(GL_UNPACK_ROW_LENGTH, &mut s.unpack_row_length);
        (f.get_integerv)(GL_BLEND_SRC_RGB, &mut s.blend_src_rgb);
        (f.get_integerv)(GL_BLEND_DST_RGB, &mut s.blend_dst_rgb);
        (f.get_integerv)(GL_BLEND_SRC_ALPHA, &mut s.blend_src_a);
        (f.get_integerv)(GL_BLEND_DST_ALPHA, &mut s.blend_dst_a);
        (f.get_integerv)(GL_BLEND_EQUATION_RGB, &mut s.blend_eq_rgb);
        (f.get_integerv)(GL_BLEND_EQUATION_ALPHA, &mut s.blend_eq_a);
        s.en_depth = (f.is_enabled)(GL_DEPTH_TEST);
        s.en_blend = (f.is_enabled)(GL_BLEND);
        s.en_cull = (f.is_enabled)(GL_CULL_FACE);
        s.en_scissor = (f.is_enabled)(GL_SCISSOR_TEST);
        s.en_stencil = (f.is_enabled)(GL_STENCIL_TEST);
        s.en_srgb = (f.is_enabled)(GL_FRAMEBUFFER_SRGB);
        s.en_raster_discard = (f.is_enabled)(GL_RASTERIZER_DISCARD);
        s.en_alpha_test = (f.is_enabled)(GL_ALPHA_TEST);
        s.en_logic_op = (f.is_enabled)(GL_COLOR_LOGIC_OP);
        s.en_vp_arb = (f.is_enabled)(GL_VERTEX_PROGRAM_ARB);
        s.en_fp_arb = (f.is_enabled)(GL_FRAGMENT_PROGRAM_ARB);
        s.en_tex2d = (f.is_enabled)(GL_TEXTURE_2D);
        (f.active_texture)(s.active_tex as u32);
    }
    s
}

fn set_cap(cap: u32, on: u8) {
    let f = gl_fns();
    match on {
        0 => unsafe { (f.disable)(cap) },
        _ => unsafe { (f.enable)(cap) },
    }
}

fn set_attrib0(on: i32) {
    let f = gl_fns();
    match on {
        0 => unsafe { (f.disable_vertex_attrib_array)(0) },
        _ => unsafe { (f.enable_vertex_attrib_array)(0) },
    }
}

pub(crate) fn restore_gl_state(s: &GlState) {
    let f = gl_fns();
    unsafe {
        (f.bind_framebuffer)(GL_DRAW_FRAMEBUFFER, s.draw_fbo as u32);
        (f.bind_framebuffer)(GL_READ_FRAMEBUFFER, s.read_fbo as u32);
        (f.bind_vertex_array)(s.vao_binding as u32);
        (f.use_program)(s.program as u32);
        (f.active_texture)(GL_TEXTURE0);
        (f.bind_texture)(GL_TEXTURE_2D, s.tex0 as u32);
        (f.active_texture)(GL_TEXTURE1);
        (f.bind_texture)(GL_TEXTURE_2D, s.tex1 as u32);
        (f.active_texture)(s.active_tex as u32);
        (f.bind_buffer)(GL_PIXEL_UNPACK_BUFFER, s.pixel_unpack as u32);
        (f.bind_buffer)(GL_ARRAY_BUFFER, s.array_buffer as u32);
        (f.viewport)(s.viewport[0], s.viewport[1], s.viewport[2], s.viewport[3]);
        (f.scissor)(s.scissor[0], s.scissor[1], s.scissor[2], s.scissor[3]);
        (f.color_mask)(s.color_mask[0], s.color_mask[1], s.color_mask[2], s.color_mask[3]);
        (f.clear_color)(s.clear_color[0], s.clear_color[1], s.clear_color[2], s.clear_color[3]);
        (f.depth_mask)(s.depth_mask);
        (f.stencil_mask)(s.stencil_mask as u32);
        (f.pixel_storei)(GL_UNPACK_ALIGNMENT, s.unpack_alignment);
        (f.pixel_storei)(GL_UNPACK_ROW_LENGTH, s.unpack_row_length);
        (f.blend_func_separate)(
            s.blend_src_rgb as u32, s.blend_dst_rgb as u32,
            s.blend_src_a as u32, s.blend_dst_a as u32,
        );
        (f.blend_equation_separate)(s.blend_eq_rgb as u32, s.blend_eq_a as u32);
    }
    set_attrib0(s.attrib0_enabled);
    set_cap(GL_DEPTH_TEST, s.en_depth);
    set_cap(GL_BLEND, s.en_blend);
    set_cap(GL_CULL_FACE, s.en_cull);
    set_cap(GL_SCISSOR_TEST, s.en_scissor);
    set_cap(GL_STENCIL_TEST, s.en_stencil);
    set_cap(GL_FRAMEBUFFER_SRGB, s.en_srgb);
    set_cap(GL_RASTERIZER_DISCARD, s.en_raster_discard);
    set_cap(GL_ALPHA_TEST, s.en_alpha_test);
    set_cap(GL_COLOR_LOGIC_OP, s.en_logic_op);
    set_cap(GL_VERTEX_PROGRAM_ARB, s.en_vp_arb);
    set_cap(GL_FRAGMENT_PROGRAM_ARB, s.en_fp_arb);
    set_cap(GL_TEXTURE_2D, s.en_tex2d);
}

pub(crate) fn draw_postfx_gl(st: &CtxState, w: i32, h: i32) {
    let f = gl_fns();
    let (t, fps) = frame_time_fps();
    unsafe {
        (f.active_texture)(GL_TEXTURE0);
        (f.bind_texture)(GL_TEXTURE_2D, st.tex_input);
        (f.bind_framebuffer)(GL_READ_FRAMEBUFFER, 0);
        (f.copy_tex_sub_image_2d)(GL_TEXTURE_2D, 0, 0, 0, 0, 0, w, h);
        (f.bind_framebuffer)(GL_DRAW_FRAMEBUFFER, 0);
        (f.disable)(GL_DEPTH_TEST);
        (f.disable)(GL_BLEND);
        (f.disable)(GL_CULL_FACE);
        (f.disable)(GL_SCISSOR_TEST);
        (f.disable)(GL_STENCIL_TEST);
        (f.disable)(GL_FRAMEBUFFER_SRGB);
        (f.disable)(GL_RASTERIZER_DISCARD);
        (f.disable)(GL_ALPHA_TEST);
        (f.disable)(GL_COLOR_LOGIC_OP);
        (f.color_mask)(1, 1, 1, 1);
        (f.depth_mask)(0);
        (f.viewport)(0, 0, w, h);
        (f.use_program)(st.program);
        (f.uniform1i)(st.locs.input, 0);
        (f.uniform1i)(st.locs.history, 1);
        (f.uniform2f)(st.locs.resolution, w as f32, h as f32);
        (f.uniform1f)(st.locs.time, t);
        (f.uniform1f)(st.locs.fps, fps);
        (f.active_texture)(GL_TEXTURE1);
        (f.bind_texture)(GL_TEXTURE_2D, st.tex_history);
        (f.bind_vertex_array)(st.vao);
        (f.draw_arrays)(GL_TRIANGLES, 0, FULLSCREEN_TRI_VERTS);
        (f.bind_framebuffer)(GL_READ_FRAMEBUFFER, 0);
        (f.bind_framebuffer)(GL_DRAW_FRAMEBUFFER, st.fbo_history);
        (f.blit_framebuffer)(0, 0, w, h, 0, 0, w, h, GL_COLOR_BUFFER_BIT, GL_LINEAR as u32);
    }
}
