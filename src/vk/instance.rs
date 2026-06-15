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

pub(crate) fn call_real_create_instance(
    link: Option<VkLayerLinkInfo>,
    ci: *const vk::InstanceCreateInfo,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::Instance,
) -> vk::Result {
    match link {
        None => vk::Result::ERROR_INITIALIZATION_FAILED,
        Some(l) => {
            let create = call_next_gipa(l.pfn_next_get_instance_proc_addr, vk::Instance::null(), "vkCreateInstance");
            match create {
                None => vk::Result::ERROR_INITIALIZATION_FAILED,
                Some(f) => unsafe {
                    let cf: vk::PFN_vkCreateInstance = mem::transmute(f);
                    let r = cf(ci, alloc, out);
                    match r {
                        vk::Result::SUCCESS => {
                            let static_fn = vk::StaticFn { get_instance_proc_addr: l.pfn_next_get_instance_proc_addr };
                            let instance = ash::Instance::load(&static_fn, *out);
                            insts_put((*out).as_raw(), VkInstState { instance, gipa: l.pfn_next_get_instance_proc_addr });
                            log_at(LogLevel::Info, "vk instance registered");
                            r
                        }
                        e => e,
                    }
                },
            }
        }
    }
}
