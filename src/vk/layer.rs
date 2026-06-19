use std::ffi::{CString, c_char, c_void};
use std::mem;
use std::ptr;

use ash::vk;
use ash::vk::Handle;

use crate::config::ensure_settings;
use crate::config::Settings;
use crate::consts::EffectDef;
use crate::consts::LAYER_DESC;
use crate::consts::LAYER_IFACE_VERSION;
use crate::consts::LAYER_LINK_INFO;
use crate::consts::LAYER_NAME;
use crate::consts::NULL_OK;
use crate::consts::REGISTRY;
use crate::effect::any_effect_enabled;
use crate::logging::init_log_level;
use crate::logging::log_at;
use crate::logging::LogLevel;
use crate::util::cstr_to_str;
use crate::watch::maybe_reload;

use super::instance::*;
use super::device::*;
use super::swapchain::*;
use super::present::*;

#[repr(C)]
struct VkNegotiateLayerInterface {
    s_type: i32,
    p_next: *mut c_void,
    loader_layer_interface_version: u32,
    pfn_get_instance_proc_addr: Option<vk::PFN_vkGetInstanceProcAddr>,
    pfn_get_device_proc_addr: Option<vk::PFN_vkGetDeviceProcAddr>,
    pfn_get_physical_device_proc_addr: Option<vk::PFN_vkVoidFunction>,
}

#[repr(C)]
pub(crate) struct VkLayerLink {
    pub(crate) p_next: *mut VkLayerLink,
    pub(crate) pfn_next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    pub(crate) pfn_next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
}

pub(crate) struct VkLayerLinkInfo {
    pub(crate) pfn_next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    pub(crate) pfn_next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
}

#[repr(C)]
pub(crate) struct VkLayerCreateInfo {
    pub(crate) s_type: vk::StructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) function: i32,
    pub(crate) u_layer_info: *mut VkLayerLink,
}

fn null_ok_name(name: &str) -> bool {
    NULL_OK.contains(&name)
}

fn vk_hooked_symbol(name: &str) -> Option<*mut c_void> {
    match name {
        "vkGetInstanceProcAddr" => Some(vkGetInstanceProcAddr as *mut c_void),
        "vkGetDeviceProcAddr" => Some(vkGetDeviceProcAddr as *mut c_void),
        "vkCreateInstance" => Some(vkCreateInstance as *mut c_void),
        "vkDestroyInstance" => Some(vkDestroyInstance as *mut c_void),
        "vkCreateDevice" => Some(vkCreateDevice as *mut c_void),
        "vkDestroyDevice" => Some(vkDestroyDevice as *mut c_void),
        "vkCreateSwapchainKHR" => Some(vkCreateSwapchainKHR as *mut c_void),
        "vkDestroySwapchainKHR" => Some(vkDestroySwapchainKHR as *mut c_void),
        "vkQueuePresentKHR" => Some(vkQueuePresentKHR as *mut c_void),
        "vkGetDeviceQueue" => Some(bones_GetDeviceQueue as *mut c_void),
        "vkGetDeviceQueue2" => Some(bones_GetDeviceQueue2 as *mut c_void),
        _ => None,
    }
}

fn null_ok_ptr(name: &str) -> *mut c_void {
    match name {
        "vkCreateInstance" => vkCreateInstance as *mut c_void,
        "vkEnumerateInstanceExtensionProperties" => bones_EnumerateInstanceExtensionProperties as *mut c_void,
        "vkEnumerateInstanceLayerProperties" => bones_EnumerateInstanceLayerProperties as *mut c_void,
        "vkEnumerateInstanceVersion" => bones_EnumerateInstanceVersion as *mut c_void,
        _ => ptr::null_mut(),
    }
}

fn non_null_ci(p: *const VkLayerCreateInfo) -> Option<*const VkLayerCreateInfo> {
    match p.is_null() {
        true => None,
        false => Some(p),
    }
}

pub(crate) fn chain_link_info(p_next: *const c_void, want: vk::StructureType) -> *mut VkLayerCreateInfo {
    std::iter::successors(non_null_ci(p_next as *const VkLayerCreateInfo), |p| {
        non_null_ci(unsafe { (**p).p_next as *const VkLayerCreateInfo })
    })
    .find(|p| unsafe { (**p).s_type == want && (**p).function == LAYER_LINK_INFO })
    .map(|p| p as *mut VkLayerCreateInfo)
    .unwrap_or(ptr::null_mut())
}

pub(crate) fn call_advance_chain(link: *mut VkLayerCreateInfo) -> Option<VkLayerLinkInfo> {
    match link.is_null() || unsafe { (*link).u_layer_info.is_null() } {
        true => None,
        false => unsafe {
            let li = (*link).u_layer_info;
            let out = VkLayerLinkInfo {
                pfn_next_get_instance_proc_addr: (*li).pfn_next_get_instance_proc_addr,
                pfn_next_get_device_proc_addr: (*li).pfn_next_get_device_proc_addr,
            };
            (*link).u_layer_info = (*li).p_next;
            Some(out)
        },
    }
}

pub(crate) fn call_next_gipa(gipa: vk::PFN_vkGetInstanceProcAddr, inst: vk::Instance, name: &str) -> vk::PFN_vkVoidFunction {
    let c = CString::new(name).unwrap_or_default();
    unsafe { gipa(inst, c.as_ptr()) }
}

pub(crate) fn call_next_gdpa(gdpa: vk::PFN_vkGetDeviceProcAddr, dev: vk::Device, name: &str) -> vk::PFN_vkVoidFunction {
    let c = CString::new(name).unwrap_or_default();
    unsafe { gdpa(dev, c.as_ptr()) }
}

fn copy_cstr(dst: &mut [c_char], s: &str) {
    s.bytes().take(dst.len() - 1).enumerate().for_each(|(i, b)| dst[i] = b as c_char);
}

unsafe extern "system" fn bones_EnumerateInstanceExtensionProperties(
    layer: *const c_char,
    count: *mut u32,
    _props: *mut vk::ExtensionProperties,
) -> vk::Result {
    match cstr_to_str(layer) == LAYER_NAME {
        true => { *count = 0; vk::Result::SUCCESS }
        false => vk::Result::ERROR_LAYER_NOT_PRESENT,
    }
}

unsafe extern "system" fn bones_EnumerateInstanceLayerProperties(
    count: *mut u32,
    props: *mut vk::LayerProperties,
) -> vk::Result {
    match props.is_null() {
        true => { *count = 1; vk::Result::SUCCESS }
        false => {
            let mut p = vk::LayerProperties {
                spec_version: vk::make_api_version(0, 1, 3, 0),
                implementation_version: 3,
                ..Default::default()
            };
            copy_cstr(&mut p.layer_name, LAYER_NAME);
            copy_cstr(&mut p.description, LAYER_DESC);
            *count = 1;
            *props = p;
            vk::Result::SUCCESS
        }
    }
}

unsafe extern "system" fn bones_EnumerateInstanceVersion(v: *mut u32) -> vk::Result {
    *v = vk::make_api_version(0, 1, 3, 0);
    vk::Result::SUCCESS
}

unsafe extern "system" fn bones_GetDeviceQueue(dev: vk::Device, qfam: u32, qidx: u32, out: *mut vk::Queue) {
    match devs_get(dev.as_raw()) {
        Some(d) => {
            let q = d.device.get_device_queue(qfam, qidx);
            queue_dev_put(q.as_raw(), dev.as_raw());
            *out = q;
        }
        None => log_at(LogLevel::Warn, "GetDeviceQueue on unregistered device"),
    }
}

unsafe extern "system" fn bones_GetDeviceQueue2(dev: vk::Device, info: *const vk::DeviceQueueInfo2, out: *mut vk::Queue) {
    match devs_get(dev.as_raw()) {
        Some(d) => {
            let q = d.device.get_device_queue2(&*info);
            queue_dev_put(q.as_raw(), dev.as_raw());
            *out = q;
        }
        None => log_at(LogLevel::Warn, "GetDeviceQueue2 on unregistered device"),
    }
}

unsafe extern "system" fn vkGetInstanceProcAddr(inst: vk::Instance, name: *const c_char) -> vk::PFN_vkVoidFunction {
    let n = cstr_to_str(name);
    let is_null = inst == vk::Instance::null();
    match (is_null, null_ok_name(n), vk_hooked_symbol(n)) {
        (true, true, _) => mem::transmute(null_ok_ptr(n)),
        (true, false, _) => None,
        (false, _, Some(p)) => mem::transmute(p),
        (false, _, None) => match insts_get(inst.as_raw()) {
            Some(st) => call_next_gipa(st.gipa, inst, n),
            None => None,
        },
    }
}

fn forward_device_proc(dev: vk::Device, name: &str) -> vk::PFN_vkVoidFunction {
    match devs_gdpa(dev.as_raw()) {
        Some(gdpa) => call_next_gdpa(gdpa, dev, name),
        None => None,
    }
}

unsafe extern "system" fn vkGetDeviceProcAddr(dev: vk::Device, name: *const c_char) -> vk::PFN_vkVoidFunction {
    let n = cstr_to_str(name);
    match vk_hooked_symbol(n) {
        Some(p) => mem::transmute(p),
        None => forward_device_proc(dev, n),
    }
}

#[no_mangle]
pub unsafe extern "system" fn vkNegotiateLoaderLayerInterfaceVersion(p: *mut c_void) -> vk::Result {
    let iface = p as *mut VkNegotiateLayerInterface;
    (*iface).loader_layer_interface_version = LAYER_IFACE_VERSION;
    (*iface).pfn_get_instance_proc_addr = Some(vkGetInstanceProcAddr);
    (*iface).pfn_get_device_proc_addr = Some(vkGetDeviceProcAddr);
    (*iface).pfn_get_physical_device_proc_addr = None;
    vk::Result::SUCCESS
}

unsafe extern "system" fn vkCreateInstance(
    ci: *const vk::InstanceCreateInfo,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::Instance,
) -> vk::Result {
    init_log_level();
    let link = call_advance_chain(chain_link_info((*ci).p_next, vk::StructureType::LOADER_INSTANCE_CREATE_INFO));
    call_real_create_instance(link, ci, alloc, out)
}

fn call_chain_destroy_instance(s: &VkInstState, inst: vk::Instance, alloc: *const vk::AllocationCallbacks) {
    match call_next_gipa(s.gipa, inst, "vkDestroyInstance") {
        Some(d) => unsafe {
            let df: vk::PFN_vkDestroyInstance = mem::transmute(d);
            df(inst, alloc);
        },
        None => (),
    }
}

unsafe extern "system" fn vkDestroyInstance(inst: vk::Instance, alloc: *const vk::AllocationCallbacks) {
    let st = insts_get(inst.as_raw());
    insts_del(inst.as_raw());
    match st {
        Some(s) => call_chain_destroy_instance(&s, inst, alloc),
        None => (),
    }
}

unsafe extern "system" fn vkCreateDevice(
    phys: vk::PhysicalDevice,
    ci: *const vk::DeviceCreateInfo,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::Device,
) -> vk::Result {
    let link = call_advance_chain(chain_link_info((*ci).p_next, vk::StructureType::LOADER_DEVICE_CREATE_INFO));
    call_real_create_device(link, phys, ci, alloc, out)
}

unsafe extern "system" fn vkDestroyDevice(dev: vk::Device, alloc: *const vk::AllocationCallbacks) {
    let removed = swap_del_for_device(dev);
    let st = devs_del(dev.as_raw());
    match st {
        Some(d) => {
            removed.iter().for_each(|s| destroy_swap_state(&d, s));
            d.device.destroy_device(alloc.as_ref());
        }
        None => (),
    }
}

unsafe extern "system" fn vkCreateSwapchainKHR(
    dev: vk::Device,
    ci: *const vk::SwapchainCreateInfoKHR,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::SwapchainKHR,
) -> vk::Result {
    let _ = ensure_settings();
    match devs_get(dev.as_raw()) {
        None => vk::Result::ERROR_INITIALIZATION_FAILED,
        Some(d) => create_swapchain_with_fx(&d, dev, ci, alloc, out),
    }
}
unsafe extern "system" fn vkDestroySwapchainKHR(
    dev: vk::Device,
    sc: vk::SwapchainKHR,
    alloc: *const vk::AllocationCallbacks,
) {
    pending_del(sc.as_raw());
    let st = swap_del(sc.as_raw());
    match (devs_get(dev.as_raw()), st) {
        (Some(d), Some(s)) => {
            destroy_swap_state(&d, &s);
            (d.swap_fp.destroy_swapchain_khr)(dev, sc, alloc);
        }
        (Some(d), None) => (d.swap_fp.destroy_swapchain_khr)(dev, sc, alloc),
        (None, _) => (),
    }
}

fn present_has_fx(info: *const vk::PresentInfoKHR, s: &Settings, reg: &[EffectDef]) -> bool {
    any_effect_enabled(s, reg) && unsafe {
        std::slice::from_raw_parts((*info).p_swapchains, (*info).swapchain_count as usize)
            .iter()
            .any(|sc| swap_has(sc.as_raw()))
    }
}

unsafe extern "system" fn vkQueuePresentKHR(queue: vk::Queue, info: *const vk::PresentInfoKHR) -> vk::Result {
    maybe_reload();
    retry_pending_registrations();
    match (queue_owner(queue), present_has_fx(info, &ensure_settings(), &REGISTRY)) {
        (Some(d), true) => run_vk_present_chain(&d, queue, info),
        (Some(d), false) => call_real_queue_present(&d, queue, info),
        (None, _) => vk::Result::ERROR_DEVICE_LOST,
    }
}
