use std::collections::HashMap;
use std::ptr;
use std::sync::Mutex;

use ash::vk;
use ash::vk::Handle;

use crate::consts::PUSH_BYTES;
use crate::logging::{LogLevel, log_at};
use crate::shader::current_spv;

use super::device::VkDevState;
use super::memory::create_offscreen_image;
use super::pipeline::*;

pub(crate) struct VkSwapState {
    pub(crate) device: vk::Device,
    pub(crate) images: Vec<vk::Image>,
    pub(crate) framebuffers: Vec<vk::Framebuffer>,
    pub(crate) extent: vk::Extent2D,
    pub(crate) render_pass: vk::RenderPass,
    pub(crate) pipeline_layout: vk::PipelineLayout,
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) desc_layout: vk::DescriptorSetLayout,
    pub(crate) desc_pool: vk::DescriptorPool,
    pub(crate) desc_sets: Vec<vk::DescriptorSet>,
    pub(crate) sampler: vk::Sampler,
    pub(crate) tex_input: vk::Image,
    pub(crate) tex_input_mem: vk::DeviceMemory,
    pub(crate) tex_input_view: vk::ImageView,
    pub(crate) tex_output: vk::Image,
    pub(crate) tex_output_mem: vk::DeviceMemory,
    pub(crate) tex_output_view: vk::ImageView,
    pub(crate) tex_history: vk::Image,
    pub(crate) tex_history_mem: vk::DeviceMemory,
    pub(crate) tex_history_view: vk::ImageView,
    pub(crate) history_init: bool,
    pub(crate) cmd_pool: vk::CommandPool,
    pub(crate) cmd_bufs: Vec<vk::CommandBuffer>,
    pub(crate) semaphores: Vec<vk::Semaphore>,
    pub(crate) fences: Vec<vk::Fence>,
}

static SWAP_FX: Mutex<Option<HashMap<u64, VkSwapState>>> = Mutex::new(None);

pub(crate) fn swap_has(sc: u64) -> bool {
    SWAP_FX.lock().ok()
        .map(|g| g.as_ref().map(|m| m.contains_key(&sc)).unwrap_or(false))
        .unwrap_or(false)
}

pub(crate) fn swap_put(sc: u64, st: VkSwapState) {
    match SWAP_FX.lock() {
        Ok(mut g) => { g.get_or_insert_with(HashMap::new).insert(sc, st); }
        Err(_) => (),
    }
}

pub(crate) fn swap_del(sc: u64) -> Option<VkSwapState> {
    SWAP_FX.lock().ok().and_then(|mut g| g.as_mut().and_then(|m| m.remove(&sc)))
}

pub(crate) fn swap_del_for_device(dev: vk::Device) -> Vec<VkSwapState> {
    SWAP_FX.lock().ok()
        .map(|mut g| {
            g.as_mut().map(|m| {
                let keys: Vec<u64> = m.iter().filter(|(_, s)| s.device == dev).map(|(k, _)| *k).collect();
                keys.iter().filter_map(|k| m.remove(k)).collect::<Vec<_>>()
            }).unwrap_or_default()
        }).unwrap_or_default()
}

pub(crate) fn swap_fx_lock_mut<F, R>(sc: u64, f: F) -> Option<R>
where
    F: FnOnce(&mut VkSwapState) -> R,
{
    SWAP_FX.lock().ok().and_then(|mut g| g.as_mut().and_then(|m| m.get_mut(&sc).map(f)))
}

fn fmt_unorm(format: vk::Format) -> vk::Format {
    match format {
        vk::Format::B8G8R8A8_SRGB => vk::Format::B8G8R8A8_UNORM,
        vk::Format::R8G8B8A8_SRGB => vk::Format::R8G8B8A8_UNORM,
        vk::Format::A8B8G8R8_SRGB_PACK32 => vk::Format::A8B8G8R8_UNORM_PACK32,
        f => f,
    }
}

fn call_get_swapchain_images(dev: &VkDevState, sc: vk::SwapchainKHR) -> Result<Vec<vk::Image>, ()> {
    let mut n: u32 = 0;
    let r1 = unsafe { (dev.swap_fp.get_swapchain_images_khr)(dev.device.handle(), sc, &mut n, ptr::null_mut()) };
    let mut v = vec![vk::Image::null(); n as usize];
    let r2 = unsafe { (dev.swap_fp.get_swapchain_images_khr)(dev.device.handle(), sc, &mut n, v.as_mut_ptr()) };
    match (r1, r2) {
        (vk::Result::SUCCESS, vk::Result::SUCCESS) => Ok(v),
        (_, _) => Err(()),
    }
}

fn write_descriptors(
    dev: &VkDevState,
    ds: vk::DescriptorSet,
    sampler: vk::Sampler,
    input: vk::ImageView,
    history: vk::ImageView,
) {
    let ii = vk::DescriptorImageInfo {
        sampler, image_view: input,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    };
    let hi = vk::DescriptorImageInfo {
        sampler, image_view: history,
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

pub(crate) fn build_swap_state(
    dev: &VkDevState,
    dev_h: vk::Device,
    sc: vk::SwapchainKHR,
    ci: &vk::SwapchainCreateInfoKHR,
) -> Result<VkSwapState, ()> {
    let spv = current_spv();
    match spv.is_empty() {
        true => {
            log_at(LogLevel::Error, "no spirv available, postfx disabled for swapchain");
            return Err(());
        }
        false => (),
    }
    let images = call_get_swapchain_images(dev, sc)?;
    let format = fmt_unorm(ci.image_format);
    let extent = ci.image_extent;
    let (tex_output, tex_output_mem, tex_output_view) = create_offscreen_image(dev, extent, format)?;
    let render_pass = create_render_pass(dev, format)?;
    let fb_ci = vk::FramebufferCreateInfo {
        render_pass, attachment_count: 1, p_attachments: &tex_output_view,
        width: extent.width, height: extent.height, layers: 1, ..Default::default()
    };
    let framebuffers = vec![unsafe { dev.device.create_framebuffer(&fb_ci, None) }.map_err(|_| ())?];
    let desc_layout = create_desc_layout(dev)?;
    let push = vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::FRAGMENT, offset: 0, size: PUSH_BYTES,
    };
    let plci = vk::PipelineLayoutCreateInfo {
        set_layout_count: 1, p_set_layouts: &desc_layout,
        push_constant_range_count: 1, p_push_constant_ranges: &push, ..Default::default()
    };
    let pipeline_layout = unsafe { dev.device.create_pipeline_layout(&plci, None) }.map_err(|_| ())?;
    let pipeline = create_pipeline(dev, pipeline_layout, render_pass, extent, &spv)?;
    let sci = vk::SamplerCreateInfo {
        mag_filter: vk::Filter::LINEAR, min_filter: vk::Filter::LINEAR,
        address_mode_u: vk::SamplerAddressMode::CLAMP_TO_EDGE,
        address_mode_v: vk::SamplerAddressMode::CLAMP_TO_EDGE,
        address_mode_w: vk::SamplerAddressMode::CLAMP_TO_EDGE, ..Default::default()
    };
    let sampler = unsafe { dev.device.create_sampler(&sci, None) }.map_err(|_| ())?;
    let (tex_input, tex_input_mem, tex_input_view) = create_offscreen_image(dev, extent, format)?;
    let (tex_history, tex_history_mem, tex_history_view) = create_offscreen_image(dev, extent, format)?;
    let pool_size = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 2 * images.len() as u32,
    };
    let dpci = vk::DescriptorPoolCreateInfo {
        max_sets: images.len() as u32, pool_size_count: 1, p_pool_sizes: &pool_size, ..Default::default()
    };
    let desc_pool = unsafe { dev.device.create_descriptor_pool(&dpci, None) }.map_err(|_| ())?;
    let layouts = vec![desc_layout; images.len()];
    let dsai = vk::DescriptorSetAllocateInfo {
        descriptor_pool: desc_pool, descriptor_set_count: images.len() as u32,
        p_set_layouts: layouts.as_ptr(), ..Default::default()
    };
    let desc_sets = unsafe { dev.device.allocate_descriptor_sets(&dsai) }.map_err(|_| ())?;
    desc_sets.iter().for_each(|ds| write_descriptors(dev, *ds, sampler, tex_input_view, tex_history_view));
    let cpci = vk::CommandPoolCreateInfo {
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER, queue_family_index: dev.qfam, ..Default::default()
    };
    let cmd_pool = unsafe { dev.device.create_command_pool(&cpci, None) }.map_err(|_| ())?;
    let cbai = vk::CommandBufferAllocateInfo {
        command_pool: cmd_pool, level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: images.len() as u32, ..Default::default()
    };
    let cmd_bufs = unsafe { dev.device.allocate_command_buffers(&cbai) }.map_err(|_| ())?;
    let semaphores = images.iter()
        .map(|_| unsafe { dev.device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }.map_err(|_| ()))
        .collect::<Result<Vec<_>, ()>>()?;
    let fences = images.iter()
        .map(|_| {
            let fci = vk::FenceCreateInfo { flags: vk::FenceCreateFlags::SIGNALED, ..Default::default() };
            unsafe { dev.device.create_fence(&fci, None) }.map_err(|_| ())
        })
        .collect::<Result<Vec<_>, ()>>()?;
    Ok(VkSwapState {
        device: dev_h, images, framebuffers, extent,
        render_pass, pipeline_layout, pipeline, desc_layout, desc_pool, desc_sets, sampler,
        tex_input, tex_input_mem, tex_input_view, tex_output, tex_output_mem, tex_output_view,
        tex_history, tex_history_mem, tex_history_view, history_init: false,
        cmd_pool, cmd_bufs, semaphores, fences,
    })
}

pub(crate) fn destroy_swap_state(dev: &VkDevState, st: &VkSwapState) {
    unsafe {
        let _ = dev.device.device_wait_idle();
        st.fences.iter().for_each(|f| dev.device.destroy_fence(*f, None));
        st.semaphores.iter().for_each(|s| dev.device.destroy_semaphore(*s, None));
        dev.device.destroy_command_pool(st.cmd_pool, None);
        dev.device.destroy_descriptor_pool(st.desc_pool, None);
        dev.device.destroy_sampler(st.sampler, None);
        dev.device.destroy_image_view(st.tex_input_view, None);
        dev.device.destroy_image(st.tex_input, None);
        dev.device.free_memory(st.tex_input_mem, None);
        dev.device.destroy_image_view(st.tex_output_view, None);
        dev.device.destroy_image(st.tex_output, None);
        dev.device.free_memory(st.tex_output_mem, None);
        dev.device.destroy_image_view(st.tex_history_view, None);
        dev.device.destroy_image(st.tex_history, None);
        dev.device.free_memory(st.tex_history_mem, None);
        dev.device.destroy_pipeline(st.pipeline, None);
        dev.device.destroy_pipeline_layout(st.pipeline_layout, None);
        dev.device.destroy_descriptor_set_layout(st.desc_layout, None);
        st.framebuffers.iter().for_each(|fb| dev.device.destroy_framebuffer(*fb, None));
        dev.device.destroy_render_pass(st.render_pass, None);
    }
}

fn call_create_swapchain_fallback(
    d: &VkDevState, dev: vk::Device,
    ci: *const vk::SwapchainCreateInfoKHR,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::SwapchainKHR,
) -> vk::Result {
    let r = unsafe { (d.swap_fp.create_swapchain_khr)(dev, ci, alloc, out) };
    match r {
        vk::Result::SUCCESS => {
            log_at(LogLevel::Info, "usage upgrade failed, swapchain created without postfx");
            vk::Result::SUCCESS
        }
        e => e,
    }
}

pub(crate) fn create_swapchain_with_fx(
    d: &VkDevState,
    dev: vk::Device,
    ci: *const vk::SwapchainCreateInfoKHR,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::SwapchainKHR,
    fx_wanted: bool,
) -> vk::Result {
    let upgraded = unsafe {
        vk::SwapchainCreateInfoKHR {
            image_usage: (*ci).image_usage
                | vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST,
            ..*ci
        }
    };
    let r1 = unsafe { (d.swap_fp.create_swapchain_khr)(dev, &upgraded, alloc, out) };
    match (r1, fx_wanted) {
        (vk::Result::SUCCESS, true) => {
            match build_swap_state(d, dev, unsafe { *out }, &upgraded) {
                Ok(st) => {
                    swap_put(unsafe { *out }.as_raw(), st);
                    log_at(LogLevel::Info, "swapchain registered for postfx");
                }
                Err(()) => log_at(LogLevel::Warn, "swapchain postfx setup failed, presenting untouched"),
            }
            vk::Result::SUCCESS
        }
        (vk::Result::SUCCESS, false) => vk::Result::SUCCESS,
        (_, _) => call_create_swapchain_fallback(d, dev, ci, alloc, out),
    }
}
