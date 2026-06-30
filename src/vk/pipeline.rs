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

fn desc_layout_flags(push_desc: bool) -> vk::DescriptorSetLayoutCreateFlags {
    match push_desc {
        true => vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR,
        false => vk::DescriptorSetLayoutCreateFlags::empty(),
    }
}

pub(crate) fn create_desc_layout_fragment(dev: &VkDevState, push_desc: bool) -> Result<vk::DescriptorSetLayout, ()> {
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
        flags: desc_layout_flags(push_desc),
        binding_count: 2,
        p_bindings: bindings.as_ptr(),
        ..Default::default()
    };
    unsafe { dev.device.create_descriptor_set_layout(&ci, None) }.map_err(|_| ())
}

pub(crate) fn create_desc_layout_compute(dev: &VkDevState, push_desc: bool) -> Result<vk::DescriptorSetLayout, ()> {
    let bindings = [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::COMPUTE,
            ..Default::default()
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::COMPUTE,
            ..Default::default()
        },
        vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::COMPUTE,
            ..Default::default()
        },
    ];
    let ci = vk::DescriptorSetLayoutCreateInfo {
        flags: desc_layout_flags(push_desc),
        binding_count: 3,
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

fn destroy_module(dev: &VkDevState, m: vk::ShaderModule) {
    unsafe { dev.device.destroy_shader_module(m, None); }
}

fn destroy_module_pair(dev: &VkDevState, vs: vk::ShaderModule, fs: vk::ShaderModule) {
    destroy_module(dev, vs);
    destroy_module(dev, fs);
}

fn create_module_pair(dev: &VkDevState, vert_spv: &[u32], frag_spv: &[u32]) -> Result<(vk::ShaderModule, vk::ShaderModule), ()> {
    let vs = create_shader_module(dev, vert_spv)?;
    match create_shader_module(dev, frag_spv) {
        Ok(fs) => Ok((vs, fs)),
        Err(()) => {
            destroy_module(dev, vs);
            Err(())
        }
    }
}

fn build_stage_array(vs: vk::ShaderModule, fs: vk::ShaderModule, entry: *const i8) -> [vk::PipelineShaderStageCreateInfo; 2] {
    [
        vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX, module: vs, p_name: entry,
            ..Default::default()
        },
        vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::FRAGMENT, module: fs, p_name: entry,
            ..Default::default()
        },
    ]
}

fn build_pipeline_states(extent: vk::Extent2D) -> (
    vk::PipelineVertexInputStateCreateInfo,
    vk::PipelineInputAssemblyStateCreateInfo,
    vk::Viewport,
    vk::Rect2D,
    vk::PipelineRasterizationStateCreateInfo,
    vk::PipelineMultisampleStateCreateInfo,
    vk::PipelineColorBlendAttachmentState,
) {
    (
        vk::PipelineVertexInputStateCreateInfo { ..Default::default() },
        vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST, ..Default::default()
        },
        vk::Viewport {
            width: extent.width as f32, height: extent.height as f32, max_depth: 1.0, ..Default::default()
        },
        vk::Rect2D { extent, ..Default::default() },
        vk::PipelineRasterizationStateCreateInfo {
            polygon_mode: vk::PolygonMode::FILL, cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE, line_width: 1.0, ..Default::default()
        },
        vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1, ..Default::default()
        },
        vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::RGBA, ..Default::default()
        },
    )
}

fn create_graphics_pipeline_with_pnext(
    dev: &VkDevState,
    layout: vk::PipelineLayout,
    pass: vk::RenderPass,
    extent: vk::Extent2D,
    frag_spv: &[u32],
    p_next: *const std::ffi::c_void,
) -> Result<vk::Pipeline, ()> {
    let vert_spv = compile_vert_spirv()?;
    let (vs, fs) = create_module_pair(dev, &vert_spv, frag_spv)?;
    let entry = CString::new("main").unwrap_or_default();
    let stages = build_stage_array(vs, fs, entry.as_ptr());
    let (vi, ia, viewport, scissor, rs, ms, blend_attachment) = build_pipeline_states(extent);
    let vp = vk::PipelineViewportStateCreateInfo {
        viewport_count: 1, p_viewports: &viewport, scissor_count: 1, p_scissors: &scissor,
        ..Default::default()
    };
    let cb = vk::PipelineColorBlendStateCreateInfo {
        attachment_count: 1, p_attachments: &blend_attachment, ..Default::default()
    };
    let pci = vk::GraphicsPipelineCreateInfo {
        p_next,
        stage_count: 2, p_stages: stages.as_ptr(),
        p_vertex_input_state: &vi, p_input_assembly_state: &ia, p_viewport_state: &vp,
        p_rasterization_state: &rs, p_multisample_state: &ms, p_color_blend_state: &cb,
        layout, render_pass: pass, ..Default::default()
    };
    let r = unsafe { dev.device.create_graphics_pipelines(vk::PipelineCache::null(), &[pci], None) };
    destroy_module_pair(dev, vs, fs);
    r.map(|v| v[0]).map_err(|_| ())
}

pub(crate) fn create_pipeline(
    dev: &VkDevState,
    layout: vk::PipelineLayout,
    pass: vk::RenderPass,
    extent: vk::Extent2D,
    frag_spv: &[u32],
) -> Result<vk::Pipeline, ()> {
    create_graphics_pipeline_with_pnext(dev, layout, pass, extent, frag_spv, std::ptr::null())
}

pub(crate) fn create_pipeline_dynren(
    dev: &VkDevState,
    layout: vk::PipelineLayout,
    extent: vk::Extent2D,
    frag_spv: &[u32],
    color_format: vk::Format,
) -> Result<vk::Pipeline, ()> {
    let formats = [color_format];
    let rendering_ci = vk::PipelineRenderingCreateInfoKHR {
        color_attachment_count: 1,
        p_color_attachment_formats: formats.as_ptr(),
        ..Default::default()
    };
    create_graphics_pipeline_with_pnext(
        dev,
        layout,
        vk::RenderPass::null(),
        extent,
        frag_spv,
        &rendering_ci as *const _ as *const std::ffi::c_void,
    )
}

pub(crate) fn create_compute_pipeline(
    dev: &VkDevState,
    layout: vk::PipelineLayout,
    comp_spv: &[u32],
) -> Result<vk::Pipeline, ()> {
    let cs = create_shader_module(dev, comp_spv)?;
    let entry = CString::new("main").unwrap_or_default();
    let stage = vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::COMPUTE,
        module: cs,
        p_name: entry.as_ptr(),
        ..Default::default()
    };
    let pci = vk::ComputePipelineCreateInfo {
        stage,
        layout,
        ..Default::default()
    };
    let r = unsafe { dev.device.create_compute_pipelines(vk::PipelineCache::null(), &[pci], None) };
    destroy_module(dev, cs);
    r.map(|v| v[0]).map_err(|_| ())
}
