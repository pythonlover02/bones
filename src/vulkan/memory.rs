use ash::vk;

use super::device::VkDevState;

pub(crate) fn find_mem_type(props: &vk::PhysicalDeviceMemoryProperties, bits: u32, flags: vk::MemoryPropertyFlags) -> u32 {
    (0..props.memory_type_count)
        .find(|i| (bits & (1 << i)) != 0 && props.memory_types[*i as usize].property_flags.contains(flags))
        .unwrap_or(0)
}

fn sharing_for_families(families: &[u32]) -> (vk::SharingMode, u32, *const u32) {
    match families.len() >= 2 {
        true => (vk::SharingMode::CONCURRENT, families.len() as u32, families.as_ptr()),
        false => (vk::SharingMode::EXCLUSIVE, 0, std::ptr::null()),
    }
}

pub(crate) fn create_offscreen_image_shared(
    dev: &VkDevState,
    extent: vk::Extent2D,
    format: vk::Format,
    usage: vk::ImageUsageFlags,
    families: &[u32],
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), ()> {
    let (sharing_mode, family_count, family_ptr) = sharing_for_families(families);
    let ici = vk::ImageCreateInfo {
        image_type: vk::ImageType::TYPE_2D,
        format,
        extent: vk::Extent3D { width: extent.width, height: extent.height, depth: 1 },
        mip_levels: 1,
        array_layers: 1,
        samples: vk::SampleCountFlags::TYPE_1,
        tiling: vk::ImageTiling::OPTIMAL,
        usage,
        sharing_mode,
        queue_family_index_count: family_count,
        p_queue_family_indices: family_ptr,
        initial_layout: vk::ImageLayout::UNDEFINED,
        ..Default::default()
    };
    let img = unsafe { dev.device.create_image(&ici, None) }.map_err(|_| ())?;
    let req = unsafe { dev.device.get_image_memory_requirements(img) };
    let mai = vk::MemoryAllocateInfo {
        allocation_size: req.size,
        memory_type_index: find_mem_type(&dev.mem_props, req.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL),
        ..Default::default()
    };
    let mem = unsafe { dev.device.allocate_memory(&mai, None) }.map_err(|_| ())?;
    unsafe { dev.device.bind_image_memory(img, mem, 0) }.map_err(|_| ())?;
    let vci = vk::ImageViewCreateInfo {
        image: img,
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
    let view = unsafe { dev.device.create_image_view(&vci, None) }.map_err(|_| ())?;
    Ok((img, mem, view))
}

pub(crate) fn create_offscreen_image_with_usage(
    dev: &VkDevState,
    extent: vk::Extent2D,
    format: vk::Format,
    usage: vk::ImageUsageFlags,
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), ()> {
    create_offscreen_image_shared(dev, extent, format, usage, &[])
}

pub(crate) fn create_offscreen_image(
    dev: &VkDevState,
    extent: vk::Extent2D,
    format: vk::Format,
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), ()> {
    create_offscreen_image_with_usage(
        dev,
        extent,
        format,
        vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::COLOR_ATTACHMENT,
    )
}

pub(crate) fn create_offscreen_image_concurrent(
    dev: &VkDevState,
    extent: vk::Extent2D,
    format: vk::Format,
    families: &[u32],
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), ()> {
    create_offscreen_image_shared(
        dev,
        extent,
        format,
        vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::COLOR_ATTACHMENT,
        families,
    )
}

pub(crate) fn create_compute_output_image(
    dev: &VkDevState,
    extent: vk::Extent2D,
    format: vk::Format,
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), ()> {
    create_offscreen_image_with_usage(
        dev,
        extent,
        format,
        vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::STORAGE,
    )
}

pub(crate) fn create_compute_output_image_concurrent(
    dev: &VkDevState,
    extent: vk::Extent2D,
    format: vk::Format,
    families: &[u32],
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), ()> {
    create_offscreen_image_shared(
        dev,
        extent,
        format,
        vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::STORAGE,
        families,
    )
}
