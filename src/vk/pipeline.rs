use std::ffi::CString;
use std::mem;

use ash::vk;

use crate::shader::compile_vert_spirv;

use super::device::VkDevState;

pub(crate) fn create_render_pass(dev: &VkDevState, format: vk::Format) -> Result<vk::RenderPass, ()> {
    let attachment = vk::AttachmentDescription {
        format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::DONT_CARE,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };
    let color_ref = vk::AttachmentReference { attachment: 0, layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL };
    let subpass = vk::SubpassDescription {
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        color_attachment_count: 1,
        p_color_attachments: &color_ref,
        ..Default::default()
    };
    let rpci = vk::RenderPassCreateInfo {
        attachment_count: 1,
        p_attachments: &attachment,
        subpass_count: 1,
        p_subpasses: &subpass,
        ..Default::default()
    };
    unsafe { dev.device.create_render_pass(&rpci, None) }.map_err(|_| ())
}

pub(crate) fn create_desc_layout(dev: &VkDevState) -> Result<vk::DescriptorSetLayout, ()> {
    let bindings = [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            ..Default::default()
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            ..Default::default()
        },
    ];
    let ci = vk::DescriptorSetLayoutCreateInfo {
        binding_count: 2,
        p_bindings: bindings.as_ptr(),
        ..Default::default()
    };
    unsafe { dev.device.create_descriptor_set_layout(&ci, None) }.map_err(|_| ())
}

fn create_shader_module(dev: &VkDevState, spv: &[u32]) -> Result<vk::ShaderModule, ()> {
    let ci = vk::ShaderModuleCreateInfo {
        code_size: spv.len() * mem::size_of::<u32>(),
        p_code: spv.as_ptr(),
        ..Default::default()
    };
    unsafe { dev.device.create_shader_module(&ci, None) }.map_err(|_| ())
}

pub(crate) fn create_pipeline(
    dev: &VkDevState,
    layout: vk::PipelineLayout,
    pass: vk::RenderPass,
    extent: vk::Extent2D,
    frag_spv: &[u32],
) -> Result<vk::Pipeline, ()> {
    let vert_spv = compile_vert_spirv()?;
    let vs = create_shader_module(dev, &vert_spv)?;
    let fs_r = create_shader_module(dev, frag_spv);
    let fs = match fs_r {
        Ok(m) => m,
        Err(()) => {
            unsafe { dev.device.destroy_shader_module(vs, None) };
            return Err(());
        }
    };
    let entry = CString::new("main").unwrap_or_default();
    let stages = [
        vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX, module: vs, p_name: entry.as_ptr(),
            ..Default::default()
        },
        vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::FRAGMENT, module: fs, p_name: entry.as_ptr(),
            ..Default::default()
        },
    ];
    let vi = vk::PipelineVertexInputStateCreateInfo { ..Default::default() };
    let ia = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST, ..Default::default()
    };
    let viewport = vk::Viewport {
        width: extent.width as f32, height: extent.height as f32, max_depth: 1.0, ..Default::default()
    };
    let scissor = vk::Rect2D { extent, ..Default::default() };
    let vp = vk::PipelineViewportStateCreateInfo {
        viewport_count: 1, p_viewports: &viewport, scissor_count: 1, p_scissors: &scissor,
        ..Default::default()
    };
    let rs = vk::PipelineRasterizationStateCreateInfo {
        polygon_mode: vk::PolygonMode::FILL, cull_mode: vk::CullModeFlags::NONE,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE, line_width: 1.0, ..Default::default()
    };
    let ms = vk::PipelineMultisampleStateCreateInfo {
        rasterization_samples: vk::SampleCountFlags::TYPE_1, ..Default::default()
    };
    let blend_attachment = vk::PipelineColorBlendAttachmentState {
        color_write_mask: vk::ColorComponentFlags::RGBA, ..Default::default()
    };
    let cb = vk::PipelineColorBlendStateCreateInfo {
        attachment_count: 1, p_attachments: &blend_attachment, ..Default::default()
    };
    let pci = vk::GraphicsPipelineCreateInfo {
        stage_count: 2, p_stages: stages.as_ptr(),
        p_vertex_input_state: &vi, p_input_assembly_state: &ia, p_viewport_state: &vp,
        p_rasterization_state: &rs, p_multisample_state: &ms, p_color_blend_state: &cb,
        layout, render_pass: pass, ..Default::default()
    };
    let r = unsafe { dev.device.create_graphics_pipelines(vk::PipelineCache::null(), &[pci], None) };
    unsafe {
        dev.device.destroy_shader_module(vs, None);
        dev.device.destroy_shader_module(fs, None);
    }
    r.map(|v| v[0]).map_err(|_| ())
}
