use std::collections::HashMap;
use std::ffi::c_char;
use std::ffi::c_void;
use std::ffi::CString;
use std::mem;
use std::sync::Arc;
use std::sync::RwLock;

use ash::vk;
use ash::vk::Handle;

use crate::config::ensure_settings;
use crate::config::Settings;
use crate::consts::REGISTRY;
use crate::consts::EXT_DYN_RENDER;
use crate::consts::EXT_MUTABLE_FMT;
use crate::consts::EXT_PUSH_DESC;
use crate::consts::EXT_SYNCHRONIZATION2;
use crate::logging::log_at;
use crate::logging::LogLevel;
use crate::shader::build_shaders;
use crate::shader::set_wg_limits;
use crate::shader::store_shaders;

use super::instance::*;
use super::layer::*;

pub(crate) struct VkDevState {
    pub(crate) device: ash::Device,
    pub(crate) phys: vk::PhysicalDevice,
    pub(crate) mem_props: vk::PhysicalDeviceMemoryProperties,
    pub(crate) gdpa: vk::PFN_vkGetDeviceProcAddr,
    pub(crate) swap_fp: vk::KhrSwapchainFn,
    pub(crate) sync2_fp: Option<vk::KhrSynchronization2Fn>,
    pub(crate) push_desc_fp: Option<vk::KhrPushDescriptorFn>,
    pub(crate) dynren_fp: Option<vk::KhrDynamicRenderingFn>,
    pub(crate) caps: DeviceCaps,
    pub(crate) app_queue_families: Vec<u32>,
    pub(crate) async_compute_queue: Option<vk::Queue>,
    pub(crate) instance_handle: u64,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Default)]
pub(crate) struct DeviceCaps {
    pub(crate) dynren: bool,
    pub(crate) pushdesc: bool,
    pub(crate) mutable_fmt: bool,
    pub(crate) storage_image_write_without_fmt: bool,
    pub(crate) sync2: bool,
    pub(crate) async_compute_family: Option<u32>,
    pub(crate) max_wg_x: u32,
    pub(crate) max_wg_y: u32,
    pub(crate) max_wg_invocations: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct QueueBinding {
    pub(crate) device_raw: u64,
    pub(crate) family: u32,
}

struct AugmentedDeviceCi {
    ci: vk::DeviceCreateInfo,
    caps: DeviceCaps,
    app_queue_families: Vec<u32>,
    _ext_cstrings: Vec<CString>,
    _ext_ptrs: Vec<*const c_char>,
    _queue_create_infos: Vec<vk::DeviceQueueCreateInfo>,
    _queue_priorities: Vec<f32>,
    _dynren_feat: Box<vk::PhysicalDeviceDynamicRenderingFeaturesKHR>,
    _sync2_feat: Box<vk::PhysicalDeviceSynchronization2FeaturesKHR>,
    _enabled_features: Box<vk::PhysicalDeviceFeatures>,
}

static DEVS: RwLock<Option<HashMap<u64, Arc<VkDevState>>>> = RwLock::new(None);
static QUEUE_TO_DEV: RwLock<Option<HashMap<u64, QueueBinding>>> = RwLock::new(None);

pub(crate) fn devs_get(h: u64) -> Option<Arc<VkDevState>> {
    DEVS.read()
        .ok()
        .and_then(|g| g.as_ref().and_then(|m| m.get(&h).cloned()))
}

pub(crate) fn devs_gdpa(h: u64) -> Option<vk::PFN_vkGetDeviceProcAddr> {
    DEVS.read()
        .ok()
        .and_then(|g| g.as_ref().and_then(|m| m.get(&h).map(|d| d.gdpa)))
}

pub(crate) fn devs_put(h: u64, v: VkDevState) {
    match DEVS.write() {
        Ok(mut g) => {
            g.get_or_insert_with(HashMap::new).insert(h, Arc::new(v));
        }
        Err(_) => (),
    }
}

pub(crate) fn devs_del(h: u64) -> Option<Arc<VkDevState>> {
    DEVS.write()
        .ok()
        .and_then(|mut g| g.as_mut().and_then(|m| m.remove(&h)))
}

pub(crate) fn queue_dev_get(q: u64) -> Option<QueueBinding> {
    QUEUE_TO_DEV
        .read()
        .ok()
        .and_then(|g| g.as_ref().and_then(|m| m.get(&q).copied()))
}

pub(crate) fn queue_dev_put(q: u64, b: QueueBinding) {
    match QUEUE_TO_DEV.write() {
        Ok(mut g) => {
            g.get_or_insert_with(HashMap::new).insert(q, b);
        }
        Err(_) => (),
    }
}

pub(crate) fn queue_owner(queue: vk::Queue) -> Option<(Arc<VkDevState>, u32)> {
    queue_dev_get(queue.as_raw())
        .and_then(|b| devs_get(b.device_raw).map(|d| (d, b.family)))
}

fn supported_device_exts(inst: &VkInstState, phys: vk::PhysicalDevice) -> Vec<String> {
    unsafe { inst.instance.enumerate_device_extension_properties(phys) }
        .unwrap_or_default()
        .into_iter()
        .map(|p| unsafe {
            std::ffi::CStr::from_ptr(p.extension_name.as_ptr())
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn ext_supported(list: &[String], name: &str) -> bool {
    list.iter().any(|s| s == name)
}

fn query_wg_limits(inst: &VkInstState, phys: vk::PhysicalDevice) -> (u32, u32, u32) {
    let props = unsafe { inst.instance.get_physical_device_properties(phys) };
    let l = props.limits;
    (
        l.max_compute_work_group_size[0],
        l.max_compute_work_group_size[1],
        l.max_compute_work_group_invocations,
    )
}

fn query_extension_features(
    inst: &VkInstState,
    phys: vk::PhysicalDevice,
) -> bool {
    let mut dynren = vk::PhysicalDeviceDynamicRenderingFeaturesKHR::default();
    let mut feats2 = vk::PhysicalDeviceFeatures2 {
        p_next: &mut dynren as *mut _ as *mut c_void,
        ..Default::default()
    };
    unsafe { inst.instance.get_physical_device_features2(phys, &mut feats2) };
    dynren.dynamic_rendering == vk::TRUE
}

fn query_core_features(inst: &VkInstState, phys: vk::PhysicalDevice) -> vk::PhysicalDeviceFeatures {
    unsafe { inst.instance.get_physical_device_features(phys) }
}

fn log_opt(requested: bool, available: bool, label: &str, dep: &str) -> bool {
    let chosen = requested && available;
    match (requested, available) {
        (true, false) => log_at(
            LogLevel::Warn,
            &format!("optimization not applied: {} requires {}", label, dep),
        ),
        (true, true) => log_at(
            LogLevel::Info,
            &format!("optimization {} active via {}", label, dep),
        ),
        (false, _) => (),
    }
    chosen
}

fn original_ext_cstrings(ci: &vk::DeviceCreateInfo) -> Vec<CString> {
    (0..ci.enabled_extension_count as usize)
        .map(|i| unsafe {
            std::ffi::CStr::from_ptr(*ci.pp_enabled_extension_names.add(i))
                .to_owned()
        })
        .collect()
}

fn add_if_chosen(chosen: bool, name: &str, exts: &mut Vec<CString>) {
    match chosen {
        true => exts.push(CString::new(name).unwrap_or_default()),
        false => (),
    }
}

fn merged_features(
    original: Option<&vk::PhysicalDeviceFeatures>,
    storage_write_without_fmt: bool,
) -> vk::PhysicalDeviceFeatures {
    let base = original.copied().unwrap_or_default();
    vk::PhysicalDeviceFeatures {
        shader_storage_image_write_without_format: match storage_write_without_fmt {
            true => vk::TRUE,
            false => base.shader_storage_image_write_without_format,
        },
        ..base
    }
}

fn original_features_ptr(ci: &vk::DeviceCreateInfo) -> Option<&vk::PhysicalDeviceFeatures> {
    match ci.p_enabled_features.is_null() {
        true => None,
        false => Some(unsafe { &*ci.p_enabled_features }),
    }
}

fn queue_family_props(inst: &VkInstState, phys: vk::PhysicalDevice) -> Vec<vk::QueueFamilyProperties> {
    unsafe { inst.instance.get_physical_device_queue_family_properties(phys) }
}

fn family_is_compute_only(p: &vk::QueueFamilyProperties) -> bool {
    p.queue_flags.contains(vk::QueueFlags::COMPUTE)
        && !p.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        && p.queue_count > 0
}

fn family_used_by_app(families: &[u32], idx: u32) -> bool {
    families.iter().any(|f| *f == idx)
}

fn extract_app_queue_families(ci: &vk::DeviceCreateInfo) -> Vec<u32> {
    (0..ci.queue_create_info_count as usize)
        .map(|i| unsafe { (*ci.p_queue_create_infos.add(i)).queue_family_index })
        .collect()
}

fn find_async_compute_family(
    inst: &VkInstState,
    phys: vk::PhysicalDevice,
    app_families: &[u32],
) -> Option<u32> {
    queue_family_props(inst, phys)
        .iter()
        .enumerate()
        .find(|(i, p)| family_is_compute_only(p) && !family_used_by_app(app_families, *i as u32))
        .map(|(i, _)| i as u32)
}

fn extra_queue_ci(
    family: u32,
    priorities: *const f32,
) -> vk::DeviceQueueCreateInfo {
    vk::DeviceQueueCreateInfo {
        queue_family_index: family,
        queue_count: 1,
        p_queue_priorities: priorities,
        ..Default::default()
    }
}

fn assemble_queue_infos(
    original: &vk::DeviceCreateInfo,
    extra: Option<vk::DeviceQueueCreateInfo>,
) -> Vec<vk::DeviceQueueCreateInfo> {
    let base: Vec<vk::DeviceQueueCreateInfo> = (0..original.queue_create_info_count as usize)
        .map(|i| unsafe { *original.p_queue_create_infos.add(i) })
        .collect();
    match extra {
        Some(q) => [base, vec![q]].concat(),
        None => base,
    }
}

fn query_sync2_feature(inst: &VkInstState, phys: vk::PhysicalDevice) -> bool {
    let mut f = vk::PhysicalDeviceSynchronization2FeaturesKHR::default();
    let mut f2 = vk::PhysicalDeviceFeatures2 {
        p_next: &mut f as *mut _ as *mut c_void,
        ..Default::default()
    };
    unsafe { inst.instance.get_physical_device_features2(phys, &mut f2) };
    f.synchronization2 == vk::TRUE
}

fn maybe_chain<T, F>(use_it: bool, head: *const c_void, feat: &mut T, set_next: F) -> *const c_void
where
    F: FnOnce(&mut T, *mut c_void),
{
    match use_it {
        true => {
            set_next(feat, head as *mut c_void);
            feat as *mut _ as *const c_void
        }
        false => head,
    }
}

fn build_augmented(
    inst: &VkInstState,
    phys: vk::PhysicalDevice,
    ci: *const vk::DeviceCreateInfo,
    s: &Settings,
) -> AugmentedDeviceCi {
    let original = unsafe { &*ci };
    let supported = supported_device_exts(inst, phys);
    let dynren_feat_ok = query_extension_features(inst, phys);
    let core = query_core_features(inst, phys);
    let (max_wg_x, max_wg_y, max_wg_inv) = query_wg_limits(inst, phys);
    let sync2_feat_ok = query_sync2_feature(inst, phys);

    let app_queue_families = extract_app_queue_families(original);
    let async_family_candidate = find_async_compute_family(inst, phys, &app_queue_families);

    let dynren_avail = ext_supported(&supported, EXT_DYN_RENDER) && dynren_feat_ok;
    let pushdesc_avail = ext_supported(&supported, EXT_PUSH_DESC);
    let mutable_avail = ext_supported(&supported, EXT_MUTABLE_FMT);
    let storage_write_avail = core.shader_storage_image_write_without_format == vk::TRUE;
    let sync2_avail = ext_supported(&supported, EXT_SYNCHRONIZATION2) && sync2_feat_ok;

    let use_dynren = log_opt(s.opt_dynren, dynren_avail, "dynamic_rendering", EXT_DYN_RENDER);
    let use_pushdesc = log_opt(s.opt_pushdesc, pushdesc_avail, "push_descriptors", EXT_PUSH_DESC);
    let use_storage_write = log_opt(s.compute, storage_write_avail, "compute_path", "shaderStorageImageWriteWithoutFormat");
    let use_sync2 = log_opt(s.opt_sync2, sync2_avail, "synchronization2", EXT_SYNCHRONIZATION2);
    let use_async_compute = log_opt(
        s.opt_async_compute && s.compute,
        async_family_candidate.is_some() && use_storage_write,
        "async_compute",
        "dedicated async compute queue family",
    );
    let async_family = match use_async_compute { true => async_family_candidate, false => None };

    let mut exts = original_ext_cstrings(original);
    add_if_chosen(use_dynren, EXT_DYN_RENDER, &mut exts);
    add_if_chosen(use_pushdesc, EXT_PUSH_DESC, &mut exts);
    add_if_chosen(mutable_avail, EXT_MUTABLE_FMT, &mut exts);
    add_if_chosen(use_sync2, EXT_SYNCHRONIZATION2, &mut exts);

    let ext_ptrs: Vec<*const c_char> = exts.iter().map(|c| c.as_ptr()).collect();

    let mut dynren_feat = Box::new(vk::PhysicalDeviceDynamicRenderingFeaturesKHR {
        dynamic_rendering: vk::TRUE,
        ..Default::default()
    });
    let mut sync2_feat = Box::new(vk::PhysicalDeviceSynchronization2FeaturesKHR {
        synchronization2: vk::TRUE,
        ..Default::default()
    });
    let enabled_features = Box::new(merged_features(original_features_ptr(original), use_storage_write));

    let mut head = original.p_next;
    head = maybe_chain(use_dynren, head, dynren_feat.as_mut(), |f, n| f.p_next = n);
    head = maybe_chain(use_sync2, head, sync2_feat.as_mut(), |f, n| f.p_next = n);

    let priorities = vec![1.0f32];
    let extra_q = async_family.map(|f| extra_queue_ci(f, priorities.as_ptr()));
    let queue_infos = assemble_queue_infos(original, extra_q);

    let new_ci = vk::DeviceCreateInfo {
        s_type: original.s_type,
        p_next: head,
        flags: original.flags,
        queue_create_info_count: queue_infos.len() as u32,
        p_queue_create_infos: queue_infos.as_ptr(),
        enabled_extension_count: ext_ptrs.len() as u32,
        pp_enabled_extension_names: ext_ptrs.as_ptr(),
        p_enabled_features: &*enabled_features,
        ..Default::default()
    };

    AugmentedDeviceCi {
        ci: new_ci,
        caps: DeviceCaps {
            dynren: use_dynren,
            pushdesc: use_pushdesc,
            mutable_fmt: mutable_avail,
            storage_image_write_without_fmt: use_storage_write,
            sync2: use_sync2,
            async_compute_family: async_family,
            max_wg_x,
            max_wg_y,
            max_wg_invocations: max_wg_inv,
        },
        app_queue_families,
        _ext_cstrings: exts,
        _ext_ptrs: ext_ptrs,
        _queue_create_infos: queue_infos,
        _queue_priorities: priorities,
        _dynren_feat: dynren_feat,
        _sync2_feat: sync2_feat,
        _enabled_features: enabled_features,
    }
}

fn load_sync2_fp(gdpa: vk::PFN_vkGetDeviceProcAddr, handle: vk::Device, enabled: bool) -> Option<vk::KhrSynchronization2Fn> {
    match enabled {
        true => Some(vk::KhrSynchronization2Fn::load(|name| unsafe {
            mem::transmute(gdpa(handle, name.as_ptr()))
        })),
        false => None,
    }
}

fn load_push_desc_fp(gdpa: vk::PFN_vkGetDeviceProcAddr, handle: vk::Device, enabled: bool) -> Option<vk::KhrPushDescriptorFn> {
    match enabled {
        true => Some(vk::KhrPushDescriptorFn::load(|name| unsafe {
            mem::transmute(gdpa(handle, name.as_ptr()))
        })),
        false => None,
    }
}

fn load_dynren_fp(gdpa: vk::PFN_vkGetDeviceProcAddr, handle: vk::Device, enabled: bool) -> Option<vk::KhrDynamicRenderingFn> {
    match enabled {
        true => Some(vk::KhrDynamicRenderingFn::load(|name| unsafe {
            mem::transmute(gdpa(handle, name.as_ptr()))
        })),
        false => None,
    }
}

fn fetch_async_queue(device: &ash::Device, family: Option<u32>) -> Option<vk::Queue> {
    family.map(|f| unsafe { device.get_device_queue(f, 0) })
}

fn register_device(
    gdpa: vk::PFN_vkGetDeviceProcAddr,
    handle: vk::Device,
    inst: &VkInstState,
    inst_handle: u64,
    phys: vk::PhysicalDevice,
    caps: DeviceCaps,
    app_queue_families: Vec<u32>,
) {
    let mut inst_fp = inst.instance.fp_v1_0().clone();
    inst_fp.get_device_proc_addr = gdpa;
    let device = unsafe { ash::Device::load(&inst_fp, handle) };
    let swap_fp = vk::KhrSwapchainFn::load(|name| unsafe {
        mem::transmute(gdpa(handle, name.as_ptr()))
    });
    let sync2_fp = load_sync2_fp(gdpa, handle, caps.sync2);
    let push_desc_fp = load_push_desc_fp(gdpa, handle, caps.pushdesc);
    let dynren_fp = load_dynren_fp(gdpa, handle, caps.dynren);
    let async_compute_queue = fetch_async_queue(&device, caps.async_compute_family);
    let mem_props = unsafe { inst.instance.get_physical_device_memory_properties(phys) };
    devs_put(
        handle.as_raw(),
        VkDevState {
            device,
            phys,
            mem_props,
            gdpa,
            swap_fp,
            sync2_fp,
            push_desc_fp,
            dynren_fp,
            caps,
            app_queue_families,
            async_compute_queue,
            instance_handle: inst_handle,
        },
    );
    log_at(LogLevel::Info, "vk device registered");
}

fn invoke_create_device(
    create_fn: unsafe extern "system" fn(),
    link: &VkLayerLinkInfo,
    inst: &VkInstState,
    inst_handle: u64,
    phys: vk::PhysicalDevice,
    aug: &AugmentedDeviceCi,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::Device,
) -> vk::Result {
    let r = unsafe {
        let cf: vk::PFN_vkCreateDevice = mem::transmute(create_fn);
        cf(phys, &aug.ci, alloc, out)
    };
    match r {
        vk::Result::SUCCESS => {
            register_device(
                link.pfn_next_get_device_proc_addr,
                unsafe { *out },
                inst,
                inst_handle,
                phys,
                aug.caps,
                aug.app_queue_families.clone(),
            );
            vk::Result::SUCCESS
        }
        e => e,
    }
}

fn apply_caps_and_rebuild_shaders(s: &Settings, caps: &DeviceCaps) {
    set_wg_limits(caps.max_wg_x, caps.max_wg_y, caps.max_wg_invocations);
    let _ = crate::shader::compile_vert_spirv();
    let spv = build_shaders(s, &REGISTRY);
    store_shaders(spv);
}

pub(crate) fn call_real_create_device(
    link: Option<VkLayerLinkInfo>,
    phys: vk::PhysicalDevice,
    ci: *const vk::DeviceCreateInfo,
    alloc: *const vk::AllocationCallbacks,
    out: *mut vk::Device,
) -> vk::Result {
    match (link, owning_instance(phys)) {
        (Some(l), Some((ih, inst))) => {
            let s = ensure_settings();
            let aug = build_augmented(&inst, phys, ci, &s);
            apply_caps_and_rebuild_shaders(&s, &aug.caps);
            call_next_gipa(
                l.pfn_next_get_instance_proc_addr,
                vk::Instance::from_raw(ih),
                "vkCreateDevice",
            )
            .map(|f| invoke_create_device(f, &l, &inst, ih, phys, &aug, alloc, out))
            .unwrap_or(vk::Result::ERROR_INITIALIZATION_FAILED)
        }
        (_, _) => vk::Result::ERROR_INITIALIZATION_FAILED,
    }
}

pub(crate) fn query_format_storage_supported(
    inst: &VkInstState,
    phys: vk::PhysicalDevice,
    format: vk::Format,
) -> bool {
    let props = unsafe { inst.instance.get_physical_device_format_properties(phys, format) };
    props
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::STORAGE_IMAGE)
}
