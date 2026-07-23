use std::collections::HashMap;
use std::ffi::CStr;
use std::mem;
use std::sync::RwLock;

use ash::vk;
use ash::vk::Handle;

use crate::consts::EXT_GPDP2;
use crate::logging::{LogLevel, log_at};
use super::layer::*;

#[derive(Clone)]
pub(crate) struct VkInstState {
    pub(crate) instance: ash::Instance,
    pub(crate) gipa: vk::PFN_vkGetInstanceProcAddr,
    pub(crate) features2_fp: Option<vk::PFN_vkGetPhysicalDeviceFeatures2>,
    pub(crate) api_version: u32,
    pub(crate) gpdp2: bool,
}

static INSTS: RwLock<Option<HashMap<u64, VkInstState>>> = RwLock::new(None);
static PHYS_OWNER: RwLock<Option<HashMap<u64, u64>>> = RwLock::new(None);

pub(crate) fn insts_get(h: u64) -> Option<VkInstState> {
    INSTS.read().ok().and_then(|g| g.as_ref().and_then(|m| m.get(&h).cloned()))
}

pub(crate) fn insts_put(h: u64, v: VkInstState) {
    match INSTS.write() {
        Ok(mut g) => { g.get_or_insert_with(HashMap::new).insert(h, v); }
        Err(_) => (),
    }
}

pub(crate) fn insts_del(h: u64) -> bool {
    phys_owner_del_inst(h);
    INSTS
        .write()
        .ok()
        .and_then(|mut g| {
            g.as_mut().map(|m| {
                m.remove(&h);
                m.is_empty()
            })
        })
        .unwrap_or(true)
}

fn phys_owner_get(phys: u64) -> Option<u64> {
    PHYS_OWNER
        .read()
        .ok()
        .and_then(|g| g.as_ref().and_then(|m| m.get(&phys).copied()))
}

fn phys_owner_put(phys: u64, inst: u64) {
    match PHYS_OWNER.write() {
        Ok(mut g) => {
            g.get_or_insert_with(HashMap::new).insert(phys, inst);
        }
        Err(_) => (),
    }
}

fn phys_owner_del_inst(inst: u64) {
    match PHYS_OWNER.write() {
        Ok(mut g) => {
            g.as_mut().map(|m| m.retain(|_, v| *v != inst));
        }
        Err(_) => (),
    }
}

fn instance_lists_phys(st: &VkInstState, phys: vk::PhysicalDevice) -> bool {
    unsafe { st.instance.enumerate_physical_devices() }
        .map(|v| v.contains(&phys))
        .unwrap_or(false)
}

fn scan_owning_instance(phys: vk::PhysicalDevice) -> Option<(u64, VkInstState)> {
    let candidates: Vec<(u64, VkInstState)> = INSTS
        .read()
        .ok()
        .and_then(|g| g.as_ref().map(|m| m.iter().map(|(k, v)| (*k, v.clone())).collect()))
        .unwrap_or_default();
    candidates
        .into_iter()
        .find(|(_, st)| instance_lists_phys(st, phys))
}

fn cache_owner(phys: vk::PhysicalDevice, found: Option<(u64, VkInstState)>) -> Option<(u64, VkInstState)> {
    found.map(|(h, st)| {
        phys_owner_put(phys.as_raw(), h);
        (h, st)
    })
}

pub(crate) fn owning_instance(phys: vk::PhysicalDevice) -> Option<(u64, VkInstState)> {
    match phys_owner_get(phys.as_raw()).and_then(|h| insts_get(h).map(|st| (h, st))) {
        Some(hit) => Some(hit),
        None => cache_owner(phys, scan_owning_instance(phys)),
    }
}

fn transmute_features2_fp(f: unsafe extern "system" fn()) -> vk::PFN_vkGetPhysicalDeviceFeatures2 {
    unsafe { mem::transmute(f) }
}

fn features2_symbol(gpdp2: bool, api_version: u32) -> Option<&'static str> {
    match (gpdp2, api_version >= vk::API_VERSION_1_1) {
        (true, _) => Some("vkGetPhysicalDeviceFeatures2KHR"),
        (false, true) => Some("vkGetPhysicalDeviceFeatures2"),
        (false, false) => None,
    }
}

fn resolve_features2_fp(
    gipa: vk::PFN_vkGetInstanceProcAddr,
    handle: vk::Instance,
    gpdp2: bool,
    api_version: u32,
) -> Option<vk::PFN_vkGetPhysicalDeviceFeatures2> {
    features2_symbol(gpdp2, api_version)
        .and_then(|name| call_next_gipa(gipa, handle, name))
        .map(transmute_features2_fp)
}

fn version_or_default(v: u32) -> u32 {
    match v {
        0 => vk::API_VERSION_1_0,
        v => v,
    }
}

fn app_api_version(ci: *const vk::InstanceCreateInfo) -> u32 {
    let app = unsafe { (*ci).p_application_info };
    match app.is_null() {
        true => vk::API_VERSION_1_0,
        false => version_or_default(unsafe { (*app).api_version }),
    }
}

fn instance_ext_names(ci: *const vk::InstanceCreateInfo) -> Vec<String> {
    (0..unsafe { (*ci).enabled_extension_count } as usize)
        .map(|i| unsafe {
            CStr::from_ptr(*(*ci).pp_enabled_extension_names.add(i))
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn app_has_gpdp2(ci: *const vk::InstanceCreateInfo) -> bool {
    instance_ext_names(ci).iter().any(|s| s == EXT_GPDP2)
}

fn register_instance(
    gipa: vk::PFN_vkGetInstanceProcAddr,
    handle: vk::Instance,
    api_version: u32,
    gpdp2: bool,
) {
    let static_fn = vk::StaticFn { get_instance_proc_addr: gipa };
    let instance = unsafe { ash::Instance::load(&static_fn, handle) };
    let features2_fp = resolve_features2_fp(gipa, handle, gpdp2, api_version);
    insts_put(handle.as_raw(), VkInstState { instance, gipa, features2_fp, api_version, gpdp2 });
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
            register_instance(gipa, unsafe { *out }, app_api_version(ci), app_has_gpdp2(ci));
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
