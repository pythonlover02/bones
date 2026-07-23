pub(crate) const LAYER_NAME: &str = "VK_LAYER_BONES_overlay";
pub(crate) const LAYER_DESC: &str = "Performance first Vulkan ubershader Post Processing layer for Linux.";

pub(crate) const ENABLE_VALUE: &str = "1";
pub(crate) const DEFAULT_PROFILE: &str = "bones";
pub(crate) const EXIT_EXEC_FAILED: i32 = 127;
pub(crate) const EXIT_USAGE: i32 = 1;
pub(crate) const EXIT_OK: i32 = 0;

pub(crate) const ENV_CONFIG_NAME: &str = "BONES_CONFIG_NAME";
pub(crate) const ENV_CONFIG: &str = "BONES_CONFIG";
pub(crate) const ENV_LOG: &str = "BONES_LOG";

pub(crate) const ENV_RES_SCALE: &str = "BONES_RESOLUTION_SCALE";
pub(crate) const ENV_OPT_DYNREN: &str = "BONES_OPTIMIZE_DYNAMIC_RENDERING";
pub(crate) const ENV_OPT_PUSHDESC: &str = "BONES_OPTIMIZE_PUSH_DESCRIPTORS";
pub(crate) const ENV_OPT_SYNC2: &str = "BONES_OPTIMIZE_SYNC2";
pub(crate) const ENV_OPT_MUTABLE_FMT: &str = "BONES_OPTIMIZE_MUTABLE_FORMAT";
pub(crate) const ENV_OPT_ASYNC_COMPUTE: &str = "BONES_OPTIMIZE_ASYNC_COMPUTE";
pub(crate) const ENV_COMPUTE: &str = "BONES_COMPUTE";
pub(crate) const ENV_COMPUTE_X: &str = "BONES_COMPUTE_X";
pub(crate) const ENV_COMPUTE_Y: &str = "BONES_COMPUTE_Y";

pub(crate) const ENV_ENABLE: &str = "BONES_ENABLE";

pub(crate) const FLATPAK_CMD: &str = "flatpak";
pub(crate) const FLATPAK_RUN: &str = "run";
pub(crate) const FLATPAK_INJECT: &str = "/usr/lib/extensions/vulkan/bones/bin/bones-flatpak";

pub(crate) const CONFIG_SEP: char = ';';
pub(crate) const POLL_INTERVAL_MS: i32 = 250;

pub(crate) const RES_SCALE_KEY: &str = "resolution_scale";
pub(crate) const OPT_DYNREN_KEY: &str = "optimize_dynamic_rendering";
pub(crate) const OPT_PUSHDESC_KEY: &str = "optimize_push_descriptors";
pub(crate) const OPT_SYNC2_KEY: &str = "optimize_sync2";
pub(crate) const OPT_MUTABLE_FMT_KEY: &str = "optimize_mutable_format";
pub(crate) const OPT_ASYNC_COMPUTE_KEY: &str = "optimize_async_compute";
pub(crate) const COMPUTE_KEY: &str = "compute";
pub(crate) const COMPUTE_X_KEY: &str = "compute_x";
pub(crate) const COMPUTE_Y_KEY: &str = "compute_y";

pub(crate) const RES_SCALE_MIN: f32 = 0.05;
pub(crate) const RES_SCALE_DEFAULT: f32 = 1.0;
pub(crate) const COMPUTE_X_DEFAULT: u32 = 8;
pub(crate) const COMPUTE_Y_DEFAULT: u32 = 8;

pub(crate) const EXT_DYN_RENDER: &str = "VK_KHR_dynamic_rendering";
pub(crate) const EXT_PUSH_DESC: &str = "VK_KHR_push_descriptor";
pub(crate) const EXT_MUTABLE_FMT: &str = "VK_KHR_swapchain_mutable_format";
pub(crate) const EXT_SYNCHRONIZATION2: &str = "VK_KHR_synchronization2";
pub(crate) const EXT_GPDP2: &str = "VK_KHR_get_physical_device_properties2";
pub(crate) const EXT_DEPTH_STENCIL_RESOLVE: &str = "VK_KHR_depth_stencil_resolve";
pub(crate) const EXT_CREATE_RENDERPASS2: &str = "VK_KHR_create_renderpass2";
pub(crate) const EXT_MULTIVIEW: &str = "VK_KHR_multiview";
pub(crate) const EXT_MAINTENANCE2: &str = "VK_KHR_maintenance2";
pub(crate) const EXT_IMAGE_FORMAT_LIST: &str = "VK_KHR_image_format_list";
pub(crate) const EXT_SWAPCHAIN_BASE: &str = "VK_KHR_swapchain";
pub(crate) const FN_PRESENT_RECTANGLES: &str = "vkGetPhysicalDevicePresentRectanglesKHR";

pub(crate) const DYNREN_EXTS: [&str; 5] = [
    EXT_DYN_RENDER,
    EXT_DEPTH_STENCIL_RESOLVE,
    EXT_CREATE_RENDERPASS2,
    EXT_MULTIVIEW,
    EXT_MAINTENANCE2,
];

pub(crate) const MUTABLE_FMT_EXTS: [&str; 4] = [
    EXT_MUTABLE_FMT,
    EXT_MAINTENANCE2,
    EXT_IMAGE_FORMAT_LIST,
    EXT_SWAPCHAIN_BASE,
];

pub(crate) const GENERAL_BOOL_KEYS: [&str; 6] = [
    OPT_DYNREN_KEY,
    OPT_PUSHDESC_KEY,
    OPT_SYNC2_KEY,
    OPT_MUTABLE_FMT_KEY,
    OPT_ASYNC_COMPUTE_KEY,
    COMPUTE_KEY,
];

pub(crate) const GENERAL_FLOAT_KEYS: [&str; 1] = [RES_SCALE_KEY];
pub(crate) const GENERAL_UINT_KEYS: [&str; 2] = [COMPUTE_X_KEY, COMPUTE_Y_KEY];

pub(crate) const LOG_FD: i32 = 2;
pub(crate) const US_PER_S: f32 = 1_000_000.0;
pub(crate) const MAX_FPS_REPORT: f32 = 9999.0;
pub(crate) const MIN_DELTA_US: u64 = 1;
pub(crate) const INOTIFY_BUF: usize = 4096;
pub(crate) const DEBOUNCE_MS: u64 = 100;
pub(crate) const FENCE_TIMEOUT_NS: u64 = 5_000_000_000;

pub(crate) const LOG_LEVEL_OFF: i32 = 0;
pub(crate) const LOG_LEVEL_ERROR: i32 = 1;
pub(crate) const LOG_LEVEL_WARN: i32 = 2;
pub(crate) const LOG_LEVEL_INFO: i32 = 3;
pub(crate) const DEFAULT_LOG_LEVEL: i32 = 2;

pub(crate) const PUSH_BYTES: u32 = 16;
pub(crate) const LAYER_IFACE_VERSION: u32 = 2;
pub(crate) const LAYER_LINK_INFO: i32 = 0;

pub(crate) const NULL_OK: [&str; 4] = [
    "vkCreateInstance",
    "vkEnumerateInstanceVersion",
    "vkEnumerateInstanceExtensionProperties",
    "vkEnumerateInstanceLayerProperties",
];

pub(crate) const USAGE: &str = "usage: bones [PROFILE] -- COMMAND [ARGS...]\n  bones -- CMD            run CMD with the default profile (~/.config/bones/bones-config.toml)\n  bones NAME -- CMD       run CMD with profile ~/.config/bones/NAME-config.toml\n";

pub(crate) const HEAD: &str = include_str!("../config/bones-config.toml");

pub(crate) const VERT_SRC: &str = include_str!("../shaders/uber.vert");

pub(crate) const GLSL_VERSION: &str = "#version 460";
pub(crate) const IO_COMPUTE: &str = include_str!("../shaders/io_compute.glsl");
pub(crate) const IO_FRAGMENT: &str = include_str!("../shaders/io_fragment.glsl");
pub(crate) const GLSL_CONSTANTS: &str = include_str!("../shaders/constants.glsl");
pub(crate) const GLSL_FUNCTIONS: &str = include_str!("../shaders/functions.glsl");
pub(crate) const GRADE_OPEN: &str = include_str!("../shaders/grade_open.glsl");
pub(crate) const GRADE_CLAMP: &str = include_str!("../shaders/grade_clamp.glsl");
pub(crate) const GRADE_CLOSE: &str = include_str!("../shaders/grade_close.glsl");
pub(crate) const MAIN_OPEN: &str = include_str!("../shaders/main_open.glsl");
pub(crate) const MAIN_FETCH: &str = include_str!("../shaders/main_fetch.glsl");
pub(crate) const MAIN_FETCH_CHROMATIC: &str = include_str!("../shaders/inline/chromatic_aberration.glsl");
pub(crate) const MAIN_GRADE: &str = include_str!("../shaders/main_grade.glsl");
pub(crate) const MAIN_CLOSE: &str = include_str!("../shaders/main_close.glsl");
pub(crate) const HISTORY_FRAG: &str = include_str!("../shaders/shared/history.glsl");
pub(crate) const HISTORY_HUD_MASK: &str = include_str!("../shaders/shared/history_hud_mask.glsl");
pub(crate) const HUD_BOX_FRAG: &str = include_str!("../shaders/overlay/fps_hud_box.glsl");
pub(crate) const STABILIZER_FRAG: &str = include_str!("../shaders/temporal/stabilizer.glsl");

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectKind {
    Geometric,
    Denoise,
    AntiAliasing,
    Sharpening,
    LocalContrast,
    Blur,
    ImageQuality,
    Inline,
    Exposure,
    Tonemapping,
    WhiteBalance,
    ColorGrading,
    ChannelCurves,
    ColorBalance,
    SelectiveColor,
    Stylization,
    Toon,
    Accessibility,
    HardwareSimulation,
    Temporal,
    Overlay,
}

pub(crate) struct EffectDef {
    pub(crate) name: &'static str,
    pub(crate) kind: EffectKind,
}

pub(crate) const REGISTRY: [EffectDef; 130] = [
    EffectDef { name: "identity", kind: EffectKind::Geometric },
    EffectDef { name: "mirror_horizontal", kind: EffectKind::Geometric },
    EffectDef { name: "mirror_vertical", kind: EffectKind::Geometric },
    EffectDef { name: "rotate_90", kind: EffectKind::Geometric },
    EffectDef { name: "rotate_180", kind: EffectKind::Geometric },
    EffectDef { name: "rotate_270", kind: EffectKind::Geometric },
    EffectDef { name: "center_zoom", kind: EffectKind::Geometric },
    EffectDef { name: "polynomial_distort", kind: EffectKind::Geometric },
    EffectDef { name: "barrel_undistort", kind: EffectKind::Geometric },
    EffectDef { name: "fisheye_warp", kind: EffectKind::Geometric },
    EffectDef { name: "trapezoid_warp", kind: EffectKind::Geometric },
    EffectDef { name: "sharp_bilinear", kind: EffectKind::Geometric },
    EffectDef { name: "bilateral_denoise", kind: EffectKind::Denoise },
    EffectDef { name: "kuwahara_paint", kind: EffectKind::Toon },
    EffectDef { name: "luma_edge_aa", kind: EffectKind::AntiAliasing },
    EffectDef { name: "normal_filter_aa", kind: EffectKind::AntiAliasing },
    EffectDef { name: "morphological_aa", kind: EffectKind::AntiAliasing },
    EffectDef { name: "subpixel_aa", kind: EffectKind::AntiAliasing },
    EffectDef { name: "contrast_adaptive_sharpen", kind: EffectKind::Sharpening },
    EffectDef { name: "robust_contrast_sharpen", kind: EffectKind::Sharpening },
    EffectDef { name: "edge_directed_sharpen", kind: EffectKind::Sharpening },
    EffectDef { name: "laplacian_sharpen", kind: EffectKind::Sharpening },
    EffectDef { name: "luminance_sharpen", kind: EffectKind::Sharpening },
    EffectDef { name: "midtone_clarity", kind: EffectKind::Sharpening },
    EffectDef { name: "falloff_sharpen", kind: EffectKind::Sharpening },
    EffectDef { name: "power_curve_sharpen", kind: EffectKind::Sharpening },
    EffectDef { name: "unsharp_mask", kind: EffectKind::Sharpening },
    EffectDef { name: "local_contrast", kind: EffectKind::LocalContrast },
    EffectDef { name: "gaussian_blur", kind: EffectKind::Blur },
    EffectDef { name: "box_blur", kind: EffectKind::Blur },
    EffectDef { name: "bokeh_blur", kind: EffectKind::Blur },
    EffectDef { name: "tilt_shift_blur", kind: EffectKind::Blur },
    EffectDef { name: "radial_blur", kind: EffectKind::Blur },
    EffectDef { name: "gradient_deband", kind: EffectKind::ImageQuality },
    EffectDef { name: "threshold_bloom", kind: EffectKind::ImageQuality },
    EffectDef { name: "ghost_flare", kind: EffectKind::ImageQuality },
    EffectDef { name: "chromatic_aberration", kind: EffectKind::Inline },
    EffectDef { name: "red_halation", kind: EffectKind::Inline },
    EffectDef { name: "anamorphic_streak", kind: EffectKind::Inline },
    EffectDef { name: "linear_exposure", kind: EffectKind::Exposure },
    EffectDef { name: "aces_tonemap", kind: EffectKind::Tonemapping },
    EffectDef { name: "agx_tonemap", kind: EffectKind::Tonemapping },
    EffectDef { name: "reinhard_tonemap", kind: EffectKind::Tonemapping },
    EffectDef { name: "hable_tonemap", kind: EffectKind::Tonemapping },
    EffectDef { name: "lottes_tonemap", kind: EffectKind::Tonemapping },
    EffectDef { name: "uchimura_tonemap", kind: EffectKind::Tonemapping },
    EffectDef { name: "tony_tonemap", kind: EffectKind::Tonemapping },
    EffectDef { name: "khronos_tonemap", kind: EffectKind::Tonemapping },
    EffectDef { name: "neutral_white_balance", kind: EffectKind::WhiteBalance },
    EffectDef { name: "warm_temperature", kind: EffectKind::WhiteBalance },
    EffectDef { name: "cool_temperature", kind: EffectKind::WhiteBalance },
    EffectDef { name: "saturation_contrast_grade", kind: EffectKind::ColorGrading },
    EffectDef { name: "levels_remap", kind: EffectKind::ColorGrading },
    EffectDef { name: "gamma_correct", kind: EffectKind::ColorGrading },
    EffectDef { name: "vibrance_boost", kind: EffectKind::ColorGrading },
    EffectDef { name: "hsl_transform", kind: EffectKind::ColorGrading },
    EffectDef { name: "split_tone", kind: EffectKind::ColorGrading },
    EffectDef { name: "lift_gamma_gain", kind: EffectKind::ColorGrading },
    EffectDef { name: "hermite_curves", kind: EffectKind::ColorGrading },
    EffectDef { name: "red_channel_curve", kind: EffectKind::ChannelCurves },
    EffectDef { name: "green_channel_curve", kind: EffectKind::ChannelCurves },
    EffectDef { name: "blue_channel_curve", kind: EffectKind::ChannelCurves },
    EffectDef { name: "trizone_color_balance", kind: EffectKind::ColorBalance },
    EffectDef { name: "red_selective_saturate", kind: EffectKind::SelectiveColor },
    EffectDef { name: "green_selective_saturate", kind: EffectKind::SelectiveColor },
    EffectDef { name: "blue_selective_saturate", kind: EffectKind::SelectiveColor },
    EffectDef { name: "dynamic_range_crush", kind: EffectKind::Stylization },
    EffectDef { name: "duotone_map", kind: EffectKind::Stylization },
    EffectDef { name: "color_wash_tint", kind: EffectKind::Stylization },
    EffectDef { name: "posterize_quantize", kind: EffectKind::Stylization },
    EffectDef { name: "bleach_bypass", kind: EffectKind::Stylization },
    EffectDef { name: "technicolor_process", kind: EffectKind::Stylization },
    EffectDef { name: "midpoint_contrast", kind: EffectKind::Stylization },
    EffectDef { name: "color_invert", kind: EffectKind::Stylization },
    EffectDef { name: "luminance_grayscale", kind: EffectKind::Stylization },
    EffectDef { name: "cel_shade", kind: EffectKind::Toon },
    EffectDef { name: "manga_screentone", kind: EffectKind::Toon },
    EffectDef { name: "crosshatch_shade", kind: EffectKind::Toon },
    EffectDef { name: "protanopia_simulation", kind: EffectKind::Accessibility },
    EffectDef { name: "deuteranopia_simulation", kind: EffectKind::Accessibility },
    EffectDef { name: "tritanopia_simulation", kind: EffectKind::Accessibility },
    EffectDef { name: "protanopia_correct", kind: EffectKind::Accessibility },
    EffectDef { name: "deuteranopia_correct", kind: EffectKind::Accessibility },
    EffectDef { name: "tritanopia_correct", kind: EffectKind::Accessibility },
    EffectDef { name: "radial_vignette", kind: EffectKind::Inline },
    EffectDef { name: "cinematic_letterbox", kind: EffectKind::Inline },
    EffectDef { name: "ordered_dither", kind: EffectKind::Inline },
    EffectDef { name: "ps1_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "saturn_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "n64_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "dreamcast_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "ps2_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "xbox_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "psp_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "ps3_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "xbox360_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "crt_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "phosphor_amber", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "phosphor_green", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "phosphor_red", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "scanline_darken", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "oled_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "vhs_simulation", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "lcd_subpixel", kind: EffectKind::HardwareSimulation },
    EffectDef { name: "ink_outline", kind: EffectKind::Toon },
    EffectDef { name: "neighborhood_clamp_aa", kind: EffectKind::Temporal },
    EffectDef { name: "motion_reject_denoise", kind: EffectKind::Temporal },
    EffectDef { name: "motion_detect_blur", kind: EffectKind::Temporal },
    EffectDef { name: "constant_blend_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "shutter_angle_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "spline_interp_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "variance_decay_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "dualrate_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "luminance_gate_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "contrast_gate_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "gradient_gate_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "sigma_clip_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "mitchell_kernel_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "ycocg_clip_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "bilateral_history_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "perceptual_chroma_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "frequency_split_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "horn_schunck_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "convergent_accumulate", kind: EffectKind::Temporal },
    EffectDef { name: "dualwarp_flow_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "variance_flow_accumulate", kind: EffectKind::Temporal },
    EffectDef { name: "edge_reconstruct_smooth", kind: EffectKind::Temporal },
    EffectDef { name: "gaussian_grain", kind: EffectKind::Inline },
    EffectDef { name: "fps_hud", kind: EffectKind::Overlay },
    EffectDef { name: "crosshair_overlay", kind: EffectKind::Overlay },
];

pub(crate) struct FragDef {
    pub(crate) name: &'static str,
    pub(crate) src: &'static str,
}

pub(crate) const UV_WARP_FRAGS: [FragDef; 19] = [
    FragDef { name: "mirror_horizontal", src: include_str!("../shaders/geometric/mirror_horizontal.glsl") },
    FragDef { name: "mirror_vertical", src: include_str!("../shaders/geometric/mirror_vertical.glsl") },
    FragDef { name: "rotate_90", src: include_str!("../shaders/geometric/rotate_90.glsl") },
    FragDef { name: "rotate_180", src: include_str!("../shaders/geometric/rotate_180.glsl") },
    FragDef { name: "rotate_270", src: include_str!("../shaders/geometric/rotate_270.glsl") },
    FragDef { name: "center_zoom", src: include_str!("../shaders/geometric/center_zoom.glsl") },
    FragDef { name: "polynomial_distort", src: include_str!("../shaders/geometric/polynomial_distort.glsl") },
    FragDef { name: "barrel_undistort", src: include_str!("../shaders/geometric/barrel_undistort.glsl") },
    FragDef { name: "fisheye_warp", src: include_str!("../shaders/geometric/fisheye_warp.glsl") },
    FragDef { name: "trapezoid_warp", src: include_str!("../shaders/geometric/trapezoid_warp.glsl") },
    FragDef { name: "sharp_bilinear", src: include_str!("../shaders/geometric/sharp_bilinear.glsl") },
    FragDef { name: "crt_simulation", src: include_str!("../shaders/hardware_simulation/crt_uv.glsl") },
    FragDef { name: "vhs_simulation", src: include_str!("../shaders/hardware_simulation/vhs_uv.glsl") },
    FragDef { name: "ps1_simulation", src: include_str!("../shaders/hardware_simulation/ps1_uv.glsl") },
    FragDef { name: "saturn_simulation", src: include_str!("../shaders/hardware_simulation/saturn_uv.glsl") },
    FragDef { name: "n64_simulation", src: include_str!("../shaders/hardware_simulation/n64_uv.glsl") },
    FragDef { name: "ps2_simulation", src: include_str!("../shaders/hardware_simulation/ps2_uv.glsl") },
    FragDef { name: "psp_simulation", src: include_str!("../shaders/hardware_simulation/psp_uv.glsl") },
    FragDef { name: "ps3_simulation", src: include_str!("../shaders/hardware_simulation/ps3_uv.glsl") },
];

pub(crate) struct RoleDef {
    pub(crate) src: &'static str,
    pub(crate) consumers: &'static [&'static str],
}

pub(crate) const ROLE_TAPS_CROSS: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/taps_cross.glsl"),
    consumers: &[
        "luma_edge_aa", "normal_filter_aa", "morphological_aa", "subpixel_aa",
        "contrast_adaptive_sharpen", "robust_contrast_sharpen", "edge_directed_sharpen",
        "laplacian_sharpen", "luminance_sharpen", "midtone_clarity", "falloff_sharpen",
        "power_curve_sharpen", "unsharp_mask", "local_contrast", "gaussian_blur",
        "threshold_bloom", "red_halation", "neighborhood_clamp_aa", "sigma_clip_smooth",
        "dualrate_smooth", "horn_schunck_smooth", "frequency_split_smooth",
        "gradient_gate_smooth", "bilateral_history_smooth", "contrast_gate_smooth",
        "dualwarp_flow_smooth", "variance_flow_accumulate", "edge_reconstruct_smooth",
        "ycocg_clip_smooth", "mitchell_kernel_smooth", "bilateral_denoise",
        "kuwahara_paint", "ink_outline",
    ],
};

pub(crate) const ROLE_TAPS_CORNER: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/taps_corner.glsl"),
    consumers: &[
        "luma_edge_aa", "morphological_aa", "subpixel_aa", "contrast_adaptive_sharpen",
        "falloff_sharpen", "unsharp_mask", "gaussian_blur", "threshold_bloom",
        "red_halation", "neighborhood_clamp_aa", "sigma_clip_smooth",
        "mitchell_kernel_smooth", "bilateral_denoise", "kuwahara_paint",
    ],
};

pub(crate) const ROLE_SUM_CROSS: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/sum_cross.glsl"),
    consumers: &[
        "contrast_adaptive_sharpen", "robust_contrast_sharpen", "edge_directed_sharpen",
        "laplacian_sharpen", "luminance_sharpen", "midtone_clarity", "power_curve_sharpen",
        "unsharp_mask", "local_contrast", "gaussian_blur", "falloff_sharpen",
        "dualrate_smooth", "sigma_clip_smooth", "frequency_split_smooth",
        "variance_flow_accumulate", "contrast_gate_smooth", "bilateral_history_smooth",
        "mitchell_kernel_smooth",
    ],
};

pub(crate) const ROLE_SUM_CORNER: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/sum_corner.glsl"),
    consumers: &[
        "contrast_adaptive_sharpen", "falloff_sharpen", "unsharp_mask",
        "gaussian_blur", "sigma_clip_smooth", "mitchell_kernel_smooth",
    ],
};

pub(crate) const ROLE_BOUNDS_CROSS: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/bounds_cross.glsl"),
    consumers: &[
        "contrast_adaptive_sharpen", "robust_contrast_sharpen", "neighborhood_clamp_aa",
        "dualwarp_flow_smooth", "edge_reconstruct_smooth",
    ],
};

pub(crate) const ROLE_BOUNDS_3X3: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/bounds_3x3.glsl"),
    consumers: &["contrast_adaptive_sharpen", "neighborhood_clamp_aa"],
};

pub(crate) const ROLE_GRAD_LUMA: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/grad_luma.glsl"),
    consumers: &[
        "edge_directed_sharpen", "normal_filter_aa", "horn_schunck_smooth",
        "dualwarp_flow_smooth", "variance_flow_accumulate", "edge_reconstruct_smooth",
        "gradient_gate_smooth", "ink_outline",
    ],
};

pub(crate) const ROLE_HISTORY_CROSS: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/history_cross.glsl"),
    consumers: &["bilateral_history_smooth", "frequency_split_smooth"],
};

pub(crate) const ROLE_GRADED_BOUNDS_CROSS: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/graded_bounds_cross.glsl"),
    consumers: &["dualwarp_flow_smooth", "edge_reconstruct_smooth"],
};

pub(crate) const ROLE_GRADED_BOUNDS_3X3: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/graded_bounds_3x3.glsl"),
    consumers: &["neighborhood_clamp_aa"],
};

pub(crate) const ROLE_GRADED_AVG_CROSS: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/graded_avg_cross.glsl"),
    consumers: &[
        "bilateral_history_smooth", "frequency_split_smooth", "contrast_gate_smooth",
        "mitchell_kernel_smooth",
    ],
};

pub(crate) const ROLE_GRADED_AVG_CORNER: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/graded_avg_corner.glsl"),
    consumers: &["mitchell_kernel_smooth"],
};

pub(crate) const ROLE_GRADED_TAPS_CROSS: RoleDef = RoleDef {
    src: include_str!("../shaders/shared/graded_taps_cross.glsl"),
    consumers: &["ycocg_clip_smooth"],
};

pub(crate) const TAP_ROLES: [&RoleDef; 8] = [
    &ROLE_TAPS_CROSS,
    &ROLE_TAPS_CORNER,
    &ROLE_SUM_CROSS,
    &ROLE_SUM_CORNER,
    &ROLE_BOUNDS_CROSS,
    &ROLE_BOUNDS_3X3,
    &ROLE_GRAD_LUMA,
    &ROLE_HISTORY_CROSS,
];

pub(crate) const GRADED_ROLES: [&RoleDef; 5] = [
    &ROLE_GRADED_BOUNDS_CROSS,
    &ROLE_GRADED_BOUNDS_3X3,
    &ROLE_GRADED_AVG_CROSS,
    &ROLE_GRADED_AVG_CORNER,
    &ROLE_GRADED_TAPS_CROSS,
];

pub(crate) const SPATIAL_FRAGS: [FragDef; 24] = [
    FragDef { name: "bilateral_denoise", src: include_str!("../shaders/denoise/bilateral_denoise.glsl") },
    FragDef { name: "kuwahara_paint", src: include_str!("../shaders/toon/kuwahara_paint.glsl") },
    FragDef { name: "luma_edge_aa", src: include_str!("../shaders/anti_aliasing/luma_edge_aa.glsl") },
    FragDef { name: "normal_filter_aa", src: include_str!("../shaders/anti_aliasing/normal_filter_aa.glsl") },
    FragDef { name: "morphological_aa", src: include_str!("../shaders/anti_aliasing/morphological_aa.glsl") },
    FragDef { name: "subpixel_aa", src: include_str!("../shaders/anti_aliasing/subpixel_aa.glsl") },
    FragDef { name: "contrast_adaptive_sharpen", src: include_str!("../shaders/sharpening/contrast_adaptive_sharpen.glsl") },
    FragDef { name: "robust_contrast_sharpen", src: include_str!("../shaders/sharpening/robust_contrast_sharpen.glsl") },
    FragDef { name: "edge_directed_sharpen", src: include_str!("../shaders/sharpening/edge_directed_sharpen.glsl") },
    FragDef { name: "laplacian_sharpen", src: include_str!("../shaders/sharpening/laplacian_sharpen.glsl") },
    FragDef { name: "luminance_sharpen", src: include_str!("../shaders/sharpening/luminance_sharpen.glsl") },
    FragDef { name: "midtone_clarity", src: include_str!("../shaders/sharpening/midtone_clarity.glsl") },
    FragDef { name: "falloff_sharpen", src: include_str!("../shaders/sharpening/falloff_sharpen.glsl") },
    FragDef { name: "power_curve_sharpen", src: include_str!("../shaders/sharpening/power_curve_sharpen.glsl") },
    FragDef { name: "unsharp_mask", src: include_str!("../shaders/sharpening/unsharp_mask.glsl") },
    FragDef { name: "local_contrast", src: include_str!("../shaders/local_contrast/local_contrast.glsl") },
    FragDef { name: "gaussian_blur", src: include_str!("../shaders/blur/gaussian_blur.glsl") },
    FragDef { name: "box_blur", src: include_str!("../shaders/blur/box_blur.glsl") },
    FragDef { name: "bokeh_blur", src: include_str!("../shaders/blur/bokeh_blur.glsl") },
    FragDef { name: "tilt_shift_blur", src: include_str!("../shaders/blur/tilt_shift_blur.glsl") },
    FragDef { name: "radial_blur", src: include_str!("../shaders/blur/radial_blur.glsl") },
    FragDef { name: "gradient_deband", src: include_str!("../shaders/image_quality/gradient_deband.glsl") },
    FragDef { name: "threshold_bloom", src: include_str!("../shaders/image_quality/threshold_bloom.glsl") },
    FragDef { name: "ghost_flare", src: include_str!("../shaders/image_quality/ghost_flare.glsl") },
];

pub(crate) const SPATIAL_TAIL_FRAGS: [FragDef; 2] = [
    FragDef { name: "red_halation", src: include_str!("../shaders/inline/red_halation.glsl") },
    FragDef { name: "anamorphic_streak", src: include_str!("../shaders/inline/anamorphic_streak.glsl") },
];

pub(crate) const GRADE_FRAGS: [FragDef; 46] = [
    FragDef { name: "linear_exposure", src: include_str!("../shaders/exposure/linear_exposure.glsl") },
    FragDef { name: "aces_tonemap", src: include_str!("../shaders/tonemapping/aces_tonemap.glsl") },
    FragDef { name: "agx_tonemap", src: include_str!("../shaders/tonemapping/agx_tonemap.glsl") },
    FragDef { name: "reinhard_tonemap", src: include_str!("../shaders/tonemapping/reinhard_tonemap.glsl") },
    FragDef { name: "hable_tonemap", src: include_str!("../shaders/tonemapping/hable_tonemap.glsl") },
    FragDef { name: "lottes_tonemap", src: include_str!("../shaders/tonemapping/lottes_tonemap.glsl") },
    FragDef { name: "uchimura_tonemap", src: include_str!("../shaders/tonemapping/uchimura_tonemap.glsl") },
    FragDef { name: "tony_tonemap", src: include_str!("../shaders/tonemapping/tony_tonemap.glsl") },
    FragDef { name: "khronos_tonemap", src: include_str!("../shaders/tonemapping/khronos_tonemap.glsl") },
    FragDef { name: "neutral_white_balance", src: include_str!("../shaders/white_balance/neutral_white_balance.glsl") },
    FragDef { name: "warm_temperature", src: include_str!("../shaders/white_balance/warm_temperature.glsl") },
    FragDef { name: "cool_temperature", src: include_str!("../shaders/white_balance/cool_temperature.glsl") },
    FragDef { name: "saturation_contrast_grade", src: include_str!("../shaders/color_grading/saturation_contrast_grade.glsl") },
    FragDef { name: "levels_remap", src: include_str!("../shaders/color_grading/levels_remap.glsl") },
    FragDef { name: "gamma_correct", src: include_str!("../shaders/color_grading/gamma_correct.glsl") },
    FragDef { name: "vibrance_boost", src: include_str!("../shaders/color_grading/vibrance_boost.glsl") },
    FragDef { name: "hsl_transform", src: include_str!("../shaders/color_grading/hsl_transform.glsl") },
    FragDef { name: "split_tone", src: include_str!("../shaders/color_grading/split_tone.glsl") },
    FragDef { name: "lift_gamma_gain", src: include_str!("../shaders/color_grading/lift_gamma_gain.glsl") },
    FragDef { name: "hermite_curves", src: include_str!("../shaders/color_grading/hermite_curves.glsl") },
    FragDef { name: "red_channel_curve", src: include_str!("../shaders/channel_curves/red_channel_curve.glsl") },
    FragDef { name: "green_channel_curve", src: include_str!("../shaders/channel_curves/green_channel_curve.glsl") },
    FragDef { name: "blue_channel_curve", src: include_str!("../shaders/channel_curves/blue_channel_curve.glsl") },
    FragDef { name: "trizone_color_balance", src: include_str!("../shaders/color_balance/trizone_color_balance.glsl") },
    FragDef { name: "red_selective_saturate", src: include_str!("../shaders/selective_color/red_selective_saturate.glsl") },
    FragDef { name: "green_selective_saturate", src: include_str!("../shaders/selective_color/green_selective_saturate.glsl") },
    FragDef { name: "blue_selective_saturate", src: include_str!("../shaders/selective_color/blue_selective_saturate.glsl") },
    FragDef { name: "dynamic_range_crush", src: include_str!("../shaders/stylization/dynamic_range_crush.glsl") },
    FragDef { name: "duotone_map", src: include_str!("../shaders/stylization/duotone_map.glsl") },
    FragDef { name: "color_wash_tint", src: include_str!("../shaders/stylization/color_wash_tint.glsl") },
    FragDef { name: "posterize_quantize", src: include_str!("../shaders/stylization/posterize_quantize.glsl") },
    FragDef { name: "bleach_bypass", src: include_str!("../shaders/stylization/bleach_bypass.glsl") },
    FragDef { name: "technicolor_process", src: include_str!("../shaders/stylization/technicolor_process.glsl") },
    FragDef { name: "midpoint_contrast", src: include_str!("../shaders/stylization/midpoint_contrast.glsl") },
    FragDef { name: "color_invert", src: include_str!("../shaders/stylization/color_invert.glsl") },
    FragDef { name: "luminance_grayscale", src: include_str!("../shaders/stylization/luminance_grayscale.glsl") },
    FragDef { name: "cel_shade", src: include_str!("../shaders/toon/cel_shade.glsl") },
    FragDef { name: "manga_screentone", src: include_str!("../shaders/toon/manga_screentone.glsl") },
    FragDef { name: "crosshatch_shade", src: include_str!("../shaders/toon/crosshatch_shade.glsl") },
    FragDef { name: "protanopia_simulation", src: include_str!("../shaders/accessibility/protanopia_simulation.glsl") },
    FragDef { name: "deuteranopia_simulation", src: include_str!("../shaders/accessibility/deuteranopia_simulation.glsl") },
    FragDef { name: "tritanopia_simulation", src: include_str!("../shaders/accessibility/tritanopia_simulation.glsl") },
    FragDef { name: "protanopia_correct", src: include_str!("../shaders/accessibility/protanopia_correct.glsl") },
    FragDef { name: "deuteranopia_correct", src: include_str!("../shaders/accessibility/deuteranopia_correct.glsl") },
    FragDef { name: "tritanopia_correct", src: include_str!("../shaders/accessibility/tritanopia_correct.glsl") },
    FragDef { name: "radial_vignette", src: include_str!("../shaders/inline/radial_vignette.glsl") },
];

pub(crate) const GRADE_FRAME_FRAGS: [FragDef; 2] = [
    FragDef { name: "cinematic_letterbox", src: include_str!("../shaders/inline/cinematic_letterbox.glsl") },
    FragDef { name: "ordered_dither", src: include_str!("../shaders/inline/ordered_dither.glsl") },
];

pub(crate) const GRADE_HARDWARE_FRAGS: [FragDef; 17] = [
    FragDef { name: "ps1_simulation", src: include_str!("../shaders/hardware_simulation/ps1_color.glsl") },
    FragDef { name: "saturn_simulation", src: include_str!("../shaders/hardware_simulation/saturn_color.glsl") },
    FragDef { name: "n64_simulation", src: include_str!("../shaders/hardware_simulation/n64_color.glsl") },
    FragDef { name: "dreamcast_simulation", src: include_str!("../shaders/hardware_simulation/dreamcast_color.glsl") },
    FragDef { name: "ps2_simulation", src: include_str!("../shaders/hardware_simulation/ps2_color.glsl") },
    FragDef { name: "xbox_simulation", src: include_str!("../shaders/hardware_simulation/xbox_color.glsl") },
    FragDef { name: "psp_simulation", src: include_str!("../shaders/hardware_simulation/psp_color.glsl") },
    FragDef { name: "ps3_simulation", src: include_str!("../shaders/hardware_simulation/ps3_color.glsl") },
    FragDef { name: "xbox360_simulation", src: include_str!("../shaders/hardware_simulation/xbox360_color.glsl") },
    FragDef { name: "crt_simulation", src: include_str!("../shaders/hardware_simulation/crt_color.glsl") },
    FragDef { name: "phosphor_amber", src: include_str!("../shaders/hardware_simulation/phosphor_amber.glsl") },
    FragDef { name: "phosphor_green", src: include_str!("../shaders/hardware_simulation/phosphor_green.glsl") },
    FragDef { name: "phosphor_red", src: include_str!("../shaders/hardware_simulation/phosphor_red.glsl") },
    FragDef { name: "scanline_darken", src: include_str!("../shaders/hardware_simulation/scanline_darken.glsl") },
    FragDef { name: "oled_simulation", src: include_str!("../shaders/hardware_simulation/oled_color.glsl") },
    FragDef { name: "vhs_simulation", src: include_str!("../shaders/hardware_simulation/vhs_color.glsl") },
    FragDef { name: "lcd_subpixel", src: include_str!("../shaders/hardware_simulation/lcd_subpixel.glsl") },
];

pub(crate) const PRE_TEMPORAL_FRAGS: [FragDef; 1] = [
    FragDef { name: "ink_outline", src: include_str!("../shaders/toon/ink_outline.glsl") },
];

pub(crate) const TEMPORAL_FRAGS: [FragDef; 22] = [
    FragDef { name: "neighborhood_clamp_aa", src: include_str!("../shaders/temporal/neighborhood_clamp_aa.glsl") },
    FragDef { name: "motion_reject_denoise", src: include_str!("../shaders/temporal/motion_reject_denoise.glsl") },
    FragDef { name: "motion_detect_blur", src: include_str!("../shaders/temporal/motion_detect_blur.glsl") },
    FragDef { name: "constant_blend_smooth", src: include_str!("../shaders/temporal/constant_blend_smooth.glsl") },
    FragDef { name: "shutter_angle_smooth", src: include_str!("../shaders/temporal/shutter_angle_smooth.glsl") },
    FragDef { name: "spline_interp_smooth", src: include_str!("../shaders/temporal/spline_interp_smooth.glsl") },
    FragDef { name: "variance_decay_smooth", src: include_str!("../shaders/temporal/variance_decay_smooth.glsl") },
    FragDef { name: "dualrate_smooth", src: include_str!("../shaders/temporal/dualrate_smooth.glsl") },
    FragDef { name: "luminance_gate_smooth", src: include_str!("../shaders/temporal/luminance_gate_smooth.glsl") },
    FragDef { name: "contrast_gate_smooth", src: include_str!("../shaders/temporal/contrast_gate_smooth.glsl") },
    FragDef { name: "gradient_gate_smooth", src: include_str!("../shaders/temporal/gradient_gate_smooth.glsl") },
    FragDef { name: "sigma_clip_smooth", src: include_str!("../shaders/temporal/sigma_clip_smooth.glsl") },
    FragDef { name: "mitchell_kernel_smooth", src: include_str!("../shaders/temporal/mitchell_kernel_smooth.glsl") },
    FragDef { name: "ycocg_clip_smooth", src: include_str!("../shaders/temporal/ycocg_clip_smooth.glsl") },
    FragDef { name: "bilateral_history_smooth", src: include_str!("../shaders/temporal/bilateral_history_smooth.glsl") },
    FragDef { name: "perceptual_chroma_smooth", src: include_str!("../shaders/temporal/perceptual_chroma_smooth.glsl") },
    FragDef { name: "frequency_split_smooth", src: include_str!("../shaders/temporal/frequency_split_smooth.glsl") },
    FragDef { name: "horn_schunck_smooth", src: include_str!("../shaders/temporal/horn_schunck_smooth.glsl") },
    FragDef { name: "convergent_accumulate", src: include_str!("../shaders/temporal/convergent_accumulate.glsl") },
    FragDef { name: "dualwarp_flow_smooth", src: include_str!("../shaders/temporal/dualwarp_flow_smooth.glsl") },
    FragDef { name: "variance_flow_accumulate", src: include_str!("../shaders/temporal/variance_flow_accumulate.glsl") },
    FragDef { name: "edge_reconstruct_smooth", src: include_str!("../shaders/temporal/edge_reconstruct_smooth.glsl") },
];

pub(crate) const NOISE_FRAGS: [FragDef; 3] = [
    FragDef { name: "gaussian_grain", src: include_str!("../shaders/inline/gaussian_grain.glsl") },
    FragDef { name: "crt_simulation", src: include_str!("../shaders/hardware_simulation/crt_pulse.glsl") },
    FragDef { name: "vhs_simulation", src: include_str!("../shaders/hardware_simulation/vhs_noise.glsl") },
];

pub(crate) const OVERLAY_FRAGS: [FragDef; 2] = [
    FragDef { name: "fps_hud", src: include_str!("../shaders/overlay/fps_hud.glsl") },
    FragDef { name: "crosshair_overlay", src: include_str!("../shaders/overlay/crosshair_overlay.glsl") },
];
