use ash::vk;
use ash::vk::Handle;

use crate::config::ensure_settings;
use crate::consts::FENCE_TIMEOUT_NS;
use crate::consts::PUSH_BYTES;
use crate::logging::log_at;
use crate::logging::LogLevel;
use crate::timing::frame_time_fps;

use super::device::VkDevState;
use super::swapchain::check_rebuild_pipeline;
use super::swapchain::ensure_fx;
use super::swapchain::ensure_submit_resources;
use super::swapchain::settings_has_any_fx;
use super::swapchain::swap_fx_lock_mut;
use super::swapchain::swap_has;
use super::swapchain::PostFxMode;
use super::swapchain::PostFxResources;
use super::swapchain::VkSwapState;

struct PostfxFrame {
    fence: vk::Fence,
    semaphore: vk::Semaphore,
    cmd_buf: vk::CommandBuffer,
    swap_image: vk::Image,
    extent: vk::Extent2D,
    fx_extent: vk::Extent2D,
    mode: PostFxMode,
    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    desc_set: vk::DescriptorSet,
    swap_view: vk::ImageView,
    tex_history_view: vk::ImageView,
    tex_output_view: vk::ImageView,
    sampler: vk::Sampler,
    use_push_desc: bool,
    submit_queue: vk::Queue,
    tex_output: vk::Image,
    tex_history: vk::Image,
    tex_upscaled: vk::Image,
    need_history_init: bool,
    needs_history: bool,
    compute_x: u32,
    compute_y: u32,
    can_blit: bool,
    filter: vk::Filter,
}

struct BarrierDesc {
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_access: vk::AccessFlags,
    dst_access: vk::AccessFlags,
    src_stage: vk::PipelineStageFlags,
    dst_stage: vk::PipelineStageFlags,
}

fn legacy_image_barrier(b: &BarrierDesc) -> vk::ImageMemoryBarrier {
    vk::ImageMemoryBarrier {
        src_access_mask: b.src_access,
        dst_access_mask: b.dst_access,
        old_layout: b.old_layout,
        new_layout: b.new_layout,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: b.image,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            level_count: 1,
            layer_count: 1,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn sync2_stage(s: vk::PipelineStageFlags) -> vk::PipelineStageFlags2 {
    vk::PipelineStageFlags2::from_raw(s.as_raw() as u64)
}

fn sync2_access(a: vk::AccessFlags) -> vk::AccessFlags2 {
    vk::AccessFlags2::from_raw(a.as_raw() as u64)
}

fn sync2_image_barrier(b: &BarrierDesc) -> vk::ImageMemoryBarrier2 {
    vk::ImageMemoryBarrier2 {
        src_stage_mask: sync2_stage(b.src_stage),
        src_access_mask: sync2_access(b.src_access),
        dst_stage_mask: sync2_stage(b.dst_stage),
        dst_access_mask: sync2_access(b.dst_access),
        old_layout: b.old_layout,
        new_layout: b.new_layout,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: b.image,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            level_count: 1,
            layer_count: 1,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn emit_legacy_batch(dev: &VkDevState, cb: vk::CommandBuffer, descs: &[BarrierDesc]) {
    descs.iter().for_each(|d| {
        let m = legacy_image_barrier(d);
        unsafe {
            dev.device.cmd_pipeline_barrier(
                cb,
                d.src_stage,
                d.dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[m],
            )
        };
    });
}

fn emit_sync2_batch(dev: &VkDevState, fp: &vk::KhrSynchronization2Fn, cb: vk::CommandBuffer, descs: &[BarrierDesc]) {
    let barriers: Vec<vk::ImageMemoryBarrier2> = descs.iter().map(sync2_image_barrier).collect();
    let dep = vk::DependencyInfo {
        image_memory_barrier_count: barriers.len() as u32,
        p_image_memory_barriers: barriers.as_ptr(),
        ..Default::default()
    };
    unsafe { (fp.cmd_pipeline_barrier2_khr)(cb, &dep) };
    let _ = dev;
}

fn emit_barriers(dev: &VkDevState, cb: vk::CommandBuffer, descs: &[BarrierDesc]) {
    match dev.sync2_fp.as_ref() {
        Some(fp) => emit_sync2_batch(dev, fp, cb, descs),
        None => emit_legacy_batch(dev, cb, descs),
    }
}

fn barrier(
    dev: &VkDevState,
    cb: vk::CommandBuffer,
    img: vk::Image,
    old: vk::ImageLayout,
    new: vk::ImageLayout,
    src: vk::AccessFlags,
    dst: vk::AccessFlags,
    src_stage: vk::PipelineStageFlags,
    dst_stage: vk::PipelineStageFlags,
) {
    emit_barriers(
        dev,
        cb,
        &[BarrierDesc {
            image: img,
            old_layout: old,
            new_layout: new,
            src_access: src,
            dst_access: dst,
            src_stage,
            dst_stage,
        }],
    );
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
        vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::COMPUTE_SHADER);
}

fn make_blit_region(src: vk::Extent2D, dst: vk::Extent2D) -> vk::ImageBlit {
    vk::ImageBlit {
        src_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR, layer_count: 1, ..Default::default()
        },
        src_offsets: [
            vk::Offset3D::default(),
            vk::Offset3D { x: src.width as i32, y: src.height as i32, z: 1 },
        ],
        dst_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR, layer_count: 1, ..Default::default()
        },
        dst_offsets: [
            vk::Offset3D::default(),
            vk::Offset3D { x: dst.width as i32, y: dst.height as i32, z: 1 },
        ],
    }
}

fn make_copy_region(extent: vk::Extent2D) -> vk::ImageCopy {
    vk::ImageCopy {
        src_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR, layer_count: 1, ..Default::default()
        },
        dst_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR, layer_count: 1, ..Default::default()
        },
        extent: vk::Extent3D { width: extent.width, height: extent.height, depth: 1 },
        ..Default::default()
    }
}

fn call_blit_image(
    dev: &VkDevState, cb: vk::CommandBuffer, src: vk::Image, dst: vk::Image,
    src_extent: vk::Extent2D, dst_extent: vk::Extent2D, filter: vk::Filter,
) {
    let region = make_blit_region(src_extent, dst_extent);
    unsafe {
        dev.device.cmd_blit_image(cb, src, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            dst, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region], filter)
    };
}

fn call_copy_image(dev: &VkDevState, cb: vk::CommandBuffer, src: vk::Image, dst: vk::Image, extent: vk::Extent2D) {
    let region = make_copy_region(extent);
    unsafe {
        dev.device.cmd_copy_image(cb, src, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            dst, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region])
    };
}

fn write_to_swap(
    dev: &VkDevState, cb: vk::CommandBuffer, src: vk::Image, dst: vk::Image,
    upscaled_img: vk::Image,
    src_extent: vk::Extent2D, dst_extent: vk::Extent2D, can_blit: bool, filter: vk::Filter,
) {
    match (src_extent.width == dst_extent.width && src_extent.height == dst_extent.height, can_blit) {
        (true, _) => call_copy_image(dev, cb, src, dst, dst_extent),
        (false, true) => {
            barrier(dev, cb, upscaled_img,
                vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER);
            call_blit_image(dev, cb, src, upscaled_img, src_extent, dst_extent, filter);
            barrier(dev, cb, upscaled_img,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::TRANSFER_READ,
                vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::TRANSFER);
            call_copy_image(dev, cb, upscaled_img, dst, dst_extent);
        }
        (false, false) => {
            log_at(LogLevel::Warn, "format lacks BLIT support, falling back to unscaled copy");
            let safe_extent = vk::Extent2D {
                width: src_extent.width.min(dst_extent.width),
                height: src_extent.height.min(dst_extent.height),
            };
            call_copy_image(dev, cb, src, dst, safe_extent);
        }
    }
}

fn pick_submit_queue(dev: &VkDevState, present_queue: vk::Queue, submit_family: u32) -> vk::Queue {
    match dev.caps.async_compute_family {
        Some(f) if f == submit_family => dev.async_compute_queue.unwrap_or(present_queue),
        _ => present_queue,
    }
}

fn legacy_desc_set(fx: &PostFxResources, idx: usize) -> vk::DescriptorSet {
    match fx.use_push_desc {
        true => vk::DescriptorSet::null(),
        false => fx.desc_sets[idx],
    }
}

fn extract_postfx_frame(
    st: &mut VkSwapState,
    idx: usize,
    dev: &VkDevState,
    present_queue: vk::Queue,
) -> Option<PostfxFrame> {
    let extent = st.extent;
    let fx_extent = st.fx_extent;
    let swap_image = st.images[idx];
    let fx = st.fx.as_mut()?;
    let r = fx.submit_res.as_ref()?;
    let need_history_init = !fx.history_init;
    fx.history_init = true;
    let swap_view = fx.swap_views[idx];
    let submit_queue = pick_submit_queue(dev, present_queue, fx.submit_family);
    Some(PostfxFrame {
        fence: r.fences[idx],
        semaphore: r.semaphores[idx],
        cmd_buf: r.cmd_bufs[idx],
        swap_image,
        extent,
        fx_extent,
        mode: fx.mode,
        render_pass: fx.render_pass,
        framebuffer: fx.framebuffer,
        pipeline: fx.pipeline,
        pipeline_layout: fx.pipeline_layout,
        desc_set: legacy_desc_set(fx, idx),
        swap_view,
        tex_history_view: fx.tex_history_view,
        tex_output_view: fx.tex_output_view,
        sampler: fx.sampler,
        use_push_desc: fx.use_push_desc,
        submit_queue,
        tex_output: fx.tex_output,
        tex_history: fx.tex_history,
        tex_upscaled: fx.tex_upscaled,
        need_history_init,
        needs_history: fx.needs_history,
        compute_x: fx.compute_x,
        compute_y: fx.compute_y,
        can_blit: fx.can_blit,
        filter: fx.filter,
    })
}

fn maybe_record_history_init(dev: &VkDevState, cb: vk::CommandBuffer, hist: vk::Image, needed: bool) {
    match needed {
        true => record_history_clear(dev, cb, hist),
        false => (),
    }
}

fn record_swap_to_shader_read(dev: &VkDevState, cb: vk::CommandBuffer, img: vk::Image) {
    barrier(dev, cb, img,
        vk::ImageLayout::PRESENT_SRC_KHR, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::AccessFlags::COLOR_ATTACHMENT_WRITE, vk::AccessFlags::SHADER_READ,
        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::COMPUTE_SHADER);
}

fn push_fragment_descriptors(
    dev: &VkDevState,
    fp: &vk::KhrPushDescriptorFn,
    cb: vk::CommandBuffer,
    layout: vk::PipelineLayout,
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
            dst_binding: 0, descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &ii, ..Default::default()
        },
        vk::WriteDescriptorSet {
            dst_binding: 1, descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &hi, ..Default::default()
        },
    ];
    unsafe {
        (fp.cmd_push_descriptor_set_khr)(
            cb,
            vk::PipelineBindPoint::GRAPHICS,
            layout,
            0,
            writes.len() as u32,
            writes.as_ptr(),
        )
    };
    let _ = dev;
}

fn push_compute_descriptors(
    dev: &VkDevState,
    fp: &vk::KhrPushDescriptorFn,
    cb: vk::CommandBuffer,
    layout: vk::PipelineLayout,
    sampler: vk::Sampler,
    input: vk::ImageView,
    history: vk::ImageView,
    output: vk::ImageView,
) {
    let ii = vk::DescriptorImageInfo {
        sampler, image_view: input,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    };
    let hi = vk::DescriptorImageInfo {
        sampler, image_view: history,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    };
    let oi = vk::DescriptorImageInfo {
        sampler: vk::Sampler::null(), image_view: output,
        image_layout: vk::ImageLayout::GENERAL,
    };
    let writes = [
        vk::WriteDescriptorSet {
            dst_binding: 0, descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &ii, ..Default::default()
        },
        vk::WriteDescriptorSet {
            dst_binding: 1, descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &hi, ..Default::default()
        },
        vk::WriteDescriptorSet {
            dst_binding: 2, descriptor_count: 1,
            descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
            p_image_info: &oi, ..Default::default()
        },
    ];
    unsafe {
        (fp.cmd_push_descriptor_set_khr)(
            cb,
            vk::PipelineBindPoint::COMPUTE,
            layout,
            0,
            writes.len() as u32,
            writes.as_ptr(),
        )
    };
    let _ = dev;
}

fn bind_or_push_fragment(dev: &VkDevState, frame: &PostfxFrame) {
    match (frame.use_push_desc, dev.push_desc_fp.as_ref()) {
        (true, Some(fp)) => push_fragment_descriptors(
            dev, fp, frame.cmd_buf, frame.pipeline_layout, frame.sampler,
            frame.swap_view, frame.tex_history_view,
        ),
        (_, _) => unsafe {
            dev.device.cmd_bind_descriptor_sets(
                frame.cmd_buf, vk::PipelineBindPoint::GRAPHICS,
                frame.pipeline_layout, 0, &[frame.desc_set], &[],
            )
        },
    }
}

fn bind_or_push_compute(dev: &VkDevState, frame: &PostfxFrame) {
    match (frame.use_push_desc, dev.push_desc_fp.as_ref()) {
        (true, Some(fp)) => push_compute_descriptors(
            dev, fp, frame.cmd_buf, frame.pipeline_layout, frame.sampler,
            frame.swap_view, frame.tex_history_view, frame.tex_output_view,
        ),
        (_, _) => unsafe {
            dev.device.cmd_bind_descriptor_sets(
                frame.cmd_buf, vk::PipelineBindPoint::COMPUTE,
                frame.pipeline_layout, 0, &[frame.desc_set], &[],
            )
        },
    }
}

fn record_fragment_pass(dev: &VkDevState, frame: &PostfxFrame, push_bytes: &[u8]) {
    let cb = frame.cmd_buf;
    barrier(dev, cb, frame.tex_output,
        vk::ImageLayout::UNDEFINED, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        vk::AccessFlags::empty(), vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT);
    let rpbi = vk::RenderPassBeginInfo {
        render_pass: frame.render_pass, framebuffer: frame.framebuffer,
        render_area: vk::Rect2D { extent: frame.fx_extent, ..Default::default() },
        ..Default::default()
    };
    unsafe {
        dev.device.cmd_begin_render_pass(cb, &rpbi, vk::SubpassContents::INLINE);
        dev.device.cmd_bind_pipeline(cb, vk::PipelineBindPoint::GRAPHICS, frame.pipeline);
    }
    bind_or_push_fragment(dev, frame);
    unsafe {
        dev.device.cmd_push_constants(cb, frame.pipeline_layout,
            vk::ShaderStageFlags::FRAGMENT, 0, push_bytes);
        dev.device.cmd_draw(cb, 3, 1, 0, 0);
        dev.device.cmd_end_render_pass(cb);
    }
    emit_barriers(dev, cb, &[BarrierDesc {
        image: frame.tex_output,
        old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        new_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        src_access: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        dst_access: vk::AccessFlags::TRANSFER_READ,
        src_stage: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_stage: vk::PipelineStageFlags::TRANSFER,
    }, BarrierDesc {
        image: frame.swap_image,
        old_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        src_access: vk::AccessFlags::SHADER_READ,
        dst_access: vk::AccessFlags::TRANSFER_WRITE,
        src_stage: vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::COMPUTE_SHADER,
        dst_stage: vk::PipelineStageFlags::TRANSFER,
    }]);
}

fn dispatch_dims(extent: vk::Extent2D, wg_x: u32, wg_y: u32) -> (u32, u32) {
    let gx = (extent.width + wg_x - 1) / wg_x.max(1);
    let gy = (extent.height + wg_y - 1) / wg_y.max(1);
    (gx, gy)
}

fn record_compute_pass(dev: &VkDevState, frame: &PostfxFrame, push_bytes: &[u8]) {
    let cb = frame.cmd_buf;
    barrier(dev, cb, frame.tex_output,
        vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL,
        vk::AccessFlags::empty(), vk::AccessFlags::SHADER_WRITE,
        vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER);
    let (gx, gy) = dispatch_dims(frame.fx_extent, frame.compute_x, frame.compute_y);
    unsafe {
        dev.device.cmd_bind_pipeline(cb, vk::PipelineBindPoint::COMPUTE, frame.pipeline);
    }
    bind_or_push_compute(dev, frame);
    unsafe {
        dev.device.cmd_push_constants(cb, frame.pipeline_layout,
            vk::ShaderStageFlags::COMPUTE, 0, push_bytes);
        dev.device.cmd_dispatch(cb, gx, gy, 1);
    }
    emit_barriers(dev, cb, &[BarrierDesc {
        image: frame.tex_output,
        old_layout: vk::ImageLayout::GENERAL,
        new_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        src_access: vk::AccessFlags::SHADER_WRITE,
        dst_access: vk::AccessFlags::TRANSFER_READ,
        src_stage: vk::PipelineStageFlags::COMPUTE_SHADER,
        dst_stage: vk::PipelineStageFlags::TRANSFER,
    }, BarrierDesc {
        image: frame.swap_image,
        old_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        src_access: vk::AccessFlags::SHADER_READ,
        dst_access: vk::AccessFlags::TRANSFER_WRITE,
        src_stage: vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::COMPUTE_SHADER,
        dst_stage: vk::PipelineStageFlags::TRANSFER,
    }]);
}

fn record_postfx_commands(dev: &VkDevState, frame: &PostfxFrame) {
    let cb = frame.cmd_buf;
    let img = frame.swap_image;
    let (t, fps) = frame_time_fps();
    let push = [frame.fx_extent.width as f32, frame.fx_extent.height as f32, t, fps];
    let push_bytes = unsafe {
        std::slice::from_raw_parts(push.as_ptr() as *const u8, PUSH_BYTES as usize)
    };
    let bi = vk::CommandBufferBeginInfo {
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
    };
    unsafe { let _ = dev.device.begin_command_buffer(cb, &bi); }

    maybe_record_history_init(dev, cb, frame.tex_history, frame.need_history_init);
    record_swap_to_shader_read(dev, cb, img);

    match frame.mode {
        PostFxMode::Fragment => record_fragment_pass(dev, frame, push_bytes),
        PostFxMode::Compute => record_compute_pass(dev, frame, push_bytes),
    }

    write_to_swap(dev, cb, frame.tex_output, img, frame.tex_upscaled, frame.fx_extent, frame.extent, frame.can_blit, frame.filter);
    record_end_of_frame_transitions(dev, frame);

    unsafe { let _ = dev.device.end_command_buffer(cb); }
}

fn record_end_of_frame_transitions(dev: &VkDevState, frame: &PostfxFrame) {
    match frame.needs_history {
        true => {
            barrier(dev, frame.cmd_buf, frame.tex_history,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::AccessFlags::SHADER_READ, vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::TRANSFER);
            call_copy_image(dev, frame.cmd_buf, frame.tex_output, frame.tex_history, frame.fx_extent);
            emit_barriers(dev, frame.cmd_buf, &[BarrierDesc {
                image: frame.tex_history,
                old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                src_access: vk::AccessFlags::TRANSFER_WRITE,
                dst_access: vk::AccessFlags::SHADER_READ,
                src_stage: vk::PipelineStageFlags::TRANSFER,
                dst_stage: vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::COMPUTE_SHADER,
            }, BarrierDesc {
                image: frame.swap_image,
                old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                src_access: vk::AccessFlags::TRANSFER_WRITE,
                dst_access: vk::AccessFlags::MEMORY_READ,
                src_stage: vk::PipelineStageFlags::TRANSFER,
                dst_stage: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            }]);
        }
        false => barrier(dev, frame.cmd_buf, frame.swap_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR,
            vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::MEMORY_READ,
            vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::BOTTOM_OF_PIPE),
    }
}

fn call_wait_and_reset_fence(dev: &VkDevState, fence: vk::Fence) {
    unsafe {
        let _ = dev.device.wait_for_fences(&[fence], true, FENCE_TIMEOUT_NS);
        let _ = dev.device.reset_fences(&[fence]);
    }
}

fn call_reset_command_buffer(dev: &VkDevState, cb: vk::CommandBuffer) {
    unsafe { let _ = dev.device.reset_command_buffer(cb, vk::CommandBufferResetFlags::empty()); }
}

fn signal_fence_empty(dev: &VkDevState, queue: vk::Queue, fence: vk::Fence) {
    unsafe { let _ = dev.device.queue_submit(queue, &[vk::SubmitInfo::default()], fence); }
}

fn recover_submit_failure(dev: &VkDevState, queue: vk::Queue, cb: vk::CommandBuffer, fence: vk::Fence) {
    call_reset_command_buffer(dev, cb);
    signal_fence_empty(dev, queue, fence);
}

fn submit_postfx(
    dev: &VkDevState,
    submit_queue: vk::Queue,
    cb: vk::CommandBuffer,
    done: vk::Semaphore,
    fence: vk::Fence,
    waits_in: Vec<vk::Semaphore>,
) -> Vec<vk::Semaphore> {
    let stages: Vec<vk::PipelineStageFlags> = waits_in
        .iter()
        .map(|_| vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::TRANSFER)
        .collect();
    let si = vk::SubmitInfo {
        wait_semaphore_count: waits_in.len() as u32,
        p_wait_semaphores: waits_in.as_ptr(),
        p_wait_dst_stage_mask: stages.as_ptr(),
        command_buffer_count: 1,
        p_command_buffers: &cb,
        signal_semaphore_count: 1,
        p_signal_semaphores: &done,
        ..Default::default()
    };
    match unsafe { dev.device.queue_submit(submit_queue, &[si], fence) } {
        Ok(()) => vec![done],
        Err(_) => {
            log_at(LogLevel::Error, "postfx submit failed, passing original waits through");
            recover_submit_failure(dev, submit_queue, cb, fence);
            waits_in
        }
    }
}

fn lock_extract_frame(
    dev: &VkDevState,
    sc_raw: u64,
    idx: usize,
    fam: u32,
    present_queue: vk::Queue,
) -> Option<PostfxFrame> {
    swap_fx_lock_mut(sc_raw, |st| {
        check_rebuild_pipeline(dev, st);
        let submit_family = st
            .fx
            .as_ref()
            .map(|fx| fx.submit_family)
            .unwrap_or(fam);
        match ensure_fx_and_submit(dev, st, submit_family) {
            true => extract_postfx_frame(st, idx, dev, present_queue),
            false => None,
        }
    })
    .flatten()
}

fn ensure_fx_and_submit(dev: &VkDevState, st: &mut VkSwapState, submit_family: u32) -> bool {
    let image_count = st.images.len();
    let present_family = st
        .fx
        .as_ref()
        .map(|fx| fx.submit_family)
        .unwrap_or(submit_family);
    let built = ensure_fx(dev, st, present_family);
    match (built, st.fx.as_mut()) {
        (true, Some(fx)) => ensure_submit_resources(dev, fx, fx.submit_family, image_count),
        (_, _) => false,
    }
}

fn record_postfx_pass(
    present_queue: vk::Queue,
    dev: &VkDevState,
    fam: u32,
    sc_raw: u64,
    idx: usize,
    waits: Vec<vk::Semaphore>,
) -> Vec<vk::Semaphore> {
    match lock_extract_frame(dev, sc_raw, idx, fam, present_queue) {
        None => waits,
        Some(frame) => {
            call_wait_and_reset_fence(dev, frame.fence);
            record_postfx_commands(dev, &frame);
            submit_postfx(dev, frame.submit_queue, frame.cmd_buf, frame.semaphore, frame.fence, waits)
        }
    }
}

pub(crate) fn call_real_queue_present(dev: &VkDevState, queue: vk::Queue, info: *const vk::PresentInfoKHR) -> vk::Result {
    unsafe { (dev.swap_fp.queue_present_khr)(queue, info) }
}

pub(crate) fn run_vk_present_chain(dev: &VkDevState, fam: u32, queue: vk::Queue, info: *const vk::PresentInfoKHR) -> vk::Result {
    let (swaps, indices, waits0) = unsafe {
        let n = (*info).swapchain_count as usize;
        let scs = std::slice::from_raw_parts((*info).p_swapchains, n).to_vec();
        let idx = std::slice::from_raw_parts((*info).p_image_indices, n).to_vec();
        let wn = (*info).wait_semaphore_count as usize;
        let ws = match wn { 0 => Vec::new(), _ => std::slice::from_raw_parts((*info).p_wait_semaphores, wn).to_vec() };
        (scs, idx, ws)
    };
    let s = ensure_settings();
    let active = settings_has_any_fx(&s);
    let finals = match active {
        true => swaps.iter().zip(indices.iter())
            .filter(|(sc, _)| swap_has(sc.as_raw()))
            .fold(waits0, |w, (sc, idx)| record_postfx_pass(queue, dev, fam, sc.as_raw(), *idx as usize, w)),
        false => waits0,
    };
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
