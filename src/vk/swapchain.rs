use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

use ash::vk;
use ash::vk::Handle;

use crate::config::ensure_settings;
use crate::config::Settings;
use crate::consts::PUSH_BYTES;
use crate::consts::FENCE_TIMEOUT_NS;
use crate::consts::COMPUTE_X_DEFAULT;
use crate::consts::COMPUTE_Y_DEFAULT;
use crate::consts::REGISTRY;
use crate::effect::any_effect_enabled;
use crate::effect::temporal_enabled;
use crate::logging::log_at;
use crate::logging::LogLevel;
use crate::shader::current_comp_spv;
use crate::shader::current_frag_spv;
use crate::shader::current_wg;
use crate::shader::GENERATION;

use super::device::query_format_storage_supported;
use super::device::VkDevState;
use super::instance::insts_get;
use super::memory::create_compute_output_image;
use super::memory::create_offscreen_image;
use super::pipeline::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PostFxMode {
    Fragment,
    Compute,
}

pub(crate) struct VkSubmitResources {
    pub(crate) family: u32,
    pub(crate) cmd_pool: vk::CommandPool,
    pub(crate) cmd_bufs: Vec<vk::CommandBuffer>,
    pub(crate) semaphores: Vec<vk::Semaphore>,
    pub(crate) fences: Vec<vk::Fence>,
}

#[allow(dead_code)]
pub(crate) struct PostFxResources {
    pub(crate) mode: PostFxMode,
    pub(crate) use_push_desc: bool,
    pub(crate) submit_family: u32,
    pub(crate) needs_history: bool,
    pub(crate) render_pass: vk::RenderPass,
    pub(crate) pipeline_layout: vk::PipelineLayout,
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) desc_layout: vk::DescriptorSetLayout,
    pub(crate) desc_pool: vk::DescriptorPool,
    pub(crate) desc_sets: Vec<vk::DescriptorSet>,
    pub(crate) sampler: vk::Sampler,
    pub(crate) swap_views: Vec<vk::ImageView>,
    pub(crate) framebuffer: vk::Framebuffer,
    pub(crate) tex_output: vk::Image,
    pub(crate) tex_output_mem: vk::DeviceMemory,
    pub(crate) tex_output_view: vk::ImageView,
    pub(crate) tex_history: vk::Image,
    pub(crate) tex_history_mem: vk::DeviceMemory,
    pub(crate) tex_history_view: vk::ImageView,
    pub(crate) tex_upscaled: vk::Image,
    pub(crate) tex_upscaled_mem: vk::DeviceMemory,
    pub(crate) tex_upscaled_view: vk::ImageView,
    pub(crate) history_init: bool,
    pub(crate) submit_res: Option<VkSubmitResources>,
    pub(crate) gen: i32,
    pub(crate) sample_format: vk::Format,
    pub(crate) compute_x: u32,
    pub(crate) compute_y: u32,
    pub(crate) can_blit: bool,
    pub(crate) filter: vk::Filter,
}

#[allow(dead_code)]
pub(crate) struct VkSwapState {
    pub(crate) device: vk::Device,
    pub(crate) sc: vk::SwapchainKHR,
    pub(crate) images: Vec<vk::Image>,
    pub(crate) image_format: vk::Format,
    pub(crate) view_format: vk::Format,
    pub(crate) extent: vk::Extent2D,
    pub(crate) fx_extent: vk::Extent2D,
    pub(crate) res_scale: f32,
    pub(crate) mutable_format: bool,
    pub(crate) fx: Option<PostFxResources>,
}

static SWAP_FX: Mutex<Option<HashMap<u64, VkSwapState>>> = Mutex::new(None);

pub(crate) fn swap_has(sc: u64) -> bool {
    SWAP_FX
        .lock()
        .ok()
        .map(|g| g.as_ref().map(|m| m.contains_key(&sc)).unwrap_or(false))
        .unwrap_or(false)
}

pub(crate) fn swap_put(sc: u64, st: VkSwapState) {
    match SWAP_FX.lock() {
        Ok(mut g) => {
            g.get_or_insert_with(HashMap::new).insert(sc, st);
        }
        Err(_) => (),
    }
}

pub(crate) fn swap_del(sc: u64) -> Option<VkSwapState> {
    SWAP_FX
        .lock()
        .ok()
        .and_then(|mut g| g.as_mut().and_then(|m| m.remove(&sc)))
}

pub(crate) fn swap_del_for_device(dev: vk::Device) -> Vec<VkSwapState> {
    SWAP_FX
        .lock()
        .ok()
        .map(|mut g| {
            g.as_mut()
                .map(|m| {
                    let keys: Vec<u64> = m
                        .iter()
                        .filter(|(_, s)| s.device == dev)
                        .map(|(k, _)| *k)
                        .collect();
                    keys.iter().filter_map(|k| m.remove(k)).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

pub(crate) fn swap_fx_lock_mut<F, R>(sc: u64, f: F) -> Option<R>
where
    F: FnOnce(&mut VkSwapState) -> R,
{
    SWAP_FX
        .lock()
        .ok()
        .and_then(|mut g| g.as_mut().and_then(|m| m.get_mut(&sc).map(f)))
}

fn fmt_unorm(format: vk::Format) -> vk::Format {
    match format {
        vk::Format::B8G8R8A8_SRGB => vk::Format::B8G8R8A8_UNORM,
        vk::Format::R8G8B8A8_SRGB => vk::Format::R8G8B8A8_UNORM,
        vk::Format::A8B8G8R8_SRGB_PACK32 => vk::Format::A8B8G8R8_UNORM_PACK32,
        f => f,
    }
}

fn scale_extent(extent: vk::Extent2D, scale: f32) -> vk::Extent2D {
    let w = ((extent.width as f32 * scale).round() as u32).max(1);
    let h = ((extent.height as f32 * scale).round() as u32).max(1);
    vk::Extent2D { width: w, height: h }
}

fn call_get_swapchain_images(
    dev: &VkDevState,
    sc: vk::SwapchainKHR,
) -> Result<Vec<vk::Image>, ()> {
    let mut n: u32 = 0;
    let r1 = unsafe {
        (dev.swap_fp.get_swapchain_images_khr)(dev.device.handle(), sc, &mut n, ptr::null_mut())
    };
    let mut v = vec![vk::Image::null(); n as usize];
    let r2 = unsafe {
        (dev.swap_fp.get_swapchain_images_khr)(dev.device.handle(), sc, &mut n, v.as_mut_ptr())
    };
    match (r1, r2) {
        (vk::Result::SUCCESS, vk::Result::SUCCESS) => Ok(v),
        (_, _) => Err(()),
    }
}

fn create_swap_view(
    dev: &VkDevState,
    image: vk::Image,
    format: vk::Format,
) -> Result<vk::ImageView, ()> {
    let vci = vk::ImageViewCreateInfo {
        image,
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            level_count: 1,
            layer_count: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    unsafe { dev.device.create_image_view(&vci, None) }.map_err(|_| ())
}

fn create_all_swap_views(
    dev: &VkDevState,
    images: &[vk::Image],
    format: vk::Format,
) -> Result<Vec<vk::ImageView>, ()> {
    images
        .iter()
        .map(|img| create_swap_view(dev, *img, format))
        .collect()
}

fn write_fragment_desc(
    dev: &VkDevState,
    ds: vk::DescriptorSet,
    sampler: vk::Sampler,
    input: vk::ImageView,
    history: vk::ImageView,
) {
    let ii = vk::DescriptorImageInfo {
        sampler,
        image_view: input,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    };
    let hi = vk::DescriptorImageInfo {
        sampler,
        image_view: history,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    };
    let writes = [
        vk::WriteDescriptorSet {
            dst_set: ds, dst_binding: 0, descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &ii, ..Default::default()
        },
        vk::WriteDescriptorSet {
            dst_set: ds, dst_binding: 1, descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &hi, ..Default::default()
        },
    ];
    unsafe { dev.device.update_descriptor_sets(&writes, &[]) };
}

fn write_compute_desc(
    dev: &VkDevState,
    ds: vk::DescriptorSet,
    sampler: vk::Sampler,
    input: vk::ImageView,
    history: vk::ImageView,
    output: vk::ImageView,
) {
    let ii = vk::DescriptorImageInfo {
        sampler,
        image_view: input,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    };
    let hi = vk::DescriptorImageInfo {
        sampler,
        image_view: history,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    };
    let oi = vk::DescriptorImageInfo {
        sampler: vk::Sampler::null(),
        image_view: output,
        image_layout: vk::ImageLayout::GENERAL,
    };
    let writes = [
        vk::WriteDescriptorSet {
            dst_set: ds, dst_binding: 0, descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &ii, ..Default::default()
        },
        vk::WriteDescriptorSet {
            dst_set: ds, dst_binding: 1, descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &hi, ..Default::default()
        },
        vk::WriteDescriptorSet {
            dst_set: ds, dst_binding: 2, descriptor_count: 1,
            descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
            p_image_info: &oi, ..Default::default()
        },
    ];
    unsafe { dev.device.update_descriptor_sets(&writes, &[]) };
}

fn create_framebuffer(
    dev: &VkDevState,
    render_pass: vk::RenderPass,
    view: vk::ImageView,
    extent: vk::Extent2D,
) -> Result<vk::Framebuffer, ()> {
    let fb_ci = vk::FramebufferCreateInfo {
        render_pass,
        attachment_count: 1,
        p_attachments: &view,
        width: extent.width,
        height: extent.height,
        layers: 1,
        ..Default::default()
    };
    unsafe { dev.device.create_framebuffer(&fb_ci, None) }.map_err(|_| ())
}

fn create_pipeline_layout(
    dev: &VkDevState,
    desc_layout: vk::DescriptorSetLayout,
    stage_flags: vk::ShaderStageFlags,
) -> Result<vk::PipelineLayout, ()> {
    let push = vk::PushConstantRange {
        stage_flags,
        offset: 0,
        size: PUSH_BYTES,
    };
    let plci = vk::PipelineLayoutCreateInfo {
        set_layout_count: 1,
        p_set_layouts: &desc_layout,
        push_constant_range_count: 1,
        p_push_constant_ranges: &push,
        ..Default::default()
    };
    unsafe { dev.device.create_pipeline_layout(&plci, None) }.map_err(|_| ())
}

fn create_sampler(dev: &VkDevState) -> Result<vk::Sampler, ()> {
    let sci = vk::SamplerCreateInfo {
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        address_mode_u: vk::SamplerAddressMode::CLAMP_TO_EDGE,
        address_mode_v: vk::SamplerAddressMode::CLAMP_TO_EDGE,
        address_mode_w: vk::SamplerAddressMode::CLAMP_TO_EDGE,
        ..Default::default()
    };
    unsafe { dev.device.create_sampler(&sci, None) }.map_err(|_| ())
}

fn create_desc_pool(dev: &VkDevState, count: u32, sampler_count: u32, storage_count: u32) -> Result<vk::DescriptorPool, ()> {
    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: sampler_count * count,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::STORAGE_IMAGE,
            descriptor_count: storage_count * count,
        },
    ];
    let used = match storage_count {
        0 => 1,
        _ => 2,
    };
    let dpci = vk::DescriptorPoolCreateInfo {
        max_sets: count,
        pool_size_count: used,
        p_pool_sizes: pool_sizes.as_ptr(),
        ..Default::default()
    };
    unsafe { dev.device.create_descriptor_pool(&dpci, None) }.map_err(|_| ())
}

fn allocate_desc_sets(
    dev: &VkDevState,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    count: usize,
) -> Result<Vec<vk::DescriptorSet>, ()> {
    let layouts = vec![layout; count];
    let dsai = vk::DescriptorSetAllocateInfo {
        descriptor_pool: pool,
        descriptor_set_count: count as u32,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };
    unsafe { dev.device.allocate_descriptor_sets(&dsai) }.map_err(|_| ())
}

struct FxBuilder<'a> {
    dev: &'a VkDevState,
    mode: PostFxMode,
    framebuffer: vk::Framebuffer,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    desc_layout: vk::DescriptorSetLayout,
    desc_pool: vk::DescriptorPool,
    sampler: vk::Sampler,
    swap_views: Vec<vk::ImageView>,
    tex_output: vk::Image,
    tex_output_mem: vk::DeviceMemory,
    tex_output_view: vk::ImageView,
    tex_history: vk::Image,
    tex_history_mem: vk::DeviceMemory,
    tex_history_view: vk::ImageView,
    tex_upscaled: vk::Image,
    tex_upscaled_mem: vk::DeviceMemory,
    tex_upscaled_view: vk::ImageView,
    desc_sets: Vec<vk::DescriptorSet>,
    armed: bool,
}

fn destroy_view_list(dev: &VkDevState, views: &[vk::ImageView]) {
    views
        .iter()
        .filter(|v| **v != vk::ImageView::null())
        .for_each(|v| unsafe { dev.device.destroy_image_view(*v, None) });
}

fn destroy_builder_tex(
    dev: &VkDevState,
    img: vk::Image,
    mem: vk::DeviceMemory,
    view: vk::ImageView,
) {
    match img == vk::Image::null() {
        true => (),
        false => unsafe {
            dev.device.destroy_image_view(view, None);
            dev.device.destroy_image(img, None);
            dev.device.free_memory(mem, None);
        },
    }
}

fn destroy_builder_handle<T, F>(dev: &VkDevState, handle: T, null: bool, destroy: F)
where
    F: FnOnce(&ash::Device, T),
{
    match null {
        true => (),
        false => destroy(&dev.device, handle),
    }
}

impl<'a> Drop for FxBuilder<'a> {
    fn drop(&mut self) {
        match self.armed {
            false => (),
            true => {
                let dev = self.dev;
                destroy_builder_handle(dev, self.framebuffer, self.framebuffer == vk::Framebuffer::null(),
                    |d, h| unsafe { d.destroy_framebuffer(h, None) });
                destroy_builder_handle(dev, self.render_pass, self.render_pass == vk::RenderPass::null(),
                    |d, h| unsafe { d.destroy_render_pass(h, None) });
                destroy_builder_handle(dev, self.pipeline_layout, self.pipeline_layout == vk::PipelineLayout::null(),
                    |d, h| unsafe { d.destroy_pipeline_layout(h, None) });
                destroy_builder_handle(dev, self.pipeline, self.pipeline == vk::Pipeline::null(),
                    |d, h| unsafe { d.destroy_pipeline(h, None) });
                destroy_builder_handle(dev, self.desc_layout, self.desc_layout == vk::DescriptorSetLayout::null(),
                    |d, h| unsafe { d.destroy_descriptor_set_layout(h, None) });
                destroy_builder_handle(dev, self.desc_pool, self.desc_pool == vk::DescriptorPool::null(),
                    |d, h| unsafe { d.destroy_descriptor_pool(h, None) });
                destroy_builder_handle(dev, self.sampler, self.sampler == vk::Sampler::null(),
                    |d, h| unsafe { d.destroy_sampler(h, None) });
                destroy_view_list(dev, &self.swap_views);
                destroy_builder_tex(dev, self.tex_output, self.tex_output_mem, self.tex_output_view);
                destroy_builder_tex(dev, self.tex_history, self.tex_history_mem, self.tex_history_view);
                destroy_builder_tex(dev, self.tex_upscaled, self.tex_upscaled_mem, self.tex_upscaled_view);
            }
        }
    }
}

fn new_fx_builder(dev: &VkDevState, mode: PostFxMode) -> FxBuilder<'_> {
    FxBuilder {
        dev,
        mode,
        framebuffer: vk::Framebuffer::null(),
        render_pass: vk::RenderPass::null(),
        pipeline_layout: vk::PipelineLayout::null(),
        pipeline: vk::Pipeline::null(),
        desc_layout: vk::DescriptorSetLayout::null(),
        desc_pool: vk::DescriptorPool::null(),
        sampler: vk::Sampler::null(),
        swap_views: Vec::new(),
        tex_output: vk::Image::null(),
        tex_output_mem: vk::DeviceMemory::null(),
        tex_output_view: vk::ImageView::null(),
        tex_history: vk::Image::null(),
        tex_history_mem: vk::DeviceMemory::null(),
        tex_history_view: vk::ImageView::null(),
        tex_upscaled: vk::Image::null(),
        tex_upscaled_mem: vk::DeviceMemory::null(),
        tex_upscaled_view: vk::ImageView::null(),
        desc_sets: Vec::new(),
        armed: true,
    }
}

fn finalize_fx(
    mut b: FxBuilder,
    sample_format: vk::Format,
    wg_x: u32,
    wg_y: u32,
    needs_history: bool,
    use_push_desc: bool,
    submit_family: u32,
) -> PostFxResources {
    let (can_blit, filter) = match insts_get(b.dev.instance_handle) {
        Some(inst) => {
            let props = unsafe { inst.instance.get_physical_device_format_properties(b.dev.phys, sample_format) };
            let fmt_blit = props.optimal_tiling_features.contains(vk::FormatFeatureFlags::BLIT_DST)
                && props.optimal_tiling_features.contains(vk::FormatFeatureFlags::BLIT_SRC);
            let f = match props.optimal_tiling_features.contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR) {
                true => vk::Filter::LINEAR,
                false => vk::Filter::NEAREST,
            };
            let q_props = unsafe { inst.instance.get_physical_device_queue_family_properties(b.dev.phys) };
            let q_blit = q_props.get(submit_family as usize)
                .map(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .unwrap_or(false);
            (fmt_blit && q_blit, f)
        }
        None => (false, vk::Filter::NEAREST),
    };

    let r = PostFxResources {
        mode: b.mode,
        use_push_desc,
        submit_family,
        needs_history,
        render_pass: b.render_pass,
        pipeline_layout: b.pipeline_layout,
        pipeline: b.pipeline,
        desc_layout: b.desc_layout,
        desc_pool: b.desc_pool,
        desc_sets: std::mem::take(&mut b.desc_sets),
        sampler: b.sampler,
        swap_views: std::mem::take(&mut b.swap_views),
        framebuffer: b.framebuffer,
        tex_output: b.tex_output,
        tex_output_mem: b.tex_output_mem,
        tex_output_view: b.tex_output_view,
        tex_history: b.tex_history,
        tex_history_mem: b.tex_history_mem,
        tex_history_view: b.tex_history_view,
        tex_upscaled: b.tex_upscaled,
        tex_upscaled_mem: b.tex_upscaled_mem,
        tex_upscaled_view: b.tex_upscaled_view,
        history_init: false,
        submit_res: None,
        gen: GENERATION.load(Ordering::Relaxed),
        sample_format,
        compute_x: wg_x,
        compute_y: wg_y,
        can_blit,
        filter,
    };
    b.armed = false;
    r
}

fn spv_or_fail(spv: Vec<u32>, label: &str) -> Result<Vec<u32>, ()> {
    match spv.is_empty() {
        true => {
            log_at(
                LogLevel::Error,
                &format!("no spirv available for {}, postfx skipped until shaders ready", label),
            );
            Err(())
        }
        false => Ok(spv),
    }
}

fn pick_submit_family(dev: &VkDevState, mode: PostFxMode, present_family: u32, extent: vk::Extent2D, fx_extent: vk::Extent2D) -> u32 {
    let is_native = extent.width == fx_extent.width && extent.height == fx_extent.height;
    match (mode, dev.caps.async_compute_family, is_native) {
        (PostFxMode::Compute, Some(f), true) => f,
        (_, _, _) => present_family,
    }
}

fn build_fx_compute(
    dev: &VkDevState,
    st: &VkSwapState,
    needs_history: bool,
    use_push_desc: bool,
    submit_family: u32,
) -> Result<PostFxResources, ()> {
    let spv = spv_or_fail(current_comp_spv(), "compute")?;
    let (wg_x, wg_y) = current_wg();
    let fx_extent = st.fx_extent;
    let format = st.view_format;
    let mut b = new_fx_builder(dev, PostFxMode::Compute);

    let (to_img, to_mem, to_view) = create_compute_output_image(dev, fx_extent, format)?;
    b.tex_output = to_img;
    b.tex_output_mem = to_mem;
    b.tex_output_view = to_view;

    b.desc_layout = create_desc_layout_compute(dev, use_push_desc)?;
    b.pipeline_layout = create_pipeline_layout(dev, b.desc_layout, vk::ShaderStageFlags::COMPUTE)?;
    b.pipeline = create_compute_pipeline(dev, b.pipeline_layout, &spv)?;
    b.sampler = create_sampler(dev)?;

    let (th_img, th_mem, th_view) = create_offscreen_image(dev, fx_extent, format)?;
    b.tex_history = th_img;
    b.tex_history_mem = th_mem;
    b.tex_history_view = th_view;

    let (up_img, up_mem, up_view) = match st.extent.width == fx_extent.width && st.extent.height == fx_extent.height {
        true => (vk::Image::null(), vk::DeviceMemory::null(), vk::ImageView::null()),
        false => create_offscreen_image(dev, st.extent, format)?,
    };
    b.tex_upscaled = up_img;
    b.tex_upscaled_mem = up_mem;
    b.tex_upscaled_view = up_view;

    b.swap_views = create_all_swap_views(dev, &st.images, st.view_format)?;
    match use_push_desc {
        false => {
            b.desc_pool = create_desc_pool(dev, st.images.len() as u32, 2, 1)?;
            b.desc_sets = allocate_desc_sets(dev, b.desc_pool, b.desc_layout, st.images.len())?;
            b.desc_sets
                .iter()
                .zip(b.swap_views.iter())
                .for_each(|(ds, sv)| write_compute_desc(dev, *ds, b.sampler, *sv, th_view, to_view));
        }
        true => (),
    }

    Ok(finalize_fx(b, st.view_format, wg_x, wg_y, needs_history, use_push_desc, submit_family))
}

fn build_fx_fragment(
    dev: &VkDevState,
    st: &VkSwapState,
    needs_history: bool,
    use_push_desc: bool,
    submit_family: u32,
) -> Result<PostFxResources, ()> {
    let spv = spv_or_fail(current_frag_spv(), "fragment")?;
    let fx_extent = st.fx_extent;
    let format = st.view_format;
    let mut b = new_fx_builder(dev, PostFxMode::Fragment);

    let (to_img, to_mem, to_view) = create_offscreen_image(dev, fx_extent, format)?;
    b.tex_output = to_img;
    b.tex_output_mem = to_mem;
    b.tex_output_view = to_view;

    b.render_pass = create_render_pass(dev, format)?;
    b.framebuffer = create_framebuffer(dev, b.render_pass, to_view, fx_extent)?;
    b.desc_layout = create_desc_layout_fragment(dev, use_push_desc)?;
    b.pipeline_layout = create_pipeline_layout(dev, b.desc_layout, vk::ShaderStageFlags::FRAGMENT)?;
    b.pipeline = create_pipeline(dev, b.pipeline_layout, b.render_pass, fx_extent, &spv)?;
    b.sampler = create_sampler(dev)?;

    let (th_img, th_mem, th_view) = create_offscreen_image(dev, fx_extent, format)?;
    b.tex_history = th_img;
    b.tex_history_mem = th_mem;
    b.tex_history_view = th_view;

    let (up_img, up_mem, up_view) = match st.extent.width == fx_extent.width && st.extent.height == fx_extent.height {
        true => (vk::Image::null(), vk::DeviceMemory::null(), vk::ImageView::null()),
        false => create_offscreen_image(dev, st.extent, format)?,
    };
    b.tex_upscaled = up_img;
    b.tex_upscaled_mem = up_mem;
    b.tex_upscaled_view = up_view;

    b.swap_views = create_all_swap_views(dev, &st.images, st.view_format)?;
    match use_push_desc {
        false => {
            b.desc_pool = create_desc_pool(dev, st.images.len() as u32, 2, 0)?;
            b.desc_sets = allocate_desc_sets(dev, b.desc_pool, b.desc_layout, st.images.len())?;
            b.desc_sets
                .iter()
                .zip(b.swap_views.iter())
                .for_each(|(ds, sv)| write_fragment_desc(dev, *ds, b.sampler, *sv, th_view));
        }
        true => (),
    }

    Ok(finalize_fx(b, st.view_format, COMPUTE_X_DEFAULT, COMPUTE_Y_DEFAULT, needs_history, use_push_desc, submit_family))
}

fn compute_format_usable(dev: &VkDevState, format: vk::Format) -> bool {
    match insts_get(dev.instance_handle) {
        Some(inst) => query_format_storage_supported(&inst, dev.phys, format),
        None => false,
    }
}

fn choose_mode(dev: &VkDevState, s: &Settings, format: vk::Format) -> PostFxMode {
    let want_compute = s.compute;
    let cap_ok = dev.caps.storage_image_write_without_fmt;
    let fmt_ok = compute_format_usable(dev, format);
    match (want_compute, cap_ok, fmt_ok) {
        (true, true, true) => PostFxMode::Compute,
        (true, true, false) => {
            log_at(LogLevel::Warn, "compute path not applied: format lacks STORAGE_IMAGE feature, falling back to fragment");
            PostFxMode::Fragment
        }
        (true, false, _) => PostFxMode::Fragment,
        (false, _, _) => PostFxMode::Fragment,
    }
}

fn build_fx_resources(dev: &VkDevState, st: &VkSwapState, present_family: u32) -> Result<PostFxResources, ()> {
    let s = ensure_settings();
    let mode = choose_mode(dev, &s, st.view_format);
    let needs_history = temporal_enabled(&s, &REGISTRY);
    let use_push_desc = dev.caps.pushdesc && dev.push_desc_fp.is_some();
    let submit_family = pick_submit_family(dev, mode, present_family, st.extent, st.fx_extent);
    match mode {
        PostFxMode::Compute => build_fx_compute(dev, st, needs_history, use_push_desc, submit_family),
        PostFxMode::Fragment => build_fx_fragment(dev, st, needs_history, use_push_desc, submit_family),
    }
}

pub(crate) fn ensure_fx(dev: &VkDevState, st: &mut VkSwapState, present_family: u32) -> bool {
    match st.fx.is_some() {
        true => true,
        false => match build_fx_resources(dev, st, present_family) {
            Ok(fx) => {
                let mode_label = match fx.mode {
                    PostFxMode::Compute => "compute",
                    PostFxMode::Fragment => "fragment",
                };
                let queue_label = match fx.submit_family == present_family {
                    true => "present queue",
                    false => "async compute queue",
                };
                log_at(
                    LogLevel::Info,
                    &format!("postfx resources lazily built ({}, {})", mode_label, queue_label),
                );
                st.fx = Some(fx);
                true
            }
            Err(()) => {
                log_at(LogLevel::Warn, "postfx build deferred until shaders ready");
                false
            }
        },
    }
}

fn pipeline_gen_stale(st_gen: i32, cur_gen: i32) -> bool {
    st_gen != cur_gen
}

fn call_wait_all_fences(dev: &VkDevState, fences: &[vk::Fence]) {
    unsafe {
        let _ = dev.device.wait_for_fences(fences, true, u64::MAX);
    }
}

fn try_rebuild_pipeline(
    dev: &VkDevState,
    fx: &PostFxResources,
    fx_extent: vk::Extent2D,
) -> Result<vk::Pipeline, ()> {
    match fx.submit_res.as_ref() {
        Some(r) => call_wait_all_fences(dev, &r.fences),
        None => (),
    }
    match fx.mode {
        PostFxMode::Compute => create_compute_pipeline(dev, fx.pipeline_layout, &current_comp_spv()),
        PostFxMode::Fragment => create_pipeline(dev, fx.pipeline_layout, fx.render_pass, fx_extent, &current_frag_spv()),
    }
}

fn apply_pipeline_rebuild(
    dev: &VkDevState,
    fx: &mut PostFxResources,
    result: Result<vk::Pipeline, ()>,
    gen: i32,
) {
    match result {
        Ok(p) => {
            unsafe { dev.device.destroy_pipeline(fx.pipeline, None); }
            fx.pipeline = p;
            let (x, y) = current_wg();
            fx.compute_x = x;
            fx.compute_y = y;
            fx.gen = gen;
            log_at(LogLevel::Info, "vk pipeline rebuilt for hot reload");
        }
        Err(()) => log_at(
            LogLevel::Error,
            "vk pipeline rebuild failed, will retry next present",
        ),
    }
}

fn spv_ready_for_mode(mode: PostFxMode) -> bool {
    match mode {
        PostFxMode::Compute => !current_comp_spv().is_empty(),
        PostFxMode::Fragment => !current_frag_spv().is_empty(),
    }
}

pub(crate) fn check_rebuild_pipeline(dev: &VkDevState, st: &mut VkSwapState) {
    let fx_extent = st.fx_extent;
    let gen = GENERATION.load(Ordering::Relaxed);
    match st.fx.as_mut() {
        Some(fx) => match pipeline_gen_stale(fx.gen, gen) && spv_ready_for_mode(fx.mode) {
            true => apply_pipeline_rebuild(dev, fx, try_rebuild_pipeline(dev, fx, fx_extent), gen),
            false => (),
        },
        None => (),
    }
}

fn create_submit_resources(
    dev: &VkDevState,
    family: u32,
    count: usize,
) -> Result<VkSubmitResources, ()> {
    let cpci = vk::CommandPoolCreateInfo {
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queue_family_index: family,
        ..Default::default()
    };
    let cmd_pool = unsafe { dev.device.create_command_pool(&cpci, None) }.map_err(|_| ())?;
    let cbai = vk::CommandBufferAllocateInfo {
        command_pool: cmd_pool,
        level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: count as u32,
        ..Default::default()
    };
    let cmd_bufs = unsafe { dev.device.allocate_command_buffers(&cbai) }.map_err(|_| ())?;
    let semaphores = (0..count)
        .map(|_| unsafe { dev.device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) })
        .collect::<Result<Vec<_>, _>>().map_err(|_| ())?;
    let fences = (0..count)
        .map(|_| unsafe {
            dev.device.create_fence(&vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            }, None)
        })
        .collect::<Result<Vec<_>, _>>().map_err(|_| ())?;
    Ok(VkSubmitResources { family, cmd_pool, cmd_bufs, semaphores, fences })
}

fn submit_res_needs_replacement(cur: &Option<VkSubmitResources>, family: u32) -> bool {
    match cur {
        Some(r) => r.family != family,
        None => true,
    }
}

pub(crate) fn ensure_submit_resources(
    dev: &VkDevState,
    fx: &mut PostFxResources,
    family: u32,
    image_count: usize,
) -> bool {
    match submit_res_needs_replacement(&fx.submit_res, family) {
        true => {
            fx.submit_res.take().into_iter().for_each(|r| destroy_submit_resources(dev, r));
            match create_submit_resources(dev, family, image_count) {
                Ok(r) => { fx.submit_res = Some(r); true }
                Err(()) => {
                    log_at(LogLevel::Error, "submit resource alloc failed for queue family");
                    false
                }
            }
        }
        false => true,
    }
}

fn destroy_submit_resources(dev: &VkDevState, r: VkSubmitResources) {
    unsafe {
        let _ = dev.device.wait_for_fences(&r.fences, true, FENCE_TIMEOUT_NS);
        r.fences.iter().for_each(|f| dev.device.destroy_fence(*f, None));
        r.semaphores.iter().for_each(|s| dev.device.destroy_semaphore(*s, None));
        dev.device.destroy_command_pool(r.cmd_pool, None);
    }
}

fn destroy_fx_resources(dev: &VkDevState, fx: &mut PostFxResources) {
    fx.submit_res
        .take()
        .into_iter()
        .for_each(|r| destroy_submit_resources(dev, r));
    let has_rp = fx.render_pass != vk::RenderPass::null();
    let has_fb = fx.framebuffer != vk::Framebuffer::null();
    unsafe {
        dev.device.destroy_descriptor_pool(fx.desc_pool, None);
        dev.device.destroy_sampler(fx.sampler, None);
        fx.swap_views
            .iter()
            .for_each(|v| dev.device.destroy_image_view(*v, None));
        dev.device.destroy_image_view(fx.tex_output_view, None);
        dev.device.destroy_image(fx.tex_output, None);
        dev.device.free_memory(fx.tex_output_mem, None);
        dev.device.destroy_image_view(fx.tex_history_view, None);
        dev.device.destroy_image(fx.tex_history, None);
        dev.device.free_memory(fx.tex_history_mem, None);
        dev.device.destroy_image_view(fx.tex_upscaled_view, None);
        dev.device.destroy_image(fx.tex_upscaled, None);
        dev.device.free_memory(fx.tex_upscaled_mem, None);
        dev.device.destroy_pipeline(fx.pipeline, None);
        dev.device.destroy_pipeline_layout(fx.pipeline_layout, None);
        dev.device.destroy_descriptor_set_layout(fx.desc_layout, None);
        match has_fb {
            true => dev.device.destroy_framebuffer(fx.framebuffer, None),
            false => (),
        }
        match has_rp {
            true => dev.device.destroy_render_pass(fx.render_pass, None),
            false => (),
        }
    }
}

pub(crate) fn destroy_swap_state(dev: &VkDevState, st: &mut VkSwapState) {
    st.fx.as_mut().into_iter().for_each(|fx| destroy_fx_resources(dev, fx));
    st.fx = None;
}

fn try_create_swapchain_mutable(
    d: &VkDevState,
    dev_h: vk::Device,
    ci: &vk::SwapchainCreateInfoKHR,
    formats: &[vk::Format; 2],
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::SwapchainKHR,
) -> vk::Result {
    let list = vk::ImageFormatListCreateInfo {
        view_format_count: formats.len() as u32,
        p_view_formats: formats.as_ptr(),
        p_next: ci.p_next,
        ..Default::default()
    };
    let upgraded = vk::SwapchainCreateInfoKHR {
        flags: ci.flags | vk::SwapchainCreateFlagsKHR::MUTABLE_FORMAT,
        image_usage: ci.image_usage
            | vk::ImageUsageFlags::COLOR_ATTACHMENT
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::SAMPLED,
        p_next: &list as *const _ as *const c_void,
        ..*ci
    };
    unsafe { (d.swap_fp.create_swapchain_khr)(dev_h, &upgraded, alloc, out) }
}

fn try_create_swapchain_plain(
    d: &VkDevState,
    dev_h: vk::Device,
    ci: &vk::SwapchainCreateInfoKHR,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::SwapchainKHR,
) -> vk::Result {
    let upgraded = vk::SwapchainCreateInfoKHR {
        image_usage: ci.image_usage
            | vk::ImageUsageFlags::COLOR_ATTACHMENT
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::SAMPLED,
        ..*ci
    };
    unsafe { (d.swap_fp.create_swapchain_khr)(dev_h, &upgraded, alloc, out) }
}

fn try_create_swapchain_passthrough(
    d: &VkDevState,
    dev_h: vk::Device,
    ci: &vk::SwapchainCreateInfoKHR,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::SwapchainKHR,
) -> vk::Result {
    unsafe { (d.swap_fp.create_swapchain_khr)(dev_h, ci, alloc, out) }
}

struct SwapAttempt {
    result: vk::Result,
    view_format: vk::Format,
    mutable: bool,
}

fn attempt_swapchain_creation_plain(
    d: &VkDevState,
    dev_h: vk::Device,
    ci: &vk::SwapchainCreateInfoKHR,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::SwapchainKHR,
) -> SwapAttempt {
    match try_create_swapchain_plain(d, dev_h, ci, alloc, out) {
        vk::Result::SUCCESS => SwapAttempt {
            result: vk::Result::SUCCESS,
            view_format: ci.image_format,
            mutable: false,
        },
        _ => SwapAttempt {
            result: try_create_swapchain_passthrough(d, dev_h, ci, alloc, out),
            view_format: ci.image_format,
            mutable: false,
        },
    }
}

fn attempt_swapchain_creation(
    d: &VkDevState,
    dev_h: vk::Device,
    ci: &vk::SwapchainCreateInfoKHR,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::SwapchainKHR,
) -> SwapAttempt {
    let unorm = fmt_unorm(ci.image_format);
    let try_mutable = d.caps.mutable_fmt && unorm != ci.image_format;
    let formats = [ci.image_format, unorm];
    match try_mutable {
        true => match try_create_swapchain_mutable(d, dev_h, ci, &formats, alloc, out) {
            vk::Result::SUCCESS => SwapAttempt {
                result: vk::Result::SUCCESS,
                view_format: unorm,
                mutable: true,
            },
            _ => attempt_swapchain_creation_plain(d, dev_h, ci, alloc, out),
        },
        false => attempt_swapchain_creation_plain(d, dev_h, ci, alloc, out),
    }
}

fn register_swap_state(
    d: &VkDevState,
    dev_h: vk::Device,
    sc: vk::SwapchainKHR,
    ci: &vk::SwapchainCreateInfoKHR,
    attempt: &SwapAttempt,
    res_scale: f32,
) {
    match call_get_swapchain_images(d, sc) {
        Ok(images) => {
            let extent = ci.image_extent;
            let fx_extent = scale_extent(extent, res_scale);
            swap_put(sc.as_raw(), VkSwapState {
                device: dev_h,
                sc,
                images,
                image_format: ci.image_format,
                view_format: attempt.view_format,
                extent,
                fx_extent,
                res_scale,
                mutable_format: attempt.mutable,
                fx: None,
            });
            log_at(LogLevel::Info, "swapchain registered (postfx lazily built)");
        }
        Err(()) => log_at(LogLevel::Error, "swapchain image enumeration failed, postfx disabled"),
    }
}

fn concurrent_families(dev: &VkDevState, ci_families: &[u32]) -> Vec<u32> {
    let mut out: Vec<u32> = dev.app_queue_families.iter().copied().collect();
    ci_families.iter().for_each(|f| match out.contains(f) {
        true => (),
        false => out.push(*f),
    });
    match dev.caps.async_compute_family {
        Some(f) => match out.contains(&f) {
            true => (),
            false => out.push(f),
        },
        None => (),
    }
    out
}

fn read_ci_families(ci: &vk::SwapchainCreateInfoKHR) -> Vec<u32> {
    match ci.queue_family_index_count {
        0 => Vec::new(),
        n => (0..n as usize)
            .map(|i| unsafe { *ci.p_queue_family_indices.add(i) })
            .collect(),
    }
}

fn upgrade_swapchain_sharing(
    dev: &VkDevState,
    ci: &vk::SwapchainCreateInfoKHR,
    families: &[u32],
) -> Option<vk::SwapchainCreateInfoKHR> {
    match dev.caps.async_compute_family.is_some() && families.len() >= 2 {
        true => Some(vk::SwapchainCreateInfoKHR {
            image_sharing_mode: vk::SharingMode::CONCURRENT,
            queue_family_index_count: families.len() as u32,
            p_queue_family_indices: families.as_ptr(),
            ..*ci
        }),
        false => None,
    }
}

pub(crate) fn create_swapchain_with_fx(
    d: &VkDevState,
    dev: vk::Device,
    ci: *const vk::SwapchainCreateInfoKHR,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::SwapchainKHR,
) -> vk::Result {
    let ci_ref = unsafe { &*ci };
    let s = ensure_settings();
    let ci_families = read_ci_families(ci_ref);
    let concurrent = concurrent_families(d, &ci_families);
    let upgraded = upgrade_swapchain_sharing(d, ci_ref, &concurrent);
    let effective_ci = upgraded.as_ref().unwrap_or(ci_ref);
    let attempt = attempt_swapchain_creation(d, dev, effective_ci, alloc, out);
    match attempt.result {
        vk::Result::SUCCESS => {
            register_swap_state(d, dev, unsafe { *out }, effective_ci, &attempt, s.res_scale);
            vk::Result::SUCCESS
        }
        e => e,
    }
}

pub(crate) fn settings_has_any_fx(s: &Settings) -> bool {
    any_effect_enabled(s, &REGISTRY)
}
