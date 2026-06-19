use std::collections::HashMap;
use std::mem;
use std::sync::RwLock;

use ash::vk;
use ash::vk::Handle;

use crate::logging::{LogLevel, log_at};
use super::layer::*;

#[derive(Clone)]
pub(crate) struct VkInstState {
    pub(crate) instance: ash::Instance,
    pub(crate) gipa: vk::PFN_vkGetInstanceProcAddr,
}

static INSTS: RwLock<Option<HashMap<u64, VkInstState>>> = RwLock::new(None);

pub(crate) fn insts_get(h: u64) -> Option<VkInstState> {
    INSTS.read().ok().and_then(|g| g.as_ref().and_then(|m| m.get(&h).cloned()))
}

pub(crate) fn insts_put(h: u64, v: VkInstState) {
    match INSTS.write() {
        Ok(mut g) => { g.get_or_insert_with(HashMap::new).insert(h, v); }
        Err(_) => (),
    }
}

pub(crate) fn insts_del(h: u64) {
    match INSTS.write() {
        Ok(mut g) => { g.as_mut().map(|m| m.remove(&h)); }
        Err(_) => (),
    }
}

pub(crate) fn owning_instance(phys: vk::PhysicalDevice) -> Option<(u64, VkInstState)> {
    let candidates: Vec<(u64, VkInstState)> = INSTS
        .read()
        .ok()
        .and_then(|g| g.as_ref().map(|m| m.iter().map(|(k, v)| (*k, v.clone())).collect()))
        .unwrap_or_default();
    candidates.into_iter().find(|(_, st)| {
        unsafe { st.instance.enumerate_physical_devices() }
            .map(|v| v.contains(&phys))
            .unwrap_or(false)
    })
}

fn register_instance(gipa: vk::PFN_vkGetInstanceProcAddr, handle: vk::Instance) {
    let static_fn = vk::StaticFn { get_instance_proc_addr: gipa };
    let instance = unsafe { ash::Instance::load(&static_fn, handle) };
    insts_put(handle.as_raw(), VkInstState { instance, gipa });
    log_at(LogLevel::Info, "vk instance registered");
}

fn invoke_create_instance(
    create_fn: unsafe extern "system" fn(),
    gipa: vk::PFN_vkGetInstanceProcAddr,
    ci: *const vk::InstanceCreateInfo,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::Instance,
) -> vk::Result {
    let r = unsafe {
        let cf: vk::PFN_vkCreateInstance = mem::transmute(create_fn);
        cf(ci, alloc, out)
    };
    match r {
        vk::Result::SUCCESS => {
            register_instance(gipa, unsafe { *out });
            vk::Result::SUCCESS
        }
        e => e,
    }
}

pub(crate) fn call_real_create_instance(
    link: Option<VkLayerLinkInfo>,
    ci: *const vk::InstanceCreateInfo,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::Instance,
) -> vk::Result {
    match link {
        None => vk::Result::ERROR_INITIALIZATION_FAILED,
        Some(l) => call_next_gipa(l.pfn_next_get_instance_proc_addr, vk::Instance::null(), "vkCreateInstance")
            .map(|f| invoke_create_instance(f, l.pfn_next_get_instance_proc_addr, ci, alloc, out))
            .unwrap_or(vk::Result::ERROR_INITIALIZATION_FAILED),
    }
}
