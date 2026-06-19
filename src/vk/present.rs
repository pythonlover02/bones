use ash::vk;
use ash::vk::Handle;

use crate::consts::PUSH_BYTES;
use crate::logging::log_at;
use crate::logging::LogLevel;
use crate::timing::frame_time_fps;

use super::device::VkDevState;
use super::swapchain::*;

fn barrier(
    dev: &VkDevState, cb: vk::CommandBuffer, img: vk::Image,
    old: vk::ImageLayout, new: vk::ImageLayout,
    src: vk::AccessFlags, dst: vk::AccessFlags,
    src_stage: vk::PipelineStageFlags, dst_stage: vk::PipelineStageFlags,
) {
    let b = vk::ImageMemoryBarrier {
        src_access_mask: src, dst_access_mask: dst, old_layout: old, new_layout: new,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: img,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR, level_count: 1, layer_count: 1, ..Default::default()
        },
        ..Default::default()
    };
    unsafe { dev.device.cmd_pipeline_barrier(cb, src_stage, dst_stage, vk::DependencyFlags::empty(), &[], &[], &[b]) };
}

fn record_history_clear(dev: &VkDevState, cb: vk::CommandBuffer, hist: vk::Image) {
    barrier(dev, cb, hist,
        vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE,
        vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER);
    let black = vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] };
    let range = vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR, level_count: 1, layer_count: 1, ..Default::default()
    };
    unsafe { dev.device.cmd_clear_color_image(cb, hist, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &black, &[range]) };
    barrier(dev, cb, hist,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ,
        vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER);
}

fn call_copy_image(
    dev: &VkDevState, cb: vk::CommandBuffer, src: vk::Image, dst: vk::Image, region: &vk::ImageCopy,
) {
    unsafe {
        dev.device.cmd_copy_image(cb, src, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            dst, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[*region]);
    }
}

struct PostfxFrame {
    fence: vk::Fence,
    semaphore: vk::Semaphore,
    cmd_buf: vk::CommandBuffer,
    swap_image: vk::Image,
    extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    desc_set: vk::DescriptorSet,
    tex_input: vk::Image,
    tex_output: vk::Image,
    tex_history: vk::Image,
    need_history_init: bool,
}

fn extract_postfx_frame(st: &mut VkSwapState, idx: usize) -> PostfxFrame {
    let need_history_init = !st.history_init;
    st.history_init = true;
    PostfxFrame {
        fence: st.fences[idx],
        semaphore: st.semaphores[idx],
        cmd_buf: st.cmd_bufs[idx],
        swap_image: st.images[idx],
        extent: st.extent,
        render_pass: st.render_pass,
        framebuffer: st.framebuffers[0],
        pipeline: st.pipeline,
        pipeline_layout: st.pipeline_layout,
        desc_set: st.desc_sets[idx],
        tex_input: st.tex_input,
        tex_output: st.tex_output,
        tex_history: st.tex_history,
        need_history_init,
    }
}

fn record_postfx_commands(dev: &VkDevState, frame: &PostfxFrame) {
    let cb = frame.cmd_buf;
    let img = frame.swap_image;
    let (t, fps) = frame_time_fps();
    let push = [frame.extent.width as f32, frame.extent.height as f32, t, fps];
    let region = vk::ImageCopy {
        src_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR, layer_count: 1, ..Default::default()
        },
        dst_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR, layer_count: 1, ..Default::default()
        },
        extent: vk::Extent3D { width: frame.extent.width, height: frame.extent.height, depth: 1 },
        ..Default::default()
    };
    let bi = vk::CommandBufferBeginInfo {
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT, ..Default::default()
    };
    unsafe { let _ = dev.device.begin_command_buffer(cb, &bi); }

    match frame.need_history_init {
        true => record_history_clear(dev, cb, frame.tex_history),
        false => (),
    }

    barrier(dev, cb, img,
        vk::ImageLayout::PRESENT_SRC_KHR, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        vk::AccessFlags::MEMORY_READ, vk::AccessFlags::TRANSFER_READ,
        vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER);
    barrier(dev, cb, frame.tex_input,
        vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE,
        vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER);

    call_copy_image(dev, cb, img, frame.tex_input, &region);

    barrier(dev, cb, frame.tex_input,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ,
        vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER);
    barrier(dev, cb, img,
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::AccessFlags::TRANSFER_READ, vk::AccessFlags::TRANSFER_WRITE,
        vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::TRANSFER);
    barrier(dev, cb, frame.tex_output,
        vk::ImageLayout::UNDEFINED, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        vk::AccessFlags::empty(), vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT);

    let rpbi = vk::RenderPassBeginInfo {
        render_pass: frame.render_pass, framebuffer: frame.framebuffer,
        render_area: vk::Rect2D { extent: frame.extent, ..Default::default() }, ..Default::default()
    };
    unsafe {
        dev.device.cmd_begin_render_pass(cb, &rpbi, vk::SubpassContents::INLINE);
        dev.device.cmd_bind_pipeline(cb, vk::PipelineBindPoint::GRAPHICS, frame.pipeline);
        dev.device.cmd_bind_descriptor_sets(cb, vk::PipelineBindPoint::GRAPHICS, frame.pipeline_layout, 0, &[frame.desc_set], &[]);
        dev.device.cmd_push_constants(cb, frame.pipeline_layout, vk::ShaderStageFlags::FRAGMENT, 0,
            std::slice::from_raw_parts(push.as_ptr() as *const u8, PUSH_BYTES as usize));
        dev.device.cmd_draw(cb, 3, 1, 0, 0);
        dev.device.cmd_end_render_pass(cb);
    }

    barrier(dev, cb, frame.tex_output,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        vk::AccessFlags::COLOR_ATTACHMENT_WRITE, vk::AccessFlags::TRANSFER_READ,
        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, vk::PipelineStageFlags::TRANSFER);
    unsafe {
        dev.device.cmd_copy_image(cb, frame.tex_output, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            img, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]);
    }

    barrier(dev, cb, frame.tex_history,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::AccessFlags::SHADER_READ, vk::AccessFlags::TRANSFER_WRITE,
        vk::PipelineStageFlags::FRAGMENT_SHADER, vk::PipelineStageFlags::TRANSFER);
    unsafe {
        dev.device.cmd_copy_image(cb, frame.tex_output, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            frame.tex_history, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]);
    }
    barrier(dev, cb, frame.tex_history,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ,
        vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER);
    barrier(dev, cb, img,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR,
        vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::MEMORY_READ,
        vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::BOTTOM_OF_PIPE);

    unsafe { let _ = dev.device.end_command_buffer(cb); }
}

fn call_wait_and_reset_fence(dev: &VkDevState, fence: vk::Fence) {
    unsafe {
        let _ = dev.device.wait_for_fences(&[fence], true, u64::MAX);
        let _ = dev.device.reset_fences(&[fence]);
    }
}

fn submit_postfx(
    dev: &VkDevState,
    queue: vk::Queue,
    cb: vk::CommandBuffer,
    done: vk::Semaphore,
    fence: vk::Fence,
    waits: Vec<vk::Semaphore>,
) -> Vec<vk::Semaphore> {
    let stages = vec![vk::PipelineStageFlags::ALL_COMMANDS; waits.len()];
    let si = vk::SubmitInfo {
        wait_semaphore_count: waits.len() as u32,
        p_wait_semaphores: waits.as_ptr(),
        p_wait_dst_stage_mask: stages.as_ptr(),
        command_buffer_count: 1,
        p_command_buffers: &cb,
        signal_semaphore_count: 1,
        p_signal_semaphores: &done,
        ..Default::default()
    };
    match unsafe { dev.device.queue_submit(queue, &[si], fence) } {
        Ok(()) => vec![done],
        Err(_) => {
            log_at(LogLevel::Error, "postfx submit failed, passing original waits through");
            waits
        }
    }
}

fn record_postfx_pass(
    queue: vk::Queue, dev: &VkDevState, sc_raw: u64, idx: usize, waits: Vec<vk::Semaphore>,
) -> Vec<vk::Semaphore> {
    match swap_fx_lock_mut(sc_raw, |st| extract_postfx_frame(st, idx)) {
        None => waits,
        Some(frame) => {
            call_wait_and_reset_fence(dev, frame.fence);
            record_postfx_commands(dev, &frame);
            submit_postfx(dev, queue, frame.cmd_buf, frame.semaphore, frame.fence, waits)
        }
    }
}

pub(crate) fn call_real_queue_present(dev: &VkDevState, queue: vk::Queue, info: *const vk::PresentInfoKHR) -> vk::Result {
    unsafe { (dev.swap_fp.queue_present_khr)(queue, info) }
}

pub(crate) fn run_vk_present_chain(dev: &VkDevState, queue: vk::Queue, info: *const vk::PresentInfoKHR) -> vk::Result {
    let (swaps, indices, waits0) = unsafe {
        let n = (*info).swapchain_count as usize;
        let scs = std::slice::from_raw_parts((*info).p_swapchains, n).to_vec();
        let idx = std::slice::from_raw_parts((*info).p_image_indices, n).to_vec();
        let wn = (*info).wait_semaphore_count as usize;
        let ws = match wn { 0 => Vec::new(), _ => std::slice::from_raw_parts((*info).p_wait_semaphores, wn).to_vec() };
        (scs, idx, ws)
    };
    let finals = swaps.iter().zip(indices.iter())
        .filter(|(sc, _)| swap_has(sc.as_raw()))
        .fold(waits0, |w, (sc, idx)| record_postfx_pass(queue, dev, sc.as_raw(), *idx as usize, w));
    let patched = unsafe {
        vk::PresentInfoKHR {
            p_next: (*info).p_next,
            wait_semaphore_count: finals.len() as u32, p_wait_semaphores: finals.as_ptr(),
            swapchain_count: swaps.len() as u32, p_swapchains: swaps.as_ptr(),
            p_image_indices: indices.as_ptr(), p_results: (*info).p_results, ..Default::default()
        }
    };
    call_real_queue_present(dev, queue, &patched)
}
