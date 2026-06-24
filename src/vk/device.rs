use std::collections::HashMap;
use std::mem;
use std::sync::RwLock;

use ash::vk;
use ash::vk::Handle;

use crate::logging::{LogLevel, log_at};
use super::instance::*;
use super::layer::*;

#[derive(Clone)]
pub(crate) struct VkDevState {
    pub(crate) device: ash::Device,
    pub(crate) mem_props: vk::PhysicalDeviceMemoryProperties,
    pub(crate) gdpa: vk::PFN_vkGetDeviceProcAddr,
    pub(crate) swap_fp: vk::KhrSwapchainFn,
}

#[derive(Clone, Copy)]
pub(crate) struct QueueBinding {
    pub(crate) device_raw: u64,
    pub(crate) family: u32,
}

static DEVS: RwLock<Option<HashMap<u64, VkDevState>>> = RwLock::new(None);
static QUEUE_TO_DEV: RwLock<Option<HashMap<u64, QueueBinding>>> = RwLock::new(None);

pub(crate) fn devs_get(h: u64) -> Option<VkDevState> {
    DEVS.read().ok().and_then(|g| g.as_ref().and_then(|m| m.get(&h).cloned()))
}

pub(crate) fn devs_gdpa(h: u64) -> Option<vk::PFN_vkGetDeviceProcAddr> {
    DEVS.read().ok().and_then(|g| g.as_ref().and_then(|m| m.get(&h).map(|d| d.gdpa)))
}

pub(crate) fn devs_put(h: u64, v: VkDevState) {
    match DEVS.write() {
        Ok(mut g) => { g.get_or_insert_with(HashMap::new).insert(h, v); }
        Err(_) => (),
    }
}

pub(crate) fn devs_del(h: u64) -> Option<VkDevState> {
    DEVS.write().ok().and_then(|mut g| g.as_mut().and_then(|m| m.remove(&h)))
}

pub(crate) fn queue_dev_get(q: u64) -> Option<QueueBinding> {
    QUEUE_TO_DEV.read().ok().and_then(|g| g.as_ref().and_then(|m| m.get(&q).copied()))
}

pub(crate) fn queue_dev_put(q: u64, b: QueueBinding) {
    match QUEUE_TO_DEV.write() {
        Ok(mut g) => { g.get_or_insert_with(HashMap::new).insert(q, b); }
        Err(_) => (),
    }
}

pub(crate) fn queue_owner(queue: vk::Queue) -> Option<(VkDevState, u32)> {
    queue_dev_get(queue.as_raw())
        .and_then(|b| devs_get(b.device_raw).map(|d| (d, b.family)))
}

fn register_device(
    gdpa: vk::PFN_vkGetDeviceProcAddr,
    handle: vk::Device,
    inst: &VkInstState,
    phys: vk::PhysicalDevice,
    _ci: *const vk::DeviceCreateInfo,
) {
    let mut inst_fp = inst.instance.fp_v1_0().clone();
    inst_fp.get_device_proc_addr = gdpa;
    let device = unsafe { ash::Device::load(&inst_fp, handle) };
    let swap_fp = vk::KhrSwapchainFn::load(|name| unsafe { mem::transmute(gdpa(handle, name.as_ptr())) });
    let mem_props = unsafe { inst.instance.get_physical_device_memory_properties(phys) };
    devs_put(handle.as_raw(), VkDevState { device, mem_props, gdpa, swap_fp });
    log_at(LogLevel::Info, "vk device registered");
}

fn invoke_create_device(
    create_fn: unsafe extern "system" fn(),
    link: &VkLayerLinkInfo,
    inst: &VkInstState,
    phys: vk::PhysicalDevice,
    ci: *const vk::DeviceCreateInfo,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::Device,
) -> vk::Result {
    let r = unsafe {
        let cf: vk::PFN_vkCreateDevice = mem::transmute(create_fn);
        cf(phys, ci, alloc, out)
    };
    match r {
        vk::Result::SUCCESS => {
            register_device(link.pfn_next_get_device_proc_addr, unsafe { *out }, inst, phys, ci);
            vk::Result::SUCCESS
        }
        e => e,
    }
}

pub(crate) fn call_real_create_device(
    link: Option<VkLayerLinkInfo>,
    phys: vk::PhysicalDevice,
    ci: *const vk::DeviceCreateInfo,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::Device,
) -> vk::Result {
    match (link, owning_instance(phys)) {
        (Some(l), Some((ih, inst))) => call_next_gipa(l.pfn_next_get_instance_proc_addr, vk::Instance::from_raw(ih), "vkCreateDevice")
            .map(|f| invoke_create_device(f, &l, &inst, phys, ci, alloc, out))
            .unwrap_or(vk::Result::ERROR_INITIALIZATION_FAILED),
        (_, _) => vk::Result::ERROR_INITIALIZATION_FAILED,
    }
}
