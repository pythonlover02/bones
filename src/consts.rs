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
pub(crate) const POLL_BLOCK: i32 = -1;

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

pub(crate) const HEAD: &str = r#"##bones default profile
# effects process in this order: geometric warps -> master fetch -> shared
# neighborhood taps -> spatial/blur/temporal -> color grade and tonemap ->
# inline lens/film -> hardware sim color half -> HUD overlay.
# pick ONE anti aliasing ONE tonemapper and ONE primary temporal smoother for
# best result, but the source never rejects stacked effects: stack whatever
# look you find good.

[general]
# resolution_scale to scale the post-fx render target relative to the swap
# chain. 1.0 = render at native resolution. 0.5 = render the entire
# post-fx pipeline at half resolution then bilinear upscale to native at
# the final blit. lower value gives big perf wins for expensive effect
# stacks at the cost of softer image. minimum 0.05. hot-reloadable:
# changing it while the game runs triggers a full post-fx resource
# rebuild on the next present, no swapchain recreation needed.
# env: BONES_RESOLUTION_SCALE
resolution_scale = 1.0

# optimize_dynamic_rendering to enable VK_KHR_dynamic_rendering at device
# creation when supported. logged at device creation when not
# available. on by default. env: BONES_OPTIMIZE_DYNAMIC_RENDERING
optimize_dynamic_rendering = true

# optimize_push_descriptors to enable VK_KHR_push_descriptor at device
# creation when supported. logged when not available. on by default.
# env: BONES_OPTIMIZE_PUSH_DESCRIPTORS
optimize_push_descriptors = true

# optimize_sync2 to enable VK_KHR_synchronization2 at device creation
# when supported. enables batched pipeline barriers via
# vkCmdPipelineBarrier2 with 64-bit stage masks. reduces per-frame CPU
# barrier overhead by collapsing the post-pass and end-of-frame
# barrier pairs into single calls. logged when not available. on by
# default. env: BONES_OPTIMIZE_SYNC2
optimize_sync2 = true

# optimize_mutable_format to enable VK_KHR_swapchain_mutable_format at
# device creation when supported. lets the layer view sRGB swap image
# as their UNORM equivalent so the shader sample the swap image
# directly with no input copy. device-level: cannot be hot-reloaded,
# require a game restart. turn off if you see gamma shift or
# corruption on a specific driver. logged when not available. on by
# default. env: BONES_OPTIMIZE_MUTABLE_FORMAT
optimize_mutable_format = true

# optimize_async_compute to submit post-fx work on a dedicated async
# compute queue family when one is available. lets the post-fx
# dispatch run concurrently with the game next-frame rendering
# instead of serially before present. requires the compute path
# (compute = true), a queue family with COMPUTE but not GRAPHICS bit
# that the application did not request, and swapchain creation that
# permits CONCURRENT sharing across the involved families. silently
# falls back to graphics-queue submission when any of these conditions
# is not met. on by default. env: BONES_OPTIMIZE_ASYNC_COMPUTE
optimize_async_compute = true

# compute to use a compute shader instead of a fragment shader for the
# ubershader pass. compute wins on most modern GPUs by skipping the
# rasterizer and exploiting shared memory/subgroup paths. requires
# the device feature shaderStorageImageWriteWithoutFormat and a swap
# format that supports VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT. if either
# is missing a log line is emitted and the layer falls back to the
# fragment shader path for that swapchain. on by default.
# env: BONES_COMPUTE
compute = true

# compute_x and compute_y to the workgroup size on each axis for the
# compute shader. 8x8 is the safe default and is what we use unless
# you know your hardware prefers something else (some AMD GPUs win
# with 16x16; some NVIDIA GPUs prefer 32x4 for memory-coalesced
# loads). product must not exceed the device max compute work group
# invocations or we fall back to 8x8 with a log line. each axis is
# clamped to the device max work group size on that axis.
# env: BONES_COMPUTE_X, BONES_COMPUTE_Y
compute_x = 8
compute_y = 8

[geometric]
# warp texture coordinate before any pixel be sampled. all warps here are
# deterministic (no time, no random) so temporal history align frame to
# frame across resolution.

identity = false

# mirror_horizontal to flip image left right around vertical axis
mirror_horizontal = false

# mirror_vertical to flip image top bottom around horizontal axis
mirror_vertical = false

# rotate_90 to rotate 90° clockwise
rotate_90 = false

# rotate_180 to rotate 180°
rotate_180 = false

# rotate_270 to rotate 270° clockwise (90° counter clockwise)
rotate_270 = false

# center_zoom to 1.5x magnification from screen center
center_zoom = false

# polynomial_distort to radial lens distortion using the Brown Conrady k1/k2
#   polynomial model (the same distortion model used in OpenCV camera
#   calibration). simulate cheap wide angle lens curvature.
polynomial_distort = false

# barrel_undistort to inverse barrel correction to straighten curved line
#   the opposite of polynomial_distort. useful for undoing lens warp on
#   emulated title or ultrawide display.
barrel_undistort = false

# fisheye_warp to wide angle fisheye projection via atan remapping. produce
#   the extreme curvature of a real fisheye lens rather than the mild barrel
#   distortion of polynomial_distort.
fisheye_warp = false

# trapezoid_warp to perspective keystoning make the image wider at the
#   bottom than the top. simulate looking at a screen from below.
trapezoid_warp = false

# sharp_bilinear to pixel art sharpener using Hermite smoothstep UV warp.
#   snap texture sampling toward pixel center so the GPU builtin
#   linear filter produce crisp pixel boundarie without nearest neighbor
#   shimmer. essential for retro/pixel art game running at nonnative
#   resolution. this must run in the UV stage because it work by changing
#   WHERE the texture be sampled not what happen after.
sharp_bilinear = false

[denoise]
# run first in the spatial chain to clean noise before other processing

# bilateral_denoise to bilateral filter the standard noise reduction
#   technique from Tomasi & Manduchi (1998). smooth area where
#   neighboring pixel be similar in color (noise) while preserving
#   area where neighbor differ (edge). the same core algorithm
#   used in Photoshop "Surface Blur" and most camera noise reduction.
bilateral_denoise = false

[anti_aliasing]
# smooth jagged edge (stairstepping). run after denoise before sharpening.
# if your game already have builtin AA you probably do not need these.
# pick ONE to stacking multiple AA method waste performance for no benefit.

# luma_edge_aa to this be FXAA (Fast Approximate Anti Aliasing) by Timothy
#   Lottes at NVIDIA. the most widely used postprocess AA in gaming history.
#   detect edge by computing luminance gradient across a 3x3 neighborhood
#   then blur along the detected edge direction with clamped span. fast
#   effective slightly soften the image. if you have played any game from
#   2011 present with "FXAA" in the setting menu this be that algorithm.
luma_edge_aa = false

# normal_filter_aa to normal filter AA (NFAA). compute the gradient
#   perpendicular to each edge then sample along the tangent direction
#   to blend away stairstepping. lighter weight than FXAA slightly less
#   effective on diagonal edge. good for low power hardware.
normal_filter_aa = false

# morphological_aa to conservative morphological AA inspired by Intel
#   CMAA technique. blend cardinal and diagonal neighbor weighted by
#   absolute edge magnitude (sum of per channel difference) with a soft
#   threshold. more conservative than FXAA to preserve more detail but
#   catch fewer edge.
morphological_aa = false

# subpixel_aa to subpixel morphological AA inspired by SMAA (Enhanced
#   Subpixel Morphological Antialiasing by Jimenez et al. 2012). detect
#   whether edge be primarily horizontal or vertical then select the
#   matching directional blend. SMAA be considered the gold standard of
#   post process AA in modern game to this be a single pass approximation
#   of it edge detection and blending logic.
subpixel_aa = false

[sharpening]
# enhance detail and crispness. run after AA.
# these can be combined (e.g. one sharpener + midtone_clarity) but stacking
# multiple strong sharpener will produce halo and ringing artifact.

# contrast_adaptive_sharpen to this be CAS (Contrast Adaptive Sharpening)
#   from AMD FidelityFX suite (open source GPUOpen). the same sharpener
#   available in the AMD Radeon driver overlay and used as the final stage
#   in AMD FSR 1.0 upscaling pipeline. work by computing the ratio of
#   local min to max so it sharpen detail in low contrast area (where
#   you need it) without overshooting in high contrast area (where halo
#   would appear).
contrast_adaptive_sharpen = false

# robust_contrast_sharpen to this be RCAS (Robust Contrast Adaptive
#   Sharpening) from AMD FSR 1.0 (open source GPUOpen). a simplified
#   variant of CAS that clamp the sharpening result to the neighborhood
#   min/max making it impossible to produce ringing artifact. slightly
#   less aggressive than CAS but completely halo free.
robust_contrast_sharpen = false

# edge_directed_sharpen to edge aware sharpening based on NVIDIA NIS
#   (NVIDIA Image Scaling open source). measure gradient magnitude to
#   determine how much each pixel be on an edge then scale sharpening
#   proportionally so edge get full sharpening and flat area (texture
#   skin) get minimal sharpening. concentrate detail enhancement where
#   structure exist rather than amplifying noise in smooth region.
edge_directed_sharpen = false

# laplacian_sharpen to the simplest possible sharpener: center pixel minus
#   the average of it four neighbor scaled by strength. this be the
#   "Sharpen" filter in every image editor. effective but produce halo
#   on high contrast edge. use robust_contrast_sharpen or
#   contrast_adaptive_sharpen instead if halo bother you.
laplacian_sharpen = false

# luminance_sharpen to sharpen only the luminance (brightness) channel
#   leaving color untouched. prevent the color fringing artifact that
#   RGB sharpener can produce on saturated edge. the same principle
#   used by Lightroom sharpening which operate in Lab color space.
luminance_sharpen = false

# midtone_clarity to midtone local contrast enhancement the same concept
#   as Photoshop/Lightroom "Clarity" slider. boost contrast in midtone
#   (the middle brightness range) without clipping highlight or crushing
#   shadow. excellent for making flat looking game feel more punchy.
midtone_clarity = false

# falloff_sharpen to adaptive sharpen where strength be inversely
#   proportional to local edge energy. flat area with fine texture get
#   full sharpening; strong edge get almost none. reduce halo compared
#   to laplacian_sharpen while being more aggressive on fine detail.
falloff_sharpen = false

# power_curve_sharpen to filmic sharpen using a rational response curve
#   instead of linear scaling. small difference get boosted more than
#   large difference producing a softer more cinematic look than
#   linear sharpener. good for story driven game where you want
#   enhanced detail without a "processed" look.
power_curve_sharpen = false

# unsharp_mask to the traditional darkroom/Photoshop technique: subtract
#   a blurred version of the image from the original then add the
#   difference back scaled by an amount. use a weighted 3x3 kernel
#   (center + cardinal + diagonal weight). the classic photographic
#   sharpening method.
unsharp_mask = false

[local_contrast]
# local_contrast to per pixel contrast enhancement based on the ratio of
#   center luminance to neighborhood average. make each pixel "pop"
#   relative to it surrounding. similar to the local contrast
#   enhancement in DaVinci Resolve color grading tool.
local_contrast = false

[blur]
# various blur effect. can be used creatively (tilt shift for miniature
# look radial blur for speed effect) or practically (soften oversharpened
# game output).

# gaussian_blur to weighted 3x3 blur with Gaussian approximating weight
#   (center 4 cardinal 2 diagonal 1). the standard "Gaussian Blur" from
#   every image editor just at a small fixed kernel size.
gaussian_blur = false

# box_blur to large radius box blur via Kawase dual filter pattern. four
#   bilinear paired sample at half pixel offset exploit GPU bilinear
#   filtering to cover a wide neighborhood at minimum fetch cost. produce
#   the same broad uniform blur as a multi tap box kernel for a fraction
#   of the bandwidth.
box_blur = false

# bokeh_blur to circular ring blur simulating shallow depth of field (the
#   blurry background you get with a wide aperture camera lens). four
#   bilinear paired sample around the unit circle with brightness
#   weighting reproduce the characteristic "bokeh circle" of out of focus
#   highlight at lower fetch cost than naive ring sampling.
bokeh_blur = false

# tilt_shift_blur to miniature/diorama effect. a sharp horizontal band
#   across the middle of the screen with progressive blur toward the top
#   and bottom edge. make real scene look like tiny scale model.
#   the same effect Instagram popularized in 2010.
tilt_shift_blur = false

# radial_blur to zoom blur radiating from screen center. simulate the
#   effect of zooming a camera lens during exposure or a "speed line"
#   feeling. useful for racing game or dramatic screenshot.
radial_blur = false

[image_quality]
# gradient_deband to remove visible banding in color gradient (the
#   stairstepping you see in dark sky or smooth surface). sample a
#   random neighbor within range and if the color difference be below
#   threshold average them. the same technique used by mpv video player
#   debanding filter. essential for game with 8 bit output and dark scene.
gradient_deband = false

# threshold_bloom to bright pixel glow. find pixel above a brightness
#   threshold (0.7) accumulate their excess brightness from the immediate
#   neighbor and add it back as a local glow. a single pass small radius
#   approximation of the multi pass bloom used in game engine. make
#   bright light explosion and reflection "glow" at close range.
threshold_bloom = false

# ghost_flare to lens flare ghost simulation. sample bright spot at
#   increasing distance along the vector from screen center to each pixel
#   creating the circular "ghost" reflection you see when pointing a camera
#   at a bright light. similar to the lens flare effect in JJ Abrams film.
ghost_flare = false

[temporal]
# blend current frame with previous frame to reduce flicker noise and judder.
# this be bones signature feature to 22 temporal processing mode from simple
# blending to motion compensated accumulation inspired by the research behind
# DLSS FSR and XeSS.
#
# pick ONE primary mode for best result. stacking multiple temporal effect
# cause compounding ghosting which may or may not be the look you want.
# the convergent_detail_recovery stabilizer activate automatically
# whenever any temporal mode be enabled. you get it for free.
#
# ordered from simplest to most sophisticated:

# neighborhood_clamp_aa to temporal anti aliasing (TAA). the standard
#   technique used in nearly every modern game engine (Unreal Unity
#   Frostbite). clamp the previous frame pixel to the current frame
#   3x3 neighborhood min/max box before blending which prevent the
#   worst ghosting artifact. if a game have "TAA" in it setting it
#   doing this (plus jitter which we cannot add from outside).
neighborhood_clamp_aa = false

# motion_reject_denoise to motion gated accumulation. heavily blend
#   with the previous frame on pixel that have not changed (reducing
#   noise and flicker) but instantly snap to the current frame when
#   motion be detected. good for static scene with noisy rendering.
motion_reject_denoise = false

# motion_detect_blur to motion blur via temporal blending. blend toward
#   the previous frame proportional to how much each pixel changed so
#   still area stay sharp and moving area get motion trail. simulate
#   camera motion blur without engine support.
motion_detect_blur = false

# constant_blend_smooth to the simplest possible temporal filter: 50/50
#   mix between current and previous frame. strong flicker reduction but
#   visible ghosting on any motion. useful as a baseline to compare
#   against more sophisticated mode.
constant_blend_smooth = false

# shutter_angle_smooth to physical 180° shutter simulation. weight the
#   history contribution by motion magnitude so still pixel integrate
#   over the full half frame exposure while moving pixel get progressive
#   shorter virtual exposure. produce the cinematic motion blur of real
#   24fps film camera where stillness be fully integrated and motion
#   smear be partial. different from constant_blend_smooth which apply
#   afixed 50/50 blend regardless of motion.
shutter_angle_smooth = false

# spline_interp_smooth to cubic Hermite reconstruction between the previous
#   and current frame with extrapolated control point. the Hermite
#   formulation overshoot slightly past linear blending producing a sharper
#   temporal response on detail recovery than the constant blend equivalent.
#   related to the Catmull Rom family of cubic interpolant used in
#   animation software and video editing.
spline_interp_smooth = false

# variance_decay_smooth to exponential IIR filter with variance based
#   decay. when a pixel change a lot between frame blend weight drop
#   instantly; when stable weight ramp up over time. the temporal
#   equivalent of a noise gate in audio to pass transient through
#   while smoothing the steady state signal.
variance_decay_smooth = false

# dualrate_smooth to dual rate adaptive filter. use two blend weight:
#   high (0.7) for still area low (0.1) for area with spatial variance.
#   the crossover be controlled by the variance of the current 5 tap
#   neighborhood. similar in concept to how adaptive bitrate streaming
#   adjust quality based on network condition.
dualrate_smooth = false

# luminance_gate_smooth to luminance adaptive blending. smooth dark
#   region aggressively (0.7 blend) and barely touch bright area
#   (0.15 blend). exploit a property of human vision: scotopic (low light)
#   vision have much worse temporal resolution so you cannot see flicker
#   as well in dark. this mean heavy dark area smoothing be "free" to
#   you get the stability benefit without noticing the ghosting.
luminance_gate_smooth = false

# contrast_gate_smooth to contrast adaptive blending. heavy smoothing on
#   low contrast region (sky fog gradient wall) where shimmer and
#   noise be most visible light smoothing on high contrast textured
#   area where detail matter. different from gradient_gate_smooth because
#   it respond to contrast magnitude not edge direction.
contrast_gate_smooth = false

# gradient_gate_smooth to edge aware temporal blending. compute the
#   spatial gradient (Sobel like) at each pixel; strong gradient = edge.
#   edge get zero temporal blend (eliminating ghosting at object
#   boundarie) flat region get full blend (maximizing smoothness).
#   the most effective single technique for eliminating edge ghosting.
gradient_gate_smooth = false

# sigma_clip_smooth to variance clipping. compute the per pixel mean
#   and standard deviation across the 3x3 neighborhood then clip the
#   history sample to mean ± sigma before blending. this be the core
#   technique of modern TAA (described by Salvi 2016) and the foundation
#   that all advanced temporal filter build on. tighter than simple
#   min/max clamping because sigma adapt to local noise level.
sigma_clip_smooth = false

# mitchell_kernel_smooth to Mitchell Netravali piecewise cubic spatial
#   reconstruction (Mitchell & Netravali 1988) with B=1/3 C=1/3 the
#   standard "balanced" tradeoff between sharpness and ringing used in
#   Pixar RenderMan Blender and ImageMagick high quality resampling.
#   apply the cubic kernel to the 3x3 neighborhood for spatial
#   reconstruction then blend the result with temporal history.
mitchell_kernel_smooth = false

# ycocg_clip_smooth to YCoCg neighborhood AABB clamp with confidence
#   weighting. convert to YCoCg color space (a decorrelated luma/chroma
#   representation) build a bounding box from cardinal tap clamp
#   history to that box then blend proportional to how much clamping
#   was needed. this be the same technique used in AMD open source
#   GPUOpen temporal stabilization code to the core of FSR 2 temporal
#   accumulation approach.
ycocg_clip_smooth = false

# bilateral_history_smooth to bilateral temporal filter. compare the
#   5 tap neighborhood average of the current frame to the 5 tap
#   neighborhood average of the history frame; if the spatial structure
#   match (low difference) the history be trustworthy. apply the
#   bilateral principle of Tomasi & Manduchi (1998) in the temporal
#   domain rather than the spatial domain.
bilateral_history_smooth = false

# perceptual_chroma_smooth to convert to YCbCr (the color space used
#   in JPEG and video compression) then blend luma and chroma channel
#   separately: luma at 0.4 chroma at 0.7. exploit the fact that human
#   vision have much lower spatial and temporal resolution for color than
#   for brightness to you literally cannot see chroma flicker as well as
#   luma flicker. this mean aggressive chroma smoothing be perceptually
#   free. the same principle that let JPEG compress color channel 2x
#   more than brightness (4:2:0 chroma subsampling).
perceptual_chroma_smooth = false

# frequency_split_smooth to split current and history frame into low
#   and high frequency band using a 5 tap box decomposition. low
#   frequencie (broad lighting change sky gradient) be blended
#   aggressively (0.7) high frequencie (edge texture detail) be
#   blended lightly (0.2). preserve sharpness while stabilizing the
#   large scale luminance fluctuation that cause visible flicker.
#   similar concept to wavelet based denoising used in film production.
frequency_split_smooth = false

# horn_schunck_smooth to optical flow temporal smoothing using the
#   Horn Schunck algorithm (Horn & Schunck 1981). estimate per pixel
#   motion by solving the brightness constancy equation:
#   Ix*u + Iy*v + It = 0 (spatial gradient time velocity equal
#   temporal gradient). warp the history frame along the estimated
#   flow field before blending. this be the foundational optical flow
#   algorithm to simple enough for a single shader pass effective enough
#   to meaningfully reduce ghosting on linear motion.
horn_schunck_smooth = false

# convergent_accumulate to convergent temporal supersampling. blend weight
#   ramp from 0.1 to 0.85 as pixel stability increase over time (using
#   a squared stability curve for gradual convergence). in stable scene
#   natural game camera jitter mean consecutive frame sample slightly
#   different sub pixel position to accumulating these build up detail
#   over time. inspired by the principle behind DLSS 2 temporal
#   accumulation: build up detail from multiple slightly offset sample.
#   without engine provided jitter it less precise but natural camera
#   and object micro motion provide enough variation for visible detail
#   improvement after 5 10 stable frame.
convergent_accumulate = false

# dualwarp_flow_smooth to dual warp frame interpolation with flow
#   consistency checking. inspired by AMD open source GPUOpen frame
#   interpolation pipeline (the algorithm at the core of FSR 3 frame
#   generation). estimate optical flow then warp BOTH the current
#   frame backward and the history frame forward by half the motion
#   vector toward a temporal midpoint to this halve the warp error
#   compared to single direction warping. perform a forward backward
#   consistency check using gradient resampled at the warped location
#   (estimate flow A to B then B to A; if they do not cancel the flow
#   be unreliable). clamp warped history to the cardinal neighborhood
#   AABB. fall back to the current frame on disocclusion.
dualwarp_flow_smooth = false

# variance_flow_accumulate to motion compensated temporal accumulation
#   with variance clipping and triple confidence gating. based on the
#   temporal supersampling paper by Brian Karis (SIGGRAPH 2014 "High
#   Quality Temporal Supersampling") and Jorge Jimenez (GDC 2016
#   "Filmic SMAA") to these paper were the foundational research that
#   directly inspired NVIDIA DLSS 2.0 temporal accumulation approach.
#   estimate optical flow to warp history clip the warped result to
#   mean ± 1.25σ of the current 5 tap neighborhood (tighter than AABB
#   clamping) then gate the blend with three independent confidence
#   factor: motion magnitude luminance consistency and clip distance.
#   exponential convergence (squared confidence) mean it take several
#   stable frame to reach full accumulation weight producing the
#   "detail build up over time" quality DLSS be known for.
variance_flow_accumulate = false

# edge_reconstruct_smooth to edge directed temporal reconstruction.
#   inspired by Intel open source XeSS DP4a fallback path (the non
#   neural network codepath of XeSS that run on any GPU). the distinctive
#   feature be edge direction detection: compute the local gradient via
#   Sobel then sample ALONG the edge (not across it) for directional
#   spatial reconstruction before temporal blending. this mean on a
#   near horizontal edge it blend pixel left right along the edge
#   for spatial quality then motion compensate the history clamp to
#   the neighborhood AABB and blend with edge aware confidence.
#   no other temporal mode do directional spatial reconstruction.
edge_reconstruct_smooth = false

[exposure]
# linear_exposure to fixed 1.3x exposure multiplier applied before
#   tonemapping. multiply all pixel value by a constant like adjusting
#   the ISO on a camera. on/off only.
linear_exposure = false

[tonemapping]
# map high dynamic range value into displayable [0 1]. essential if
# the game output value above 1.0 (HDR rendering). each curve have a
# different "look" to pick ONE based on preference.

# aces_tonemap to the Academy Color Encoding System filmic curve. the
#   exact Narkowicz (2015) algebraic fit used in countless game and film
#   VFX pipeline. the ACES curve be the motion picture industry standard
#   for tone mapping. characteristic warm highlight and slightly crushed
#   shadow. coefficient: A=2.51 B=0.03 C=2.43 D=0.59 E=0.14.
aces_tonemap = false

# agx_tonemap to AgX sigmoid curve. a newer alternative to ACES that
#   avoid the oversaturation and hue shift ACES produce in bright
#   highlight. operate in log2 domain with a cubic polynomial. used
#   in Blender new default color management. more neutral than ACES.
agx_tonemap = false

# reinhard_tonemap to Reinhard extended with white point (Reinhard et al.
#   2002). the first practical tone mapping operator still popular for
#   it natural gentle highlight rolloff. white point constant 4.0
#   (enters squared per the extended Reinhard form).
reinhard_tonemap = false

# hable_tonemap to the Uncharted 2 filmic curve by John Hable (GDC 2010).
#   designed specifically for video game with a strong shoulder that
#   preserve highlight detail and rich midtone contrast. the "cinematic
#   game look" of the early 2010. use the standard constant:
#   A=0.15 B=0.5 C=0.1 D=0.2 E=0.02 F=0.3 W=11.2.
hable_tonemap = false

# lottes_tonemap to Timothy Lottes curve (Lottes 2016). designed as a
#   more tunable alternative to Reinhard with separate contrast and
#   shoulder control. Lottes also create FXAA to this be the tone
#   mapping equivalent of that same "simple but effective" philosophy.
lottes_tonemap = false

# uchimura_tonemap to the Gran Turismo curve by Hajime Uchimura
#   (CEDEC 2017). designed for racing game where you need smooth
#   handling of extremely bright specular highlight (sun reflection
#   on car paint). smooth toe controlled linear section and shoulder.
uchimura_tonemap = false

# tony_tonemap to Tony McMapface. simple offset scale highlight rolloff.
#   the least complex tonemap to just divide by (color + offset) and
#   rescale. useful as a lightweight baseline.
tony_tonemap = false

# khronos_tonemap to Khronos PBR Neutral tone mapping from the Khronos
#   Group PBR specification. soft highlight compression starting at
#   peak luminance 0.76. designed for physically based rendering pipeline
#   where color accuracy matter more than dramatic curve.
khronos_tonemap = false

[white_balance]
# neutral_white_balance to D65 daylight chromatic adaptation. gently
#   correct typical display warm bias toward the CIE D65 reference
#   white point used in sRGB and Rec.709 standard. a subtle correction
#   that slightly reduce red warmth and boost blue to reach neutral
#   reference white.
neutral_white_balance = false

# warm_temperature to warm color shift: red boosted 1.05x blue cut to
#   0.92x. make the image warmer like sunset light or indoor tungsten
#   lighting. similar to setting white balance to "Tungsten" on a camera.
warm_temperature = false

# cool_temperature to cool color shift: blue boosted 1.05x red cut to
#   0.92x. make the image cooler like overcast daylight or moonlight.
#   similar to setting white balance to "Daylight" on a camera.
cool_temperature = false

[color_grading]
# saturation_contrast_grade to combined saturation (1.1x) contrast (1.05x)
#   and brightness offset in a single pass. a general purpose "make it
#   look better" adjustment similar to auto enhance in phone camera.
saturation_contrast_grade = false

# levels_remap to black/white point remapping in the spirit of the
#   Photoshop Levels adjustment. pull the input range slightly inward
#   (0.02 to 0.98) and rescale to full range for a gentle contrast
#   expansion. on/off only.
levels_remap = false

# gamma_correct to fixed gamma curve that darken midtone slightly
#   (gamma 1.1). apply a power function to all channel. the same gamma
#   correction used in display calibration. on/off only.
gamma_correct = false

# vibrance_boost to smart saturation. boost saturation on pixel that
#   be already low saturation while leaving already vivid color alone.
#   prevent clipping artifact that uniform saturation boost cause.
#   the same algorithm as Lightroom "Vibrance" slider (as opposed to
#   "Saturation" which boost everything equally).
vibrance_boost = false

# hsl_transform to a mild saturation boost (1.1x) in HSL color space,
#   in the spirit of the Photoshop Hue/Saturation dialog. on/off only.
#   note: do not enable alongside red/green/blue_channel_curve or
#   hermite_curves as the curve double apply on shared channel.
hsl_transform = false

# split_tone to tint shadow cool (blue) and highlight warm (amber)
#   independently with a luminance driven crossfade. the classic
#   cinematic "orange and teal" look used in blockbuster color grading.
#   same concept as Lightroom Split Toning panel.
split_tone = false

# lift_gamma_gain to a slight shadow lift (the "lift" of a three way
#   color corrector), the standard tool in DaVinci Resolve, Premiere
#   Pro and every professional color grading application. on/off only.
lift_gamma_gain = false

# hermite_curves to s curve contrast via Hermite smoothstep (3t² to 2t³)
#   applied to all channel. the same smooth s curve used in CSS
#   transition and shader easing function. increase contrast in
#   midtone while compressing highlight and shadow. note: enabling
#   alongside any of red/green/blue_channel_curve will double apply
#   on that channel.
hermite_curves = false

[channel_curves]
# per channel s curve for fine color control. note: each channel curve
# stack with hermite_curves on the same channel; enable only one of the
# two for predictable result.

# red_channel_curve to Hermite s curve applied to red channel only
red_channel_curve = false

# green_channel_curve to Hermite s curve applied to green channel only
green_channel_curve = false

# blue_channel_curve to Hermite s curve applied to blue channel only
blue_channel_curve = false

[color_balance]
# trizone_color_balance to three zone color tint. shadow get a teal push
#   (cyan 0.03 magenta +0.01 yellow +0.02) highlight get a warm push
#   (cyan +0.02 yellow 0.02). produce a cinematic teal and orange look
#   similar to the default color grading in many Hollywood blockbuster.
#   same concept as DaVinci Resolve three way color balance wheel.
trizone_color_balance = false

[selective_color]
# boost saturation for pixel dominated by a specific color channel

# red_selective_saturate to boost saturation of red dominant pixel by
#   0.3x their existing saturation. make red pop without affecting
#   green or blue. useful for sunset scene or red themed game.
red_selective_saturate = false

# green_selective_saturate to boost saturation of green dominant pixel
green_selective_saturate = false

# blue_selective_saturate to boost saturation of blue dominant pixel
blue_selective_saturate = false

[stylization]
# creative color effect for specific visual style

# dynamic_range_crush to narrow the dynamic range by clamping black
#   and white point inward. lift shadow and pull down highlight for
#   a flatter more "Instagram filter" look. remap the crushed range
#   back to full [0 1] so the image stay usable.
dynamic_range_crush = false

# duotone_map to map the entire image to two color based on luminance.
#   shadow become dark blue (0.1 0.1 0.3) highlight become warm
#   cream (1.0 0.9 0.7). the same duotone effect used in Spotify
#   playlist cover and modern graphic design.
duotone_map = false

# color_wash_tint to tint the entire image toward a single color with
#   0.2 blend strength. a subtle color wash that unify the palette.
color_wash_tint = false

# posterize_quantize to reduce to 8 color level per channel creating
#   a poster/comic book look with visible color banding. the same effect
#   as Photoshop Posterize filter.
posterize_quantize = false

# bleach_bypass to desaturated high contrast look simulating the "bleach
#   bypass" photochemical process where the bleach bath be skipped during
#   film development. produce the gritty desaturated high contrast
#   look of film like Saving Private Ryan and Minority Report. use
#   an overlay blend mode between color and luminance.
bleach_bypass = false

# technicolor_process to vintage three strip Technicolor simulation.
#   cross bleed complementary color between channel producing the
#   characteristic oversaturated slightly unrealistic color palette of
#   1930s 50s Technicolor film like The Wizard of Oz.
technicolor_process = false

# midpoint_contrast to simple contrast boost: scale the distance from
#   the 0.5 midpoint by 1.5x. make dark darker and light lighter.
#   the simplest possible contrast adjustment.
midpoint_contrast = false

# color_invert to negative image. subtract each channel from 1.0
#   like a photographic negative.
color_invert = false

# luminance_grayscale to convert to grayscale using BT.709 luminance
#   weight (0.2126R + 0.7152G + 0.0722B). the perceptually correct
#   conversion used in HDTV standard to green contribute the most
#   because human vision be most sensitive to green.
luminance_grayscale = false

[accessibility]
# color vision simulation and correction tool

# protanopia_simulation to simulate protanopia (red blind) vision using
#   a 3x3 color transformation matrix. show what the image look like
#   to someone with red cone deficiency (~1% of male).
protanopia_simulation = false

# deuteranopia_simulation to simulate deuteranopia (green blind) vision.
#   the most common color vision deficiency (~6% of male).
deuteranopia_simulation = false

# tritanopia_simulation to simulate tritanopia (blue blind) vision.
#   rare (~0.01% of population).
tritanopia_simulation = false

# protanopia_correct to Daltonization for protanopia. redistribute the
#   color information lost by red cone deficiency to the green and blue
#   channel making red green distinction visible through brightness
#   and blue shift difference. designed to be used BY someone with
#   protanopia to improve their ability to distinguish color.
protanopia_correct = false

# deuteranopia_correct to Daltonization for deuteranopia. same principle
#   correcting for green cone deficiency.
deuteranopia_correct = false

# tritanopia_correct to Daltonization for tritanopia. redistribute lost
#   blue channel information to red and green channel.
tritanopia_correct = false

[inline]
# per pixel lens/film effect applied after color grading before hardware
# simulation. these belong to the "source image" the virtual monitor receive.

# gaussian_grain to film grain using the Box Muller transform to generate
#   Gaussian distributed noise. animated via u_time so the grain pattern
#   change every frame like real film grain. the Box Muller method be the
#   standard way to generate Gaussian noise in shader and scientific
#   computing.
gaussian_grain = false

# chromatic_aberration to lateral chromatic aberration. shift the red
#   channel outward from center and the blue channel inward (or vice
#   versa) simulating the color fringing of a cheap camera lens that
#   cannot focus all wavelength to the same point. happen at master
#   fetch so it form the base image before any neighborhood operation.
chromatic_aberration = false

# red_halation to halation glow. in real film bright highlight scatter
#   light through the film base and re expose the emulsion from behind
#   creating a red glow around bright object. derived from shared corner
#   tap so it dedupe with any other 3x3 consumer enabled.
red_halation = false

# anamorphic_streak to horizontal streak flare from bright highlight.
#   simulate the horizontal lens flare streak produced by anamorphic
#   (widescreen) cinema lens. six bilinear paired horizontal sample
#   with brightness threshold tinted blue to match real anamorphic
#   coating. all distance resolution scaled.
anamorphic_streak = false

# radial_vignette to darkened edge with smoothstep falloff from inner
#   to outer radius. simulate the natural light falloff at the edge of
#   a camera lens. standard cinematic technique to draw the eye toward
#   the center of the frame.
radial_vignette = false

# cinematic_letterbox to black bar for 2.35:1 widescreen aspect ratio.
#   the same aspect ratio used by most Hollywood blockbuster (called
#   "Scope" or "CinemaScope"). add black bar top and bottom to crop
#   a 16:9 image to the wider cinematic ratio.
cinematic_letterbox = false

# ordered_dither to Bayer 4x4 ordered dithering (Bayer 1973). add a
#   spatially structured noise pattern that break up visible color
#   banding in gradient. the same technique used in GIF image and
#   retro game console hardware.
ordered_dither = false

[hardware_simulation]
# simulate specific hardware rendering and display characteristic.
# split across two phase: deterministic UV warp run in the geometric
# stage (alongside fisheye and barrel) so temporal stay aligned, and
# color/mask/quantize run at the end so they act as the "monitor"
# showing the finished image. AA sharpen and color grade see the ideal
# frame not the CRT distorted one.
#
# pick ONE console GPU simulation for best result. stacking multiple be
# undefined but never rejected. pair one console with one display sim
# for the full experience: ps1_simulation + crt_simulation = 90s living
# room. psp_simulation + lcd_subpixel = handheld screen.
# n64_simulation + crt_simulation = childhood.

# ps1_simulation to PlayStation 1 GPU simulation. vertex snap via
#   coarse grid quantization simulate the GTE fixed point integer
#   coordinate quantization. nearest neighbor snap remove bilinear
#   filtering. 15 bit color (5 bit per channel 32 level) with Bayer
#   4x4 ordered dither.
ps1_simulation = false

# saturn_simulation to Sega Saturn GPU simulation. coarse pixel
#   quantization plus smoothing pass approximate the gouraud shading
#   interpolation banding across polygon face. dark desaturated color
#   palette with warm brown shift. 15 bit color with dither.
saturn_simulation = false

# n64_simulation to Nintendo 64 GPU simulation. unique 3 point bilinear
#   filter create the characteristic vaseline smear look. radial
#   distance fog darken screen edge. warm color shift. the opposite
#   of PS1: where PS1 be sharp and jittery N64 be soft and smeared.
n64_simulation = false

# dreamcast_simulation to Sega Dreamcast GPU simulation. visible texture
#   shimmer from edge weighting. over bright color response and boosted
#   saturation. visible polygon edge darkening. subtle specular boost
#   on bright area.
dreamcast_simulation = false

# ps2_simulation to PlayStation 2 GPU simulation. 480i interlace combing
#   on alternating field. horizontal chroma bleed from the GS color
#   path. soft bloom on bright surface. subtle scanline darkening.
ps2_simulation = false

# xbox_simulation to original Xbox GPU simulation. DXT S3TC texture
#   compression 4x4 pixel block artifact. heavy sharpening with
#   ringing overshoot. crude large radius bloom. plastic specular
#   boost. slight green warm color bias.
xbox_simulation = false

# psp_simulation to PlayStation Portable GPU simulation. UV phase quantize
#   to 480x272 native resolution. LCD gamma washout with black floor at
#   6 percent and white ceiling at 92 percent. dark region color banding.
#   slight desaturation. run independent of the temporal stabilizer.
psp_simulation = false

# ps3_simulation to PlayStation 3 GPU simulation. sub HD rendering around
#   72 percent native upscaled with bilinear filtering. quincunx anti
#   aliasing half pixel diagonal blend. crushed shadow from RSX gamma.
#   cool color shift. tear line removed (time varying so it broke the
#   phase rule); the rest of the PS3 look land intact.
ps3_simulation = false

# xbox360_simulation to Xbox 360 GPU simulation. sharp 4xMSAA edge cleanup
#   from free eDRAM anti aliasing. eDRAM tiling seam at one third and
#   two third screen height. HDR gradient banding in bright region.
#   lifted warm gamma. slight desaturation with specular peak boost.
xbox360_simulation = false

# crt_simulation to CRT television simulation. barrel warp distortion
#   in the geometric stage, RGB phosphor triad mask scanline darkening
#   and brightness compensation in the color stage. brightness pulsing
#   replace the old random instability so temporal history stay aligned.
crt_simulation = false

# phosphor_amber to monochrome amber phosphor CRT. the warm orange white
#   look of 1980s IBM PC monitor and airport departure board.
phosphor_amber = false

# phosphor_green to monochrome green phosphor CRT. the classic "hacker
#   terminal" look from 1970s 80s computer (Apple II VT100).
phosphor_green = false

# phosphor_red to monochrome red phosphor CRT. less common historically
#   but used in some military and medical display.
phosphor_red = false

# scanline_darken to horizontal scanline darkening only without the barrel
#   warp and phosphor mask of crt_simulation. add the characteristic
#   "lined" look of a CRT without the geometric distortion.
scanline_darken = false

# oled_simulation to simulate OLED display characteristic: near black
#   crush (dark gray fade toward black via smoothstep) plus saturation
#   boost. make the image look like it displayed on a high end OLED TV.
oled_simulation = false

# vhs_simulation to VHS videotape degradation. deterministic horizontal
#   ripple in the geometric stage. chroma bandwidth reduction luma noise
#   color desaturation warm shift and dropout in the color stage. the
#   ripple amplitude be fixed per line so temporal stay coherent.
vhs_simulation = false

# lcd_subpixel to visible LCD subpixel grid overlay. show the RGB
#   subpixel structure you would see examining an LCD screen with a magnifying
#   glass. cell size scale with resolution so it look correct at any
#   display resolution.
lcd_subpixel = false

[overlay]
# HUD element drawn on top of the processed image. overlay render
# after all processing so they be not affected by temporal blending
# tonemapping color grading or any other effect. your crosshair
# stay the same color and your FPS counter do not ghost.

# fps_hud to performance overlay showing FPS as a uniquely
#   colored 7 segment display. Just measures FPS.
fps_hud = false

# crosshair_overlay to centered crosshair with gap arm and center dot
#   all resolution scaled. styled with a neon blue core and glow to
#   match the fps hud. useful for game that lack a built in crosshair
#   or when you want a consistent crosshair across different game.
crosshair_overlay = false
"#;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectKind {
    Geometric,
    Denoise,
    AntiAliasing,
    Sharpening,
    LocalContrast,
    Blur,
    ImageQuality,
    Temporal,
    Exposure,
    Tonemapping,
    WhiteBalance,
    ColorGrading,
    ChannelCurves,
    ColorBalance,
    SelectiveColor,
    Stylization,
    Accessibility,
    Inline,
    HardwareSimulation,
    Overlay,
}

pub(crate) struct EffectDef {
    pub(crate) name: &'static str,
    pub(crate) kind: EffectKind,
}

pub(crate) const REGISTRY: [EffectDef; 125] = [
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
    EffectDef { name: "protanopia_simulation", kind: EffectKind::Accessibility },
    EffectDef { name: "deuteranopia_simulation", kind: EffectKind::Accessibility },
    EffectDef { name: "tritanopia_simulation", kind: EffectKind::Accessibility },
    EffectDef { name: "protanopia_correct", kind: EffectKind::Accessibility },
    EffectDef { name: "deuteranopia_correct", kind: EffectKind::Accessibility },
    EffectDef { name: "tritanopia_correct", kind: EffectKind::Accessibility },
    EffectDef { name: "gaussian_grain", kind: EffectKind::Inline },
    EffectDef { name: "chromatic_aberration", kind: EffectKind::Inline },
    EffectDef { name: "red_halation", kind: EffectKind::Inline },
    EffectDef { name: "anamorphic_streak", kind: EffectKind::Inline },
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
    EffectDef { name: "fps_hud", kind: EffectKind::Overlay },
    EffectDef { name: "crosshair_overlay", kind: EffectKind::Overlay },
];

pub(crate) const VERT_SRC: &str = r#"#version 460
void main() {
    vec2 vk_pos = vec2(float((gl_VertexIndex << 1) & 2), float(gl_VertexIndex & 2));
    gl_Position = vec4(vk_pos * 2.0 - 1.0, 0.0, 1.0);
}
"#;

pub(crate) const UBER_SRC: &str = r#"#version 460
#ifdef COMPUTE_PATH
layout(local_size_x = LOCAL_SIZE_X, local_size_y = LOCAL_SIZE_Y, local_size_z = 1) in;
layout(set=0, binding=0) uniform sampler2D u_input;
layout(set=0, binding=1) uniform sampler2D u_history;
layout(set=0, binding=2, rgba8) uniform writeonly image2D u_output;
layout(push_constant) uniform PushBlock { vec2 res; float time; float fps; } pc;
#define u_resolution pc.res
#define u_time pc.time
#define u_fps pc.fps
#define BONES_FRAGCOORD vec2(gl_GlobalInvocationID.xy) + vec2(0.5)
#define BONES_WRITE_OUT(rgb) imageStore(u_output, ivec2(gl_GlobalInvocationID.xy), vec4(clamp(rgb, 0.0, 1.0), 1.0))
#define BONES_EARLY_OUT if (any(greaterThanEqual(gl_GlobalInvocationID.xy, uvec2(u_resolution)))) return
#else
layout(set=0, binding=0) uniform sampler2D u_input;
layout(set=0, binding=1) uniform sampler2D u_history;
layout(push_constant) uniform PushBlock { vec2 res; float time; float fps; } pc;
#define u_resolution pc.res
#define u_time pc.time
#define u_fps pc.fps
layout(location=0) out vec4 frag_out;
#define BONES_FRAGCOORD gl_FragCoord.xy
#define BONES_WRITE_OUT(rgb) frag_out = vec4(clamp(rgb, 0.0, 1.0), 1.0)
#define BONES_EARLY_OUT
#endif

const vec3 LUMA_BT601 = vec3(0.299, 0.587, 0.114);
const vec3 LUMA_BT709 = vec3(0.2126, 0.7152, 0.0722);
const vec3 LUMA_AVG = vec3(0.3333333333);
const vec3 ONE3 = vec3(1.0);
const vec2 HALF2 = vec2(0.5);
const vec3 HALF3 = vec3(0.5);
const vec3 ZERO3 = vec3(0.0);

#if defined(ENABLE_NEIGHBORHOOD_CLAMP_AA) || defined(ENABLE_MOTION_REJECT_DENOISE) || defined(ENABLE_MOTION_DETECT_BLUR) || defined(ENABLE_CONSTANT_BLEND_SMOOTH) || defined(ENABLE_SHUTTER_ANGLE_SMOOTH) || defined(ENABLE_SPLINE_INTERP_SMOOTH) || defined(ENABLE_VARIANCE_DECAY_SMOOTH) || defined(ENABLE_DUALRATE_SMOOTH) || defined(ENABLE_LUMINANCE_GATE_SMOOTH) || defined(ENABLE_CONTRAST_GATE_SMOOTH) || defined(ENABLE_GRADIENT_GATE_SMOOTH) || defined(ENABLE_SIGMA_CLIP_SMOOTH) || defined(ENABLE_MITCHELL_KERNEL_SMOOTH) || defined(ENABLE_YCOCG_CLIP_SMOOTH) || defined(ENABLE_BILATERAL_HISTORY_SMOOTH) || defined(ENABLE_PERCEPTUAL_CHROMA_SMOOTH) || defined(ENABLE_FREQUENCY_SPLIT_SMOOTH) || defined(ENABLE_HORN_SCHUNCK_SMOOTH) || defined(ENABLE_CONVERGENT_ACCUMULATE) || defined(ENABLE_DUALWARP_FLOW_SMOOTH) || defined(ENABLE_VARIANCE_FLOW_ACCUMULATE) || defined(ENABLE_EDGE_RECONSTRUCT_SMOOTH)
    #define ROLE_HISTORY
    #define ENABLE_TEMPORAL_STABILIZER
#endif

#if defined(ENABLE_HORN_SCHUNCK_SMOOTH) || defined(ENABLE_DUALWARP_FLOW_SMOOTH) || defined(ENABLE_VARIANCE_FLOW_ACCUMULATE) || defined(ENABLE_EDGE_RECONSTRUCT_SMOOTH)
    #define ROLE_HISTORY_FLOW
#endif

#if defined(ENABLE_LUMA_EDGE_AA) || defined(ENABLE_NORMAL_FILTER_AA) || defined(ENABLE_MORPHOLOGICAL_AA) || defined(ENABLE_SUBPIXEL_AA) || defined(ENABLE_CONTRAST_ADAPTIVE_SHARPEN) || defined(ENABLE_ROBUST_CONTRAST_SHARPEN) || defined(ENABLE_EDGE_DIRECTED_SHARPEN) || defined(ENABLE_LAPLACIAN_SHARPEN) || defined(ENABLE_LUMINANCE_SHARPEN) || defined(ENABLE_MIDTONE_CLARITY) || defined(ENABLE_FALLOFF_SHARPEN) || defined(ENABLE_POWER_CURVE_SHARPEN) || defined(ENABLE_UNSHARP_MASK) || defined(ENABLE_LOCAL_CONTRAST) || defined(ENABLE_GAUSSIAN_BLUR) || defined(ENABLE_THRESHOLD_BLOOM) || defined(ENABLE_RED_HALATION) || defined(ENABLE_NEIGHBORHOOD_CLAMP_AA) || defined(ENABLE_SIGMA_CLIP_SMOOTH) || defined(ENABLE_DUALRATE_SMOOTH) || defined(ENABLE_HORN_SCHUNCK_SMOOTH) || defined(ENABLE_FREQUENCY_SPLIT_SMOOTH) || defined(ENABLE_GRADIENT_GATE_SMOOTH) || defined(ENABLE_BILATERAL_HISTORY_SMOOTH) || defined(ENABLE_CONTRAST_GATE_SMOOTH) || defined(ENABLE_DUALWARP_FLOW_SMOOTH) || defined(ENABLE_VARIANCE_FLOW_ACCUMULATE) || defined(ENABLE_EDGE_RECONSTRUCT_SMOOTH) || defined(ENABLE_YCOCG_CLIP_SMOOTH) || defined(ENABLE_MITCHELL_KERNEL_SMOOTH) || defined(ENABLE_BILATERAL_DENOISE)
    #define ROLE_TAPS_CROSS
#endif

#if defined(ENABLE_LUMA_EDGE_AA) || defined(ENABLE_MORPHOLOGICAL_AA) || defined(ENABLE_SUBPIXEL_AA) || defined(ENABLE_CONTRAST_ADAPTIVE_SHARPEN) || defined(ENABLE_FALLOFF_SHARPEN) || defined(ENABLE_UNSHARP_MASK) || defined(ENABLE_GAUSSIAN_BLUR) || defined(ENABLE_THRESHOLD_BLOOM) || defined(ENABLE_RED_HALATION) || defined(ENABLE_NEIGHBORHOOD_CLAMP_AA) || defined(ENABLE_SIGMA_CLIP_SMOOTH) || defined(ENABLE_MITCHELL_KERNEL_SMOOTH) || defined(ENABLE_BILATERAL_DENOISE)
    #define ROLE_TAPS_CORNER
#endif

#if defined(ENABLE_CONTRAST_ADAPTIVE_SHARPEN) || defined(ENABLE_ROBUST_CONTRAST_SHARPEN) || defined(ENABLE_EDGE_DIRECTED_SHARPEN) || defined(ENABLE_LAPLACIAN_SHARPEN) || defined(ENABLE_LUMINANCE_SHARPEN) || defined(ENABLE_MIDTONE_CLARITY) || defined(ENABLE_POWER_CURVE_SHARPEN) || defined(ENABLE_UNSHARP_MASK) || defined(ENABLE_LOCAL_CONTRAST) || defined(ENABLE_GAUSSIAN_BLUR) || defined(ENABLE_FALLOFF_SHARPEN) || defined(ENABLE_DUALRATE_SMOOTH) || defined(ENABLE_SIGMA_CLIP_SMOOTH) || defined(ENABLE_FREQUENCY_SPLIT_SMOOTH) || defined(ENABLE_VARIANCE_FLOW_ACCUMULATE) || defined(ENABLE_CONTRAST_GATE_SMOOTH) || defined(ENABLE_BILATERAL_HISTORY_SMOOTH) || defined(ENABLE_MITCHELL_KERNEL_SMOOTH)
    #define ROLE_SUM_CROSS
#endif

#if defined(ENABLE_CONTRAST_ADAPTIVE_SHARPEN) || defined(ENABLE_FALLOFF_SHARPEN) || defined(ENABLE_UNSHARP_MASK) || defined(ENABLE_GAUSSIAN_BLUR) || defined(ENABLE_SIGMA_CLIP_SMOOTH) || defined(ENABLE_MITCHELL_KERNEL_SMOOTH)
    #define ROLE_SUM_CORNER
#endif

#if defined(ENABLE_CONTRAST_ADAPTIVE_SHARPEN) || defined(ENABLE_ROBUST_CONTRAST_SHARPEN) || defined(ENABLE_NEIGHBORHOOD_CLAMP_AA) || defined(ENABLE_DUALWARP_FLOW_SMOOTH) || defined(ENABLE_EDGE_RECONSTRUCT_SMOOTH) || defined(ENABLE_YCOCG_CLIP_SMOOTH)
    #define ROLE_BOUNDS_CROSS
#endif

#if defined(ENABLE_CONTRAST_ADAPTIVE_SHARPEN) || defined(ENABLE_NEIGHBORHOOD_CLAMP_AA)
    #define ROLE_BOUNDS_3X3
#endif

#if defined(ENABLE_EDGE_DIRECTED_SHARPEN) || defined(ENABLE_NORMAL_FILTER_AA) || defined(ENABLE_HORN_SCHUNCK_SMOOTH) || defined(ENABLE_DUALWARP_FLOW_SMOOTH) || defined(ENABLE_VARIANCE_FLOW_ACCUMULATE) || defined(ENABLE_EDGE_RECONSTRUCT_SMOOTH) || defined(ENABLE_GRADIENT_GATE_SMOOTH)
    #define ROLE_GRAD_LUMA
#endif

#if defined(ENABLE_BILATERAL_HISTORY_SMOOTH) || defined(ENABLE_FREQUENCY_SPLIT_SMOOTH)
    #define ROLE_HISTORY
    #define ROLE_HISTORY_CROSS
#endif

#if defined(ENABLE_GRADIENT_DEBAND) || defined(ENABLE_GAUSSIAN_GRAIN) || defined(ENABLE_VHS_SIMULATION) || defined(ENABLE_CRT_SIMULATION) || defined(ENABLE_PS2_SIMULATION) || defined(ENABLE_PS1_SIMULATION) || defined(ENABLE_ORDERED_DITHER) || defined(ENABLE_SATURN_SIMULATION) || defined(ENABLE_N64_SIMULATION) || defined(ENABLE_PSP_SIMULATION)
    #define ROLE_HASH
#endif

#if defined(ROLE_HASH)
    float hash21(vec2 p) {
        vec3 p3 = fract(vec3(p.xyx) * 0.1031);
        p3 += dot(p3, p3.yzx + 33.33);
        return fract((p3.x + p3.y) * p3.z);
    }
#endif

#if defined(ENABLE_CRT_SIMULATION) || defined(ENABLE_VHS_SIMULATION) || defined(ENABLE_GAUSSIAN_GRAIN) || defined(ENABLE_PS2_SIMULATION) || defined(ENABLE_XBOX360_SIMULATION)
    float fast_sin(float x) {
        x = x * 0.15915494 + 0.5;
        x = fract(x) * 2.0 - 1.0;
        float a = abs(x);
        return x * (3.5841 - 2.5841 * a) * (1.0 - a);
    }
    float fast_cos(float x) {
        return fast_sin(x + 1.5707963);
    }
#endif

#if defined(ENABLE_PS1_SIMULATION) || defined(ENABLE_SATURN_SIMULATION) || defined(ENABLE_N64_SIMULATION) || defined(ENABLE_PSP_SIMULATION) || defined(ENABLE_ORDERED_DITHER)
    const float BAYER4[16] = float[16](
         0.0,  8.0,  2.0, 10.0,
        12.0,  4.0, 14.0,  6.0,
         3.0, 11.0,  1.0,  9.0,
        15.0,  7.0, 13.0,  5.0
    );
    float bayer_signed(vec2 fc) {
        int ix = int(floor(fc.x)) & 3;
        int iy = int(floor(fc.y)) & 3;
        return (BAYER4[ix + iy * 4] + 0.5) * 0.0625 - 0.5;
    }
#endif

#if defined(ENABLE_YCOCG_CLIP_SMOOTH)
    vec3 ycocg_encode(vec3 x) {
        return vec3(0.25 * x.r + 0.5 * x.g + 0.25 * x.b,
                    0.5 * x.r - 0.5 * x.b,
                    -0.25 * x.r + 0.5 * x.g - 0.25 * x.b);
    }
    vec3 ycocg_decode(vec3 y) {
        return vec3(y.x + y.y - y.z, y.x + y.z, y.x - y.y - y.z);
    }
#endif

#if defined(ENABLE_PERCEPTUAL_CHROMA_SMOOTH)
    vec3 perc_ycc(vec3 x) {
        float y = dot(x, LUMA_BT709);
        return vec3(y, (x.b - y) * 0.5388766, (x.r - y) * 0.6350048);
    }
    vec3 perc_rgb(vec3 y) {
        float r = y.x + y.z * 1.5748;
        float b = y.x + y.y * 1.8556;
        float g = (y.x - LUMA_BT709.r * r - LUMA_BT709.b * b) * 1.398313;
        return vec3(r, g, b);
    }
#endif

#if defined(ENABLE_HABLE_TONEMAP)
    vec3 hable_map(vec3 x) {
        return ((x * (0.15 * x + 0.05) + 0.004) /
                (x * (0.15 * x + 0.50) + 0.060)) - 0.066666666;
    }
#endif

#if defined(ENABLE_AGX_TONEMAP)
    vec3 agx_contrast(vec3 x) {
        vec3 x2 = x * x;
        vec3 x4 = x2 * x2;
        return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4
             - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
    }
#endif

#if defined(ENABLE_UCHIMURA_TONEMAP)
    vec3 uchi_map(vec3 x) {
        const float U_P = 1.0;
        const float U_A = 1.0;
        const float U_M = 0.22;
        const float U_L = 0.4;
        const float U_C = 1.33;
        float u_l0 = (U_P - U_M) * U_L;
        float u_s0 = U_M + u_l0;
        float u_s1 = U_M + U_A * u_l0;
        float u_c2 = (U_A * U_P) / max(U_P - u_s1, 0.0001);
        vec3 u_toe = U_M * pow(max(x, ZERO3) / max(U_M, 0.0001), vec3(U_C));
        vec3 u_lin = vec3(U_M) + U_A * (x - U_M);
        vec3 u_sho = U_P - (U_P - u_s1) * exp(-u_c2 * (x - u_s0));
        return mix(mix(u_toe, u_lin, smoothstep(0.0, U_M, x)),
                   u_sho, smoothstep(U_M, u_s0, x));
    }
#endif

#if defined(ENABLE_MITCHELL_KERNEL_SMOOTH)
    float mn_kernel(float x) {
        float ax = abs(x);
        float ax2 = ax * ax;
        float ax3 = ax2 * ax;
        float w_inner = 1.16666667 * ax3 - 2.0 * ax2 + 0.88888889;
        float w_outer = -0.38888889 * ax3 + 2.0 * ax2 - 3.33333333 * ax + 1.77777778;
        float in_inner = step(ax, 1.0);
        float in_outer = step(ax, 2.0) - in_inner;
        return w_inner * in_inner + w_outer * in_outer;
    }
    float mn_w2d(float dx, float dy) {
        return mn_kernel(dx) * mn_kernel(dy);
    }
#endif

#if defined(ENABLE_FPS_HUD)
    float hud_seg(vec2 p, float mx, float my, float dx, float dy) {
        vec2 d = abs(p - vec2(mx, my)) - vec2(dx, dy);
        return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
    }

    float hud_digit(vec2 p, float n) {
        p.y = 1.0 - p.y;
        float s0 = hud_seg(p, 0.5, 0.85, 0.15, 0.0);
        float s1 = hud_seg(p, 0.75, 0.65, 0.0, 0.15);
        float s2 = hud_seg(p, 0.75, 0.25, 0.0, 0.15);
        float s3 = hud_seg(p, 0.5, 0.05, 0.15, 0.0);
        float s4 = hud_seg(p, 0.25, 0.25, 0.0, 0.15);
        float s5 = hud_seg(p, 0.25, 0.65, 0.0, 0.15);
        float s6 = hud_seg(p, 0.5, 0.45, 0.15, 0.0);
        float ge0 = step(0.5, n);
        float ge1 = step(1.5, n);
        float ge2 = step(2.5, n);
        float ge3 = step(3.5, n);
        float ge4 = step(4.5, n);
        float ge5 = step(5.5, n);
        float ge6 = step(6.5, n);
        float ge7 = step(7.5, n);
        float ge8 = step(8.5, n);
        float m0 = 1.0 - ge0;
        float m1 = ge0 - ge1;
        float m2 = ge1 - ge2;
        float m3 = ge2 - ge3;
        float m4 = ge3 - ge4;
        float m5 = ge4 - ge5;
        float m6 = ge5 - ge6;
        float m7 = ge6 - ge7;
        float m8 = ge7 - ge8;
        float m9 = ge8;
        float r0 = m0 * min(s0, min(s1, min(s2, min(s3, min(s4, s5)))));
        float r1 = m1 * min(s1, s2);
        float r2 = m2 * min(s0, min(s1, min(s6, min(s4, s3))));
        float r3 = m3 * min(s0, min(s1, min(s6, min(s2, s3))));
        float r4 = m4 * min(s5, min(s6, min(s1, s2)));
        float r5 = m5 * min(s0, min(s5, min(s6, min(s2, s3))));
        float r6 = m6 * min(s0, min(s5, min(s6, min(s4, min(s3, s2)))));
        float r7 = m7 * min(s0, min(s1, s2));
        float r8 = m8 * min(s0, min(s1, min(s2, min(s3, min(s4, min(s5, s6))))));
        float r9 = m9 * min(s0, min(s1, min(s2, min(s3, min(s5, s6)))));
        return r0 + r1 + r2 + r3 + r4 + r5 + r6 + r7 + r8 + r9;
    }
#endif

void main() {
    BONES_EARLY_OUT;
    vec2 frag_coord = BONES_FRAGCOORD;
    vec2 inv = 1.0 / u_resolution;
    vec2 v_uv = frag_coord * inv;
    float res_scale = u_resolution.y * 0.0009259259;

    #ifdef ENABLE_MIRROR_HORIZONTAL
        v_uv.x = 1.0 - v_uv.x;
    #endif

    #ifdef ENABLE_MIRROR_VERTICAL
        v_uv.y = 1.0 - v_uv.y;
    #endif

    #ifdef ENABLE_ROTATE_90
        v_uv = vec2(v_uv.y, 1.0 - v_uv.x);
    #endif

    #ifdef ENABLE_ROTATE_180
        v_uv = vec2(1.0) - v_uv;
    #endif

    #ifdef ENABLE_ROTATE_270
        v_uv = vec2(1.0 - v_uv.y, v_uv.x);
    #endif

    #ifdef ENABLE_CENTER_ZOOM
        v_uv = HALF2 + (v_uv - HALF2) * 0.6666666667;
    #endif

    #ifdef ENABLE_POLYNOMIAL_DISTORT
        {
            vec2 dc = v_uv - HALF2;
            v_uv = HALF2 + dc * (1.0 + 0.1 * dot(dc, dc));
        }
    #endif

    #ifdef ENABLE_BARREL_UNDISTORT
        {
            vec2 bc = v_uv - HALF2;
            v_uv = HALF2 + bc / max(1.0 + 0.2 * dot(bc, bc), 0.0001);
        }
    #endif

    #ifdef ENABLE_FISHEYE_WARP
        {
            vec2 fc = v_uv - HALF2;
            float fr = sqrt(dot(fc, fc));
            float ff = mix(1.0, atan(fr) / max(fr, 0.0001), step(0.0001, fr));
            v_uv = HALF2 + fc * ff;
        }
    #endif

    #ifdef ENABLE_TRAPEZOID_WARP
        v_uv.x = 0.5 + (v_uv.x - 0.5) * mix(1.0, 1.2, v_uv.y);
    #endif

    #ifdef ENABLE_SHARP_BILINEAR
        {
            vec2 pg = u_resolution / max(4.0 * res_scale, 1.0);
            vec2 pt = v_uv * pg;
            vec2 pi = floor(pt - 0.5) + 0.5;
            vec2 pf = pt - pi;
            pf = pf * pf * (3.0 - 2.0 * pf);
            v_uv = (pi + pf) / pg;
        }
    #endif

    #ifdef ENABLE_CRT_SIMULATION
        {
            vec2 cc2 = v_uv - HALF2;
            float cr = dot(cc2, cc2);
            v_uv = v_uv + cc2 * cr * vec2(0.031, 0.041);
        }
    #endif

    #ifdef ENABLE_VHS_SIMULATION
        {
            float vrip_arg = v_uv.y * 4.77464829;
            float vrip = (abs(fract(vrip_arg) - 0.5) * 4.0 - 1.0) * 1.5 * res_scale * inv.x;
            v_uv.x = v_uv.x + vrip;
        }
    #endif

    #ifdef ENABLE_PS1_SIMULATION
        {
            const float PS1_GRID = 140.0;
            vec2 ps_grid = floor(v_uv * PS1_GRID);
            v_uv = (ps_grid + 0.5) * 0.00714286;
        }
    #endif

    #ifdef ENABLE_SATURN_SIMULATION
        {
            float sat_ps = 2.0 * res_scale;
            v_uv = (floor(v_uv * u_resolution / sat_ps) + 0.5) * sat_ps * inv;
        }
    #endif

    #ifdef ENABLE_N64_SIMULATION
        {
            float n64_ps = 1.5 * res_scale;
            v_uv = (floor(v_uv * u_resolution / n64_ps) + 0.5) * n64_ps * inv;
        }
    #endif

    #ifdef ENABLE_PS2_SIMULATION
        {
            float ps2_ps = 1.8 * res_scale;
            v_uv = (floor(v_uv * u_resolution / ps2_ps) + 0.5) * ps2_ps * inv;
        }
    #endif

    #ifdef ENABLE_PSP_SIMULATION
        {
            vec2 psp_res = vec2(480.0, 272.0);
            v_uv = (floor(v_uv * psp_res) + 0.5) / psp_res;
        }
    #endif

    #ifdef ENABLE_PS3_SIMULATION
        {
            vec2 ps3_res = u_resolution * 0.72;
            v_uv = (floor(v_uv * ps3_res) + 0.5) / ps3_res;
        }
    #endif

    vec3 c;
    #ifdef ENABLE_CHROMATIC_ABERRATION
        {
            vec2 ca_d = (v_uv - HALF2) * 0.005;
            c.r = texture(u_input, v_uv + ca_d).r;
            c.g = texture(u_input, v_uv).g;
            c.b = texture(u_input, v_uv - ca_d).b;
        }
    #else
        c = texture(u_input, v_uv).rgb;
    #endif

    #ifdef ROLE_TAPS_CROSS
        vec3 tap_1_0  = texture(u_input, v_uv + vec2( inv.x, 0.0)).rgb;
        vec3 tap_m1_0 = texture(u_input, v_uv + vec2(-inv.x, 0.0)).rgb;
        vec3 tap_0_1  = texture(u_input, v_uv + vec2(0.0,  inv.y)).rgb;
        vec3 tap_0_m1 = texture(u_input, v_uv + vec2(0.0, -inv.y)).rgb;
    #endif

    #ifdef ROLE_TAPS_CORNER
        vec3 tap_1_1   = texture(u_input, v_uv + vec2( inv.x,  inv.y)).rgb;
        vec3 tap_m1_1  = texture(u_input, v_uv + vec2(-inv.x,  inv.y)).rgb;
        vec3 tap_1_m1  = texture(u_input, v_uv + vec2( inv.x, -inv.y)).rgb;
        vec3 tap_m1_m1 = texture(u_input, v_uv + vec2(-inv.x, -inv.y)).rgb;
    #endif

    #ifdef ROLE_SUM_CROSS
        vec3 cross_sum = tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1;
        vec3 cross_avg = cross_sum * 0.25;
    #endif

    #ifdef ROLE_SUM_CORNER
        vec3 corner_sum = tap_1_1 + tap_m1_1 + tap_1_m1 + tap_m1_m1;
    #endif

    #ifdef ROLE_BOUNDS_CROSS
        vec3 box_min_x = min(min(tap_1_0, tap_m1_0), min(tap_0_1, tap_0_m1));
        vec3 box_max_x = max(max(tap_1_0, tap_m1_0), max(tap_0_1, tap_0_m1));
    #endif

    #ifdef ROLE_BOUNDS_3X3
        vec3 box_min_3x3 = min(box_min_x, min(min(tap_1_1, tap_m1_1), min(tap_1_m1, tap_m1_m1)));
        vec3 box_max_3x3 = max(box_max_x, max(max(tap_1_1, tap_m1_1), max(tap_1_m1, tap_m1_m1)));
    #endif

    #ifdef ROLE_GRAD_LUMA
        vec3 grad_x_rgb = tap_1_0 - tap_m1_0;
        vec3 grad_y_rgb = tap_0_1 - tap_0_m1;
        float lgrad_x = dot(grad_x_rgb, LUMA_AVG);
        float lgrad_y = dot(grad_y_rgb, LUMA_AVG);
    #endif

    #ifdef ROLE_HISTORY
        vec3 history = texture(u_history, v_uv).rgb;
        float hist_valid = step(1e-6, dot(history, history));
    #endif

    #ifdef ROLE_HISTORY_CROSS
        vec3 hist_e = texture(u_history, v_uv + vec2( inv.x, 0.0)).rgb;
        vec3 hist_w = texture(u_history, v_uv + vec2(-inv.x, 0.0)).rgb;
        vec3 hist_n = texture(u_history, v_uv + vec2(0.0,  inv.y)).rgb;
        vec3 hist_s = texture(u_history, v_uv + vec2(0.0, -inv.y)).rgb;
    #endif

    #ifdef ENABLE_BILATERAL_DENOISE
        {
            vec3 e00 = tap_m1_m1 - c; float w00 = 1.0 / (1.0 + dot(e00, e00) * 100.0);
            vec3 e10 = tap_0_m1  - c; float w10 = 1.0 / (1.0 + dot(e10, e10) * 100.0);
            vec3 e20 = tap_1_m1  - c; float w20 = 1.0 / (1.0 + dot(e20, e20) * 100.0);
            vec3 e01 = tap_m1_0  - c; float w01 = 1.0 / (1.0 + dot(e01, e01) * 100.0);
            vec3 e21 = tap_1_0   - c; float w21 = 1.0 / (1.0 + dot(e21, e21) * 100.0);
            vec3 e02 = tap_m1_1  - c; float w02 = 1.0 / (1.0 + dot(e02, e02) * 100.0);
            vec3 e12 = tap_0_1   - c; float w12 = 1.0 / (1.0 + dot(e12, e12) * 100.0);
            vec3 e22 = tap_1_1   - c; float w22 = 1.0 / (1.0 + dot(e22, e22) * 100.0);
            vec3 ds = tap_m1_m1*w00 + tap_0_m1*w10 + tap_1_m1*w20 + tap_m1_0*w01 + c
                    + tap_1_0*w21 + tap_m1_1*w02 + tap_0_1*w12 + tap_1_1*w22;
            float dw = w00 + w10 + w20 + w01 + 1.0 + w21 + w02 + w12 + w22;
            c = mix(c, ds / max(dw, 0.0001), 0.6);
        }
    #endif

    #ifdef ENABLE_LUMA_EDGE_AA
        {
            float lc = dot(c, LUMA_BT601);
            float ln = dot(tap_0_m1, LUMA_BT601);
            float ls = dot(tap_0_1, LUMA_BT601);
            float le = dot(tap_1_0, LUMA_BT601);
            float lw = dot(tap_m1_0, LUMA_BT601);
            float lnw = dot(tap_m1_m1, LUMA_BT601);
            float lne = dot(tap_1_m1, LUMA_BT601);
            float lsw = dot(tap_m1_1, LUMA_BT601);
            float lse = dot(tap_1_1, LUMA_BT601);
            float fmn = min(lc, min(min(ln, ls), min(le, lw)));
            float fmx = max(lc, max(max(ln, ls), max(le, lw)));
            vec2 fd = vec2(-((lnw + lne) - (lsw + lse)), (lnw + lsw) - (lne + lse));
            float fr = max((lnw + lne + lsw + lse) * 0.03125, 0.0078125);
            float fp = 1.0 / max(min(abs(fd.x), abs(fd.y)) + fr, 0.0001);
            fd = clamp(fd * fp, vec2(-8.0), vec2(8.0)) * inv;
            vec3 fa = (texture(u_input, v_uv + fd * -0.16666667).rgb +
                       texture(u_input, v_uv + fd *  0.16666667).rgb) * 0.5;
            vec3 fb = fa * 0.5 + (texture(u_input, v_uv + fd * -0.5).rgb +
                                  texture(u_input, v_uv + fd *  0.5).rgb) * 0.25;
            float fl = dot(fb, LUMA_BT601);
            c = mix(fb, fa, clamp(step(fl, fmn) + step(fmx, fl), 0.0, 1.0));
        }
    #endif

    #ifdef ENABLE_NORMAL_FILTER_AA
        {
            vec2 nt = vec2(-lgrad_y, lgrad_x) * (1.5 * res_scale) * inv;
            float ng2 = lgrad_x * lgrad_x + lgrad_y * lgrad_y;
            c = mix(c, (texture(u_input, v_uv + nt).rgb +
                        texture(u_input, v_uv - nt).rgb + c) * 0.3333333,
                    clamp(ng2 * 16.0, 0.0, 1.0));
        }
    #endif

    #ifdef ENABLE_MORPHOLOGICAL_AA
        {
            vec3 ca = (tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) * 0.25;
            vec3 cd = (tap_1_1 + tap_m1_1 + tap_1_m1 + tap_m1_m1) * 0.25;
            vec3 dx_abs = abs(tap_1_0 - tap_m1_0);
            vec3 dy_abs = abs(tap_0_1 - tap_0_m1);
            float ce = (dx_abs.r + dx_abs.g + dx_abs.b + dy_abs.r + dy_abs.g + dy_abs.b) * 0.16666667;
            c = mix(c, (ca + cd) * 0.5, smoothstep(0.1, 0.2, ce) * 0.7);
        }
    #endif

    #ifdef ENABLE_SUBPIXEL_AA
        {
            float sh = abs(tap_1_0.g - c.g) + abs(tap_m1_0.g - c.g);
            float sv = abs(tap_0_1.g - c.g) + abs(tap_0_m1.g - c.g);
            float se = max(sh, sv);
            vec3 sc2 = mix((tap_1_0 + tap_m1_0) * 0.5,
                           (tap_0_1 + tap_0_m1) * 0.5,
                           step(sv, sh));
            vec3 sd2 = (tap_1_1 + tap_m1_1 + tap_1_m1 + tap_m1_m1) * 0.25;
            c = mix(c, mix(sc2, sd2, 0.3), min(smoothstep(0.1, 0.2, se), 0.75));
        }
    #endif

    #ifdef ENABLE_CONTRAST_ADAPTIVE_SHARPEN
        {
            vec3 mn = min(box_min_3x3, c);
            vec3 mx = max(box_max_3x3, c);
            vec3 amp = sqrt(clamp(min(mn, 2.0 - mx) / max(mx, vec3(0.0001)), 0.0, 1.0));
            vec3 w = -(amp * 0.1625);
            c = clamp((cross_sum * w + c) / max(w * 4.0 + ONE3, vec3(0.0001)), 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_ROBUST_CONTRAST_SHARPEN
        c = clamp(c + (c * 4.0 - cross_sum) * 0.0625, min(box_min_x, c), max(box_max_x, c));
    #endif

    #ifdef ENABLE_EDGE_DIRECTED_SHARPEN
        {
            float eg2 = lgrad_x * lgrad_x + lgrad_y * lgrad_y;
            float ew = 0.5 * clamp(eg2 * 64.0, 0.0, 1.0);
            c = clamp(c + (c * 4.0 - cross_sum) * ew * 0.25, 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_LAPLACIAN_SHARPEN
        c = clamp(c + (c - cross_avg) * 0.5, 0.0, 1.0);
    #endif

    #ifdef ENABLE_LUMINANCE_SHARPEN
        {
            float ll = dot(c, LUMA_BT601);
            float la = dot(cross_avg, LUMA_BT601);
            float ld = clamp(ll - la, -0.1, 0.1);
            c = c + vec3(ld);
        }
    #endif

    #ifdef ENABLE_MIDTONE_CLARITY
        {
            float ml = dot(c, LUMA_AVG);
            float mm = 1.0 - abs(ml * 2.0 - 1.0);
            c = c + (c - cross_avg) * 0.5 * mm;
        }
    #endif

    #ifdef ENABLE_FALLOFF_SHARPEN
        {
            vec3 ab = cross_sum * 0.2 + corner_sum * 0.05;
            vec3 ad = c - ab;
            float ae = abs(ad.r) + abs(ad.g) + abs(ad.b);
            c = c + ad * (0.6 / max(1.0 + ae * 4.0, 0.0001));
        }
    #endif

    #ifdef ENABLE_POWER_CURVE_SHARPEN
        {
            vec3 pd = c - cross_avg;
            vec3 ap = abs(pd);
            c = c + sign(pd) * (ap / (ap + vec3(0.3))) * 0.65;
        }
    #endif

    #ifdef ENABLE_UNSHARP_MASK
        {
            vec3 ub = (c * 4.0 + cross_sum + corner_sum * 0.5) * 0.1;
            c = c + (c - ub) * 0.5;
        }
    #endif

    #ifdef ENABLE_LOCAL_CONTRAST
        {
            float ll = dot(c, LUMA_AVG);
            float la = dot(cross_avg, LUMA_AVG);
            c = c * (1.0 + (ll - la) * 0.3 / max(ll, 0.0001));
        }
    #endif

    #ifdef ENABLE_GAUSSIAN_BLUR
        c = (c * 4.0 + cross_sum * 2.0 + corner_sum) * 0.0625;
    #endif

    #ifdef ENABLE_BOX_BLUR
        {
            float br = res_scale;
            vec2 hp = vec2(br * 0.5 + 0.5) * inv;
            vec3 k0 = texture(u_input, v_uv + vec2( hp.x,  hp.y)).rgb;
            vec3 k1 = texture(u_input, v_uv + vec2( hp.x, -hp.y)).rgb;
            vec3 k2 = texture(u_input, v_uv + vec2(-hp.x,  hp.y)).rgb;
            vec3 k3 = texture(u_input, v_uv + vec2(-hp.x, -hp.y)).rgb;
            vec2 hp2 = vec2(br * 1.5 + 0.5) * inv;
            vec3 k4 = texture(u_input, v_uv + vec2( hp2.x,  hp2.y)).rgb;
            vec3 k5 = texture(u_input, v_uv + vec2( hp2.x, -hp2.y)).rgb;
            vec3 k6 = texture(u_input, v_uv + vec2(-hp2.x,  hp2.y)).rgb;
            vec3 k7 = texture(u_input, v_uv + vec2(-hp2.x, -hp2.y)).rgb;
            c = (c + k0 + k1 + k2 + k3 + k4 + k5 + k6 + k7) * 0.11111111;
        }
    #endif

    #ifdef ENABLE_BOKEH_BLUR
        {
            float kr = 4.0 * res_scale;
            vec2 d0 = vec2( 0.8535534,  0.3535534) * kr * inv;
            vec2 d1 = vec2(-0.3535534,  0.8535534) * kr * inv;
            vec2 d2 = vec2(-0.8535534, -0.3535534) * kr * inv;
            vec2 d3 = vec2( 0.3535534, -0.8535534) * kr * inv;
            vec3 k0a = texture(u_input, v_uv + d0).rgb;
            vec3 k0b = texture(u_input, v_uv - d0).rgb;
            vec3 k1a = texture(u_input, v_uv + d1).rgb;
            vec3 k1b = texture(u_input, v_uv - d1).rgb;
            vec3 k2a = texture(u_input, v_uv + d2).rgb;
            vec3 k2b = texture(u_input, v_uv - d2).rgb;
            vec3 k3a = texture(u_input, v_uv + d3).rgb;
            vec3 k3b = texture(u_input, v_uv - d3).rgb;
            float w0a = 1.0 + dot(k0a, ONE3) * 0.5;
            float w0b = 1.0 + dot(k0b, ONE3) * 0.5;
            float w1a = 1.0 + dot(k1a, ONE3) * 0.5;
            float w1b = 1.0 + dot(k1b, ONE3) * 0.5;
            float w2a = 1.0 + dot(k2a, ONE3) * 0.5;
            float w2b = 1.0 + dot(k2b, ONE3) * 0.5;
            float w3a = 1.0 + dot(k3a, ONE3) * 0.5;
            float w3b = 1.0 + dot(k3b, ONE3) * 0.5;
            vec3 ks = c + k0a*w0a + k0b*w0b + k1a*w1a + k1b*w1b
                        + k2a*w2a + k2b*w2b + k3a*w3a + k3b*w3b;
            float kn = 1.0 + w0a + w0b + w1a + w1b + w2a + w2b + w3a + w3b;
            c = ks / max(kn, 0.0001);
        }
    #endif

    #ifdef ENABLE_TILT_SHIFT_BLUR
        {
            float tr = 2.0 * res_scale;
            float td = clamp((abs(v_uv.y - 0.5) - 0.15) / 0.2, 0.0, 1.0);
            float tx = tr * td * inv.x;
            vec3 ts = c
                + texture(u_input, v_uv + vec2(tx, 0.0)).rgb
                + texture(u_input, v_uv - vec2(tx, 0.0)).rgb
                + texture(u_input, v_uv + vec2(tx * 2.0, 0.0)).rgb
                + texture(u_input, v_uv - vec2(tx * 2.0, 0.0)).rgb
                + texture(u_input, v_uv + vec2(tx * 3.0, 0.0)).rgb
                + texture(u_input, v_uv - vec2(tx * 3.0, 0.0)).rgb
                + texture(u_input, v_uv + vec2(tx * 4.0, 0.0)).rgb
                - texture(u_input, v_uv - vec2(tx * 4.0, 0.0)).rgb;
            c = mix(c, ts * 0.1111111, td);
        }
    #endif

    #ifdef ENABLE_RADIAL_BLUR
        {
            vec2 rd = (HALF2 - v_uv) * 0.2;
            vec3 rs = c
                + texture(u_input, v_uv + rd * 0.14285714).rgb
                + texture(u_input, v_uv + rd * 0.28571429).rgb
                + texture(u_input, v_uv + rd * 0.42857143).rgb
                + texture(u_input, v_uv + rd * 0.57142857).rgb
                + texture(u_input, v_uv + rd * 0.71428571).rgb
                + texture(u_input, v_uv + rd * 0.85714286).rgb
                + texture(u_input, v_uv + rd).rgb;
            c = mix(c, rs * 0.125, 0.5);
        }
    #endif

    #ifdef ENABLE_GRADIENT_DEBAND
        {
            float dr = 8.0 * res_scale;
            vec2 t2 = vec2(u_time);
            float dh1 = hash21(v_uv + t2);
            float dh2 = hash21(v_uv + t2 + 17.31);
            vec2 ddir = vec2(dh1, dh2) * 2.0 - 1.0;
            ddir = ddir / max(length(ddir), 0.0001);
            vec3 ds = texture(u_input, v_uv + ddir * dr * inv).rgb;
            vec3 dd = abs(ds - c);
            float dm = step(max(dd.r, max(dd.g, dd.b)), 0.02);
            c = mix(c, (c + ds) * 0.5, dm);
        }
    #endif

    #ifdef ENABLE_THRESHOLD_BLOOM
        {
            const vec3 THR_BLOOM = vec3(0.7);
            vec3 bs = max(tap_1_0 - THR_BLOOM, ZERO3) * 2.0 +
                      max(tap_m1_0 - THR_BLOOM, ZERO3) * 2.0 +
                      max(tap_0_1 - THR_BLOOM, ZERO3) * 2.0 +
                      max(tap_0_m1 - THR_BLOOM, ZERO3) * 2.0 +
                      max(tap_1_1 - THR_BLOOM, ZERO3) +
                      max(tap_m1_1 - THR_BLOOM, ZERO3) +
                      max(tap_1_m1 - THR_BLOOM, ZERO3) +
                      max(tap_m1_m1 - THR_BLOOM, ZERO3);
            c = c + bs * 0.05;
        }
    #endif

    #ifdef ENABLE_GHOST_FLARE
        {
            const vec3 THR_GHOST = vec3(0.7);
            vec2 gc = HALF2 - v_uv;
            vec2 gc_step = gc * 0.2;
            vec3 gg = max(texture(u_input, v_uv + gc_step).rgb - THR_GHOST, ZERO3)
                    + max(texture(u_input, v_uv + gc_step * 2.0).rgb - THR_GHOST, ZERO3)
                    + max(texture(u_input, v_uv + gc_step * 3.0).rgb - THR_GHOST, ZERO3);
            c = c + gg * 0.4 * (1.0 - clamp(dot(gc, gc), 0.0, 1.0));
        }
    #endif

    #ifdef ENABLE_NEIGHBORHOOD_CLAMP_AA
        {
            vec3 tmn = min(box_min_3x3, c);
            vec3 tmx = max(box_max_3x3, c);
            c = mix(c, mix(c, clamp(history, tmn, tmx), 0.9), hist_valid);
        }
    #endif

    #ifdef ENABLE_MOTION_REJECT_DENOISE
        {
            vec3 td = abs(c - history);
            float tm = max(td.r, max(td.g, td.b));
            c = mix(c, mix(c, history, 0.8 * (1.0 - clamp(tm * 8.0, 0.0, 1.0))), hist_valid);
        }
    #endif

    #ifdef ENABLE_MOTION_DETECT_BLUR
        {
            vec3 md = abs(c - history);
            float mm = clamp(max(md.r, max(md.g, md.b)) * 8.0, 0.0, 1.0);
            c = mix(c, mix(history, c, mix(1.0, 0.5, mm)), hist_valid);
        }
    #endif

    #ifdef ENABLE_CONSTANT_BLEND_SMOOTH
        c = mix(c, mix(c, history, 0.5), hist_valid);
    #endif

    #ifdef ENABLE_SHUTTER_ANGLE_SMOOTH
        {
            vec3 sd = c - history;
            float sm = clamp(dot(sd, sd) * 30.0, 0.0, 1.0);
            float sw = 0.5 * (1.0 - sm) + 0.15 * sm;
            c = mix(c, mix(c, history, sw), hist_valid);
        }
    #endif

    #ifdef ENABLE_SPLINE_INTERP_SMOOTH
        {
            vec3 p0 = history - (c - history);
            vec3 p3 = c + (c - history);
            vec3 m1 = 0.5 * (c - p0);
            vec3 m2 = 0.5 * (p3 - history);
            const float h00 =  0.5;
            const float h10 =  0.125;
            const float h01 =  0.5;
            const float h11 = -0.125;
            vec3 ch = h00 * history + h10 * m1 + h01 * c + h11 * m2;
            c = mix(c, ch, hist_valid);
        }
    #endif

    #ifdef ENABLE_VARIANCE_DECAY_SMOOTH
        {
            vec3 ed = c - history;
            float ev = dot(ed, ed);
            float ea = 0.6 / max(1.0 + ev * 50.0, 0.0001);
            c = mix(c, mix(c, history, clamp(ea, 0.0, 0.9)), hist_valid);
        }
    #endif

    #ifdef ENABLE_DUALRATE_SMOOTH
        {
            vec3 rm = (c + cross_sum) * 0.2;
            vec3 r0 = c - rm;
            vec3 r1 = tap_1_0 - rm;
            vec3 r2 = tap_m1_0 - rm;
            vec3 r3 = tap_0_1 - rm;
            vec3 r4 = tap_0_m1 - rm;
            float rv = (dot(r0, r0) + dot(r1, r1) + dot(r2, r2) + dot(r3, r3) + dot(r4, r4)) * 0.2;
            float rw = mix(0.7, 0.1, clamp(rv * 50.0, 0.0, 1.0));
            c = mix(c, mix(c, history, rw), hist_valid);
        }
    #endif

    #ifdef ENABLE_LUMINANCE_GATE_SMOOTH
        {
            float ll = dot(c, LUMA_BT709);
            float lw = mix(0.7, 0.15, smoothstep(0.0, 0.3, ll));
            c = mix(c, mix(c, history, lw), hist_valid);
        }
    #endif

    #ifdef ENABLE_CONTRAST_GATE_SMOOTH
        {
            float cl = dot(c, LUMA_BT709);
            float cn = dot(cross_avg, LUMA_BT709);
            float cw = mix(0.7, 0.1, clamp(abs(cl - cn) * 8.0, 0.0, 1.0));
            c = mix(c, mix(c, history, cw), hist_valid);
        }
    #endif

    #ifdef ENABLE_GRADIENT_GATE_SMOOTH
        {
            float gx2 = dot(grad_x_rgb, grad_x_rgb);
            float gy2 = dot(grad_y_rgb, grad_y_rgb);
            float ge = clamp((gx2 + gy2) * 144.0, 0.0, 1.0);
            c = mix(c, mix(c, history, 0.6 * (1.0 - ge)), hist_valid);
        }
    #endif

    #ifdef ENABLE_SIGMA_CLIP_SMOOTH
        {
            vec3 vm1 = (c + cross_sum + corner_sum) * 0.1111111;
            vec3 vm2 = (c * c + tap_1_0 * tap_1_0 + tap_m1_0 * tap_m1_0 +
                        tap_0_1 * tap_0_1 + tap_0_m1 * tap_0_m1 +
                        tap_1_1 * tap_1_1 + tap_m1_1 * tap_m1_1 +
                        tap_1_m1 * tap_1_m1 + tap_m1_m1 * tap_m1_m1) * 0.1111111;
            vec3 vsd = sqrt(max(vm2 - vm1 * vm1, ZERO3));
            vec3 vh = clamp(history, vm1 - vsd, vm1 + vsd);
            c = mix(c, mix(c, vh, 0.6), hist_valid);
        }
    #endif

    #ifdef ENABLE_MITCHELL_KERNEL_SMOOTH
        {
            const float MN_W_C  = 0.79012346;
            const float MN_W_AX = 0.04938272;
            const float MN_W_DI = 0.00308642;
            const float MN_W_SUM = MN_W_C + 4.0 * MN_W_AX + 4.0 * MN_W_DI;
            const float MN_W_NORM = 1.0 / MN_W_SUM;
            vec3 ms = (c * MN_W_C + cross_sum * MN_W_AX + corner_sum * MN_W_DI) * MN_W_NORM;
            c = mix(c, mix(ms, history, 0.5), hist_valid);
        }
    #endif

    #ifdef ENABLE_YCOCG_CLIP_SMOOTH
        {
            vec3 fh = ycocg_encode(history);
            vec3 fu = ycocg_encode(c);
            vec3 fe = ycocg_encode(tap_1_0);
            vec3 fw = ycocg_encode(tap_m1_0);
            vec3 fn = ycocg_encode(tap_0_1);
            vec3 fs = ycocg_encode(tap_0_m1);
            vec3 fmn = min(fu, min(min(fe, fw), min(fn, fs)));
            vec3 fmx = max(fu, max(max(fe, fw), max(fn, fs)));
            vec3 fcl = clamp(fh, fmn, fmx);
            vec3 fdc = abs(fh - fcl);
            float ff = 1.0 - clamp((fdc.x + fdc.y + fdc.z) * 2.0, 0.0, 1.0);
            c = mix(c, ycocg_decode(mix(fu, fcl, 0.7 * ff)), hist_valid);
        }
    #endif

    #ifdef ENABLE_BILATERAL_HISTORY_SMOOTH
        {
            vec3 bca = (c + cross_sum) * 0.2;
            vec3 bha = (history + hist_e + hist_w + hist_n + hist_s) * 0.2;
            vec3 bd = bca - bha;
            float bsm = 1.0 / (1.0 + dot(bd, bd) * 20.0);
            c = mix(c, mix(c, history, 0.5 * bsm), hist_valid);
        }
    #endif

    #ifdef ENABLE_PERCEPTUAL_CHROMA_SMOOTH
        {
            vec3 pu = perc_ycc(c);
            vec3 ph = perc_ycc(history);
            c = mix(c, perc_rgb(mix(pu, ph, vec3(0.4, 0.7, 0.7))), hist_valid);
        }
    #endif

    #ifdef ENABLE_FREQUENCY_SPLIT_SMOOTH
        {
            vec3 flo = (c + cross_sum) * 0.2;
            vec3 fhi = c - flo;
            vec3 fhl = (history + hist_e + hist_w + hist_n + hist_s) * 0.2;
            vec3 fhhi = history - fhl;
            vec3 ff_out = mix(flo, fhl, 0.7) + mix(fhi, fhhi, 0.2);
            c = mix(c, ff_out, hist_valid);
        }
    #endif

    #ifdef ENABLE_HORN_SCHUNCK_SMOOTH
        {
            float om = 4.0 * res_scale;
            float ol = dot(c, LUMA_AVG);
            float ot = ol - dot(history, LUMA_AVG);
            float od = lgrad_x * lgrad_x * 0.25 + lgrad_y * lgrad_y * 0.25 + 0.001;
            vec2 of2 = clamp(vec2(lgrad_x * 0.5, lgrad_y * 0.5) * (-ot / od), vec2(-om), vec2(om));
            vec3 hs_out = mix(c, texture(u_history, v_uv + of2 * inv).rgb, 0.5);
            c = mix(c, hs_out, hist_valid);
        }
    #endif

    #ifdef ENABLE_CONVERGENT_ACCUMULATE
        {
            vec3 ad = c - history;
            float am = dot(ad, ad);
            float as2 = 1.0 - clamp(am * 15.0, 0.0, 1.0);
            float aw = mix(0.1, 0.85, as2 * as2);
            c = mix(c, mix(c, history, aw), hist_valid);
        }
    #endif

    #ifdef ENABLE_DUALWARP_FLOW_SMOOTH
        {
            float fm = 4.0 * res_scale;
            float fl = dot(c, LUMA_AVG);
            float fx = lgrad_x * 0.5;
            float fy = lgrad_y * 0.5;
            float ft2 = fl - dot(history, LUMA_AVG);
            float fd2 = fx * fx + fy * fy + 0.001;
            vec2 ff = clamp(vec2(fx, fy) * (-ft2 / fd2), vec2(-fm), vec2(fm));
            vec2 fw_uv = v_uv + ff * inv;
            vec3 fa = texture(u_history, fw_uv).rgb;
            vec3 fa_e = texture(u_history, fw_uv + vec2( inv.x, 0.0)).rgb;
            vec3 fa_w = texture(u_history, fw_uv + vec2(-inv.x, 0.0)).rgb;
            vec3 fa_n = texture(u_history, fw_uv + vec2(0.0,  inv.y)).rgb;
            vec3 fa_s = texture(u_history, fw_uv + vec2(0.0, -inv.y)).rgb;
            float fx_w = dot(fa_e - fa_w, LUMA_AVG) * 0.5;
            float fy_w = dot(fa_n - fa_s, LUMA_AVG) * 0.5;
            float rl = dot(fa, LUMA_AVG);
            float rt = rl - fl;
            float rd2 = fx_w * fx_w + fy_w * fy_w + 0.001;
            vec2 rv = clamp(vec2(fx_w, fy_w) * (-rt / rd2), vec2(-fm), vec2(fm));
            vec2 sum_flow = ff + rv;
            float fc2 = 1.0 - clamp(dot(sum_flow, sum_flow) * 4.0, 0.0, 1.0);
            vec2 fhf = ff * 0.5 * inv;
            vec3 wc = texture(u_input, v_uv - fhf).rgb;
            vec3 wh = clamp(texture(u_history, v_uv + fhf).rgb, min(box_min_x, c), max(box_max_x, c));
            vec3 dd = abs(wc - wh);
            float ds = step(0.12, max(dd.r, max(dd.g, dd.b)));
            vec3 dw_out = mix(mix(wc, wh, 0.6 * fc2), c, ds);
            c = mix(c, dw_out, hist_valid);
        }
    #endif

    #ifdef ENABLE_VARIANCE_FLOW_ACCUMULATE
        {
            float dl = dot(c, LUMA_AVG);
            float dx = lgrad_x * 0.5;
            float dy = lgrad_y * 0.5;
            float dt = dl - dot(history, LUMA_AVG);
            float dd2 = dx * dx + dy * dy + 0.001;
            vec2 df = clamp(vec2(dx, dy) * (-dt / dd2),
                            vec2(-4.0 * res_scale), vec2(4.0 * res_scale));
            vec3 dw = texture(u_history, v_uv + df * inv).rgb;
            vec3 dm1 = (c + cross_sum) * 0.2;
            vec3 dm2 = (c * c + tap_1_0 * tap_1_0 + tap_m1_0 * tap_m1_0 +
                        tap_0_1 * tap_0_1 + tap_0_m1 * tap_0_m1) * 0.2;
            vec3 ds_ = sqrt(max(dm2 - dm1 * dm1, ZERO3));
            vec3 dc_ = clamp(dw, dm1 - ds_ * 1.25, dm1 + ds_ * 1.25);
            vec3 dcd = abs(dw - dc_);
            float dca = max(dcd.r, max(dcd.g, dcd.b));
            float dmc = 1.0 - clamp(dot(df, df) * 144.0 / max(res_scale * res_scale, 0.0001), 0.0, 1.0);
            float dlc = 1.0 - clamp(abs(dot(c, LUMA_BT709) - dot(dc_, LUMA_BT709)) * 8.0, 0.0, 1.0);
            float dcf = dmc * dlc * (1.0 - clamp(dca * 8.0, 0.0, 1.0));
            float vw = mix(0.05, 0.85, dcf * dcf);
            c = mix(c, mix(c, dc_, vw), hist_valid);
        }
    #endif

    #ifdef ENABLE_EDGE_RECONSTRUCT_SMOOTH
        {
            float egx = dot(grad_x_rgb, LUMA_BT709);
            float egy = dot(grad_y_rgb, LUMA_BT709);
            float egm2 = egx * egx + egy * egy;
            vec2 egd = vec2(-egy, egx) / max(sqrt(egm2), 0.0001);
            vec3 edp = texture(u_input, v_uv + egd * inv).rgb;
            vec3 edn = texture(u_input, v_uv - egd * inv).rgb;
            vec3 eda = (edp + edn) * 0.5;
            float edw = clamp(egm2 * 4.0, 0.0, 1.0);
            vec3 esp = mix(c, eda, edw * 0.3);
            float el = dot(c, LUMA_AVG);
            float elt = el - dot(history, LUMA_AVG);
            float edn2 = lgrad_x * lgrad_x * 0.25 + lgrad_y * lgrad_y * 0.25 + 0.001;
            vec2 ef = clamp(vec2(lgrad_x * 0.5, lgrad_y * 0.5) * (-elt / edn2),
                            vec2(-4.0 * res_scale), vec2(4.0 * res_scale));
            vec3 ew_ = clamp(texture(u_history, v_uv + ef * inv).rgb, min(box_min_x, c), max(box_max_x, c));
            vec3 edf = abs(esp - ew_);
            float ecf = 1.0 - clamp(max(edf.r, max(edf.g, edf.b)) * 10.0, 0.0, 1.0);
            float eoe = clamp(egm2 * 16.0, 0.0, 1.0);
            float ewt = mix(0.6, 0.08, 1.0 - ecf) * mix(1.0, 0.5, eoe);
            c = mix(c, mix(esp, ew_, ewt), hist_valid);
        }
    #endif

    #ifdef ENABLE_TEMPORAL_STABILIZER
        {
            vec3 dd_t = abs(c - history);
            float dm_t = max(dd_t.r, max(dd_t.g, dd_t.b));
            vec3 c1 = mix(c, history, 0.5 * (1.0 - step(0.15, dm_t)));
            vec3 td_t = history - c1;
            float tw_t = 0.3 * (1.0 - clamp(dot(td_t, td_t) * 10.0, 0.0, 1.0));
            vec3 c2 = clamp(mix(c1, history + td_t * tw_t, tw_t), 0.0, 1.0);
            c = mix(c, c2, hist_valid);
        }
    #endif

    #ifdef ENABLE_LINEAR_EXPOSURE
        c = c * 1.3;
    #endif

    #ifdef ENABLE_ACES_TONEMAP
        c = clamp((c * (2.51 * c + 0.03)) / max(c * (2.43 * c + 0.59) + 0.14, vec3(0.0001)), 0.0, 1.0);
    #endif

    #ifdef ENABLE_AGX_TONEMAP
        {
            const mat3 AGX_IN = mat3(
                0.842479062253094, 0.0423282422610123, 0.0423756549057051,
                0.0784335999999992, 0.878468636469772, 0.0784336,
                0.0792237451477643, 0.0791661274605434, 0.879142973793104
            );
            const mat3 AGX_OUT = mat3(
                 1.19687900512017,  -0.0528968517574562, -0.0529716355144438,
                -0.0980208811401368, 1.15190312990417,   -0.0980434501171241,
                -0.0990297440797205, -0.0989611768448433, 1.15107367264116
            );
            vec3 av = AGX_IN * c;
            av = clamp(log2(max(av, vec3(0.0001))), -12.47393, 4.026069);
            av = (av + 12.47393) * 0.06059967;
            av = agx_contrast(av);
            c = clamp(AGX_OUT * av, 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_REINHARD_TONEMAP
        c = c * (ONE3 + c * 0.0625) / (ONE3 + c);
    #endif

    #ifdef ENABLE_HABLE_TONEMAP
        {
            const float HABLE_W_INV = 1.0 / 0.81243248;
            c = clamp(hable_map(c * 2.0) * HABLE_W_INV, 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_LOTTES_TONEMAP
        {
            vec3 lx = max(c, vec3(0.0001));
            vec3 la = pow(lx, vec3(1.6));
            c = la / max(la * pow(lx, vec3(-0.112)) * 0.977 + vec3(0.18), vec3(0.0001));
        }
    #endif

    #ifdef ENABLE_UCHIMURA_TONEMAP
        c = uchi_map(c);
    #endif

    #ifdef ENABLE_TONY_TONEMAP
        {
            vec3 tx = max(c, ZERO3);
            c = clamp(tx / (tx + vec3(0.155)) * 1.19, 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_KHRONOS_TONEMAP
        {
            float kx = min(c.r, min(c.g, c.b));
            float ko = mix(0.04, kx - 6.25 * kx * kx, step(kx, 0.08));
            c -= ko;
            float kp = max(c.r, max(c.g, c.b));
            float knp = 1.0 - 0.0576 / max(kp + 0.24 - 0.76, 0.0001);
            float kgate = step(0.76, kp);
            c *= mix(1.0, knp / max(kp, 0.0001), kgate);
            float kg = 1.0 - 1.0 / max(0.15 * (kp - knp) + 1.0, 0.0001);
            c = clamp(mix(c, vec3(knp), kgate * kg), 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_NEUTRAL_WHITE_BALANCE
        c = c * vec3(0.985, 1.0, 1.030);
    #endif

    #ifdef ENABLE_WARM_TEMPERATURE
        c = c * vec3(1.05, 1.0, 0.92);
    #endif

    #ifdef ENABLE_COOL_TEMPERATURE
        c = c * vec3(0.92, 1.0, 1.05);
    #endif

    #ifdef ENABLE_SATURATION_CONTRAST_GRADE
        {
            float gl = dot(c, LUMA_BT601);
            c = mix(vec3(gl), c, 1.1);
            c = (c - HALF3) * 1.05 + HALF3;
        }
    #endif

    #ifdef ENABLE_LEVELS_REMAP
        c = clamp((c - vec3(0.02)) * 1.04166667, 0.0, 1.0);
    #endif

    #ifdef ENABLE_GAMMA_CORRECT
        c = pow(max(c, ZERO3), vec3(0.9090909));
    #endif

    #ifdef ENABLE_VIBRANCE_BOOST
        {
            float vmx = max(c.r, max(c.g, c.b));
            float vmn = min(c.r, min(c.g, c.b));
            float vl = dot(c, LUMA_BT601);
            c = mix(vec3(vl), c, 1.0 + 0.5 * (1.0 - (vmx - vmn)));
        }
    #endif

    #ifdef ENABLE_HSL_TRANSFORM
        {
            float hmx = max(c.r, max(c.g, c.b));
            float hmn = min(c.r, min(c.g, c.b));
            float hd = hmx - hmn;
            float hl = (hmx + hmn) * 0.5;
            vec3 d3 = vec3(hmx) - c;
            float invd = 1.0 / max(hd, 1e-5);
            vec3 hcand = vec3(
                (d3.g - d3.b) * invd,
                (d3.b - d3.r) * invd + 2.0,
                (d3.r - d3.g) * invd + 4.0
            );
            float pick_r = step(hmx - 1e-5, c.r);
            float pick_g = step(hmx - 1e-5, c.g) * (1.0 - pick_r);
            float pick_b = (1.0 - pick_r) * (1.0 - pick_g);
            float hue_raw = hcand.x * pick_r + hcand.y * pick_g + hcand.z * pick_b;
            float hue = fract(hue_raw * 0.16666667) * step(1e-5, hd);
            float hs = clamp(hd / max(1.0 - abs(2.0 * hl - 1.0), 0.0001) * 1.1, 0.0, 1.0);
            float hl2 = clamp(hl, 0.0, 1.0);
            float hc = (1.0 - abs(2.0 * hl2 - 1.0)) * hs;
            float hm = hl2 - hc * 0.5;
            vec3 hbase = clamp(abs(mod(hue * 6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0);
            c = hbase * hc + vec3(hm);
        }
    #endif

    #ifdef ENABLE_SPLIT_TONE
        {
            float sl = dot(c, LUMA_BT601);
            c = c + mix(vec3(-0.1, 0.0, 0.1), vec3(0.1, 0.0, -0.1), smoothstep(0.0, 1.0, sl)) * 0.3;
        }
    #endif

    #ifdef ENABLE_LIFT_GAMMA_GAIN
        c = max(c + vec3(0.03) * (ONE3 - c), ZERO3);
    #endif

    #ifdef ENABLE_HERMITE_CURVES
        {
            vec3 cv = c * c * (3.0 - 2.0 * c);
            c = mix(c, cv, 0.5);
        }
    #endif

    #ifdef ENABLE_RED_CHANNEL_CURVE
        {
            float rv = c.r * c.r * (3.0 - 2.0 * c.r);
            c.r = mix(c.r, rv, 0.5);
        }
    #endif

    #ifdef ENABLE_GREEN_CHANNEL_CURVE
        {
            float gv = c.g * c.g * (3.0 - 2.0 * c.g);
            c.g = mix(c.g, gv, 0.5);
        }
    #endif

    #ifdef ENABLE_BLUE_CHANNEL_CURVE
        {
            float bv = c.b * c.b * (3.0 - 2.0 * c.b);
            c.b = mix(c.b, bv, 0.5);
        }
    #endif

    #ifdef ENABLE_TRIZONE_COLOR_BALANCE
        {
            float cl = dot(c, LUMA_BT601);
            float cs = 1.0 - smoothstep(0.0, 0.4, cl);
            float ch = smoothstep(0.6, 1.0, cl);
            c = c + vec3(-0.03, 0.01, 0.02) * cs + vec3(0.02, 0.0, -0.02) * ch;
        }
    #endif

    #ifdef ENABLE_RED_SELECTIVE_SATURATE
        {
            float mx = max(c.r, max(c.g, c.b));
            float st = mx - min(c.r, min(c.g, c.b));
            c = c * (1.0 + step(abs(c.r - mx), 1e-5) * 0.3 * st);
        }
    #endif

    #ifdef ENABLE_GREEN_SELECTIVE_SATURATE
        {
            float mx = max(c.r, max(c.g, c.b));
            float st = mx - min(c.r, min(c.g, c.b));
            float ir = step(abs(c.r - mx), 1e-5);
            float ig = step(abs(c.g - mx), 1e-5) * (1.0 - ir);
            c = c * (1.0 + ig * 0.3 * st);
        }
    #endif

    #ifdef ENABLE_BLUE_SELECTIVE_SATURATE
        {
            float mx = max(c.r, max(c.g, c.b));
            float st = mx - min(c.r, min(c.g, c.b));
            float ir = step(abs(c.r - mx), 1e-5);
            float ig = step(abs(c.g - mx), 1e-5) * (1.0 - ir);
            float ib = step(abs(c.b - mx), 1e-5) * (1.0 - ir) * (1.0 - ig);
            c = c * (1.0 + ib * 0.3 * st);
        }
    #endif

    #ifdef ENABLE_DYNAMIC_RANGE_CRUSH
        c = (clamp(c, vec3(0.03), vec3(0.97)) - vec3(0.03)) * 1.06382979;
    #endif

    #ifdef ENABLE_DUOTONE_MAP
        {
            float dl = dot(c, LUMA_BT601);
            c = mix(vec3(0.1, 0.1, 0.3), vec3(1.0, 0.9, 0.7), dl);
        }
    #endif

    #ifdef ENABLE_COLOR_WASH_TINT
        c = mix(c, vec3(0.5, 0.5, 0.6), 0.2);
    #endif

    #ifdef ENABLE_POSTERIZE_QUANTIZE
        c = floor(c * 8.0) * 0.125;
    #endif

    #ifdef ENABLE_BLEACH_BYPASS
        {
            float bl = dot(c, LUMA_BT601);
            vec3 bb = 2.0 * c * vec3(bl);
            vec3 bs = ONE3 - 2.0 * (ONE3 - c) * (ONE3 - vec3(bl));
            c = mix(c, mix(bb, bs, step(0.5, bl)), 0.7);
        }
    #endif

    #ifdef ENABLE_TECHNICOLOR_PROCESS
        {
            vec3 tn = ONE3 - c;
            vec3 tp = vec3(tn.g + tn.b, tn.r + tn.b, tn.r + tn.g) * 0.5;
            c = mix(c, clamp(ONE3 - tp * 0.5, 0.0, 1.0), 0.7);
        }
    #endif

    #ifdef ENABLE_MIDPOINT_CONTRAST
        c = clamp((c - HALF3) * 1.5 + HALF3, 0.0, 1.0);
    #endif

    #ifdef ENABLE_COLOR_INVERT
        c = ONE3 - c;
    #endif

    #ifdef ENABLE_LUMINANCE_GRAYSCALE
        c = vec3(dot(c, LUMA_BT709));
    #endif

    #ifdef ENABLE_PROTANOPIA_SIMULATION
        c = vec3(dot(c, vec3(0.567, 0.433, 0.0)),
                 dot(c, vec3(0.558, 0.442, 0.0)),
                 dot(c, vec3(0.0, 0.242, 0.758)));
    #endif

    #ifdef ENABLE_DEUTERANOPIA_SIMULATION
        c = vec3(dot(c, vec3(0.625, 0.375, 0.0)),
                 dot(c, vec3(0.7, 0.3, 0.0)),
                 dot(c, vec3(0.0, 0.3, 0.7)));
    #endif

    #ifdef ENABLE_TRITANOPIA_SIMULATION
        c = vec3(dot(c, vec3(0.95, 0.05, 0.0)),
                 dot(c, vec3(0.0, 0.433, 0.567)),
                 dot(c, vec3(0.0, 0.475, 0.525)));
    #endif

    #ifdef ENABLE_PROTANOPIA_CORRECT
        {
            vec3 ds = vec3(dot(c, vec3(0.567, 0.433, 0.0)),
                           dot(c, vec3(0.558, 0.442, 0.0)),
                           dot(c, vec3(0.0, 0.242, 0.758)));
            vec3 de = c - ds;
            c = clamp(c + vec3(0.0, de.r * 0.7, de.r * 0.7), 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_DEUTERANOPIA_CORRECT
        {
            vec3 ds = vec3(dot(c, vec3(0.625, 0.375, 0.0)),
                           dot(c, vec3(0.7, 0.3, 0.0)),
                           dot(c, vec3(0.0, 0.3, 0.7)));
            vec3 de = c - ds;
            c = clamp(c + vec3(0.0, de.r * 0.7, de.r * 0.7), 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_TRITANOPIA_CORRECT
        {
            vec3 ds = vec3(dot(c, vec3(0.95, 0.05, 0.0)),
                           dot(c, vec3(0.0, 0.433, 0.567)),
                           dot(c, vec3(0.0, 0.475, 0.525)));
            vec3 de = c - ds;
            c = clamp(c + vec3(de.b * 0.7, de.b * 0.7, 0.0), 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_GAUSSIAN_GRAIN
        {
            float g1 = max(hash21(v_uv + vec2(u_time * 10.0)), 0.0001);
            float g2 = hash21(v_uv + vec2(u_time * 10.0 + 7.31));
            float gg = sqrt(-2.0 * log(g1)) * fast_cos(6.2831853 * g2);
            c = c + vec3(gg) * 0.05;
        }
    #endif

    #ifdef ENABLE_RED_HALATION
        {
            const float HAL_THR = 0.6;
            float hal_sum_r = tap_1_1.r + tap_m1_1.r + tap_1_m1.r + tap_m1_m1.r;
            c.r = c.r + max(hal_sum_r * 0.25 - HAL_THR, 0.0) * 0.5;
        }
    #endif

    #ifdef ENABLE_ANAMORPHIC_STREAK
        {
            const vec3 ANA_THR = vec3(0.7);
            float ax_unit = 6.0 * res_scale * inv.x;
            float as_x  = ax_unit * 1.5;
            float as_x2 = ax_unit * 3.5;
            float as_x3 = ax_unit * 5.5;
            vec3 a1p = texture(u_input, v_uv + vec2( as_x, 0.0)).rgb;
            vec3 a1m = texture(u_input, v_uv + vec2(-as_x, 0.0)).rgb;
            vec3 a2p = texture(u_input, v_uv + vec2( as_x2, 0.0)).rgb;
            vec3 a2m = texture(u_input, v_uv + vec2(-as_x2, 0.0)).rgb;
            vec3 a3p = texture(u_input, v_uv + vec2( as_x3, 0.0)).rgb;
            vec3 a3m = texture(u_input, v_uv + vec2(-as_x3, 0.0)).rgb;
            vec3 as2 = (max(a1p - ANA_THR, ZERO3) + max(a1m - ANA_THR, ZERO3)) * 1.5
                     + (max(a2p - ANA_THR, ZERO3) + max(a2m - ANA_THR, ZERO3)) * 0.58333333
                     + (max(a3p - ANA_THR, ZERO3) + max(a3m - ANA_THR, ZERO3)) * 0.36666667;
            c = c + as2 * 0.3 * vec3(0.4, 0.5, 1.0);
        }
    #endif

    #ifdef ENABLE_RADIAL_VIGNETTE
        {
            vec2 vd = v_uv - HALF2;
            c = c * (1.0 - smoothstep(0.09, 0.64, dot(vd, vd)) * 0.5);
        }
    #endif

    #ifdef ENABLE_CINEMATIC_LETTERBOX
        {
            float la = u_resolution.x / max(u_resolution.y, 0.0001);
            float lv = clamp(la * 0.42553191, 0.0, 1.0);
            float lb = (1.0 - lv) * 0.5;
            c = mix(c, ZERO3, step(v_uv.y, lb) + step(1.0 - lb, v_uv.y));
        }
    #endif

    #ifdef ENABLE_ORDERED_DITHER
        c = c + vec3(bayer_signed(frag_coord)) * 0.01;
    #endif

    c = clamp(c, 0.0, 1.0);

    #ifdef ENABLE_PS1_SIMULATION
        c = floor(c * 31.0 + 0.5 + bayer_signed(frag_coord) * 0.025) * 0.03225806;
    #endif

    #ifdef ENABLE_SATURN_SIMULATION
        {
            float sat_lum = dot(c, LUMA_BT601);
            float sat_qlum = floor(sat_lum * 12.0 + 0.5) * 0.08333333;
            c = c * (sat_qlum / max(sat_lum, 0.0001));
            c = c * 0.85;
            float sat_g = dot(c, LUMA_BT601);
            c = mix(vec3(sat_g), c, 0.75);
            c = c * vec3(1.05, 0.97, 0.85);
            c = floor(c * 31.0 + 0.5 + bayer_signed(frag_coord) * 0.02) * 0.03225806;
        }
    #endif

    #ifdef ENABLE_N64_SIMULATION
        {
            vec2 n64_d2 = v_uv - HALF2;
            c = mix(c, vec3(0.6, 0.65, 0.75), smoothstep(0.04, 0.5625, dot(n64_d2, n64_d2)) * 0.25);
            c = c * vec3(1.06, 1.02, 0.9);
            c = floor(c * 31.0 + 0.5 + bayer_signed(frag_coord) * 0.015) * 0.03225806;
        }
    #endif

    #ifdef ENABLE_DREAMCAST_SIMULATION
        {
            c = c * 1.15;
            float dc_lum = dot(c, LUMA_BT601);
            c = mix(vec3(dc_lum), c, 1.12);
            c = c + vec3(smoothstep(0.65, 0.95, dc_lum)) * 0.08;
        }
    #endif

    #ifdef ENABLE_PS2_SIMULATION
        {
            const vec3 PS2_THR = vec3(0.6);
            float ps2_scanline = floor(gl_FragCoord.y);
            float ps2_sc = 0.97 + 0.03 * (abs(fract(ps2_scanline * 0.5) - 0.5) * 4.0 - 1.0);
            c = c * ps2_sc;
            c = c + max(c - PS2_THR, ZERO3) * 0.15;
        }
    #endif

    #ifdef ENABLE_XBOX_SIMULATION
        {
            float xb_lum = dot(c, LUMA_BT601);
            c = c + vec3(smoothstep(0.6, 0.9, xb_lum)) * 0.1;
            c = c + max(c - HALF3, ZERO3) * 0.12;
            c = c * vec3(1.02, 1.04, 0.96);
        }
    #endif

    #ifdef ENABLE_PSP_SIMULATION
        {
            float psp_lum = dot(c, LUMA_BT601);
            float psp_qlum = floor(psp_lum * 16.0 + 0.5) * 0.0625;
            c = c * (psp_qlum / max(psp_lum, 0.0001));
            c = mix(vec3(0.06), vec3(0.92), c);
            float psp_dl = dot(c, LUMA_BT601);
            float psp_dark_q = mix(8.0, 31.0, smoothstep(0.0, 0.3, psp_dl));
            c = floor(c * psp_dark_q + 0.5) / psp_dark_q;
            c = floor(c * 31.0 + 0.5 + bayer_signed(frag_coord) * 0.018) * 0.03225806;
            float psp_g = dot(c, LUMA_BT601);
            c = mix(vec3(psp_g), c, 0.88);
        }
    #endif

    #ifdef ENABLE_PS3_SIMULATION
        {
            float ps3_lum = dot(c, LUMA_BT601);
            c = c * smoothstep(0.0, 0.06, ps3_lum);
            c = c * vec3(0.98, 1.0, 1.03);
        }
    #endif

    #ifdef ENABLE_XBOX360_SIMULATION
        {
            float x3_s1 = 1.0 - smoothstep(0.0, 0.003, abs(v_uv.y - 0.333));
            float x3_s2 = 1.0 - smoothstep(0.0, 0.003, abs(v_uv.y - 0.667));
            c = c + vec3(0.015) * (x3_s1 + x3_s2);
            float x3_lum = dot(c, LUMA_BT601);
            float x3_steps = mix(256.0, 24.0, smoothstep(0.5, 0.9, x3_lum));
            float x3_qlum = floor(x3_lum * x3_steps + 0.5) / x3_steps;
            c = c * (x3_qlum / max(x3_lum, 0.0001));
            c = c + vec3(0.04) * (ONE3 - c);
            c = c * vec3(1.03, 1.01, 0.96);
            float x3_g = dot(c, LUMA_BT601);
            c = mix(vec3(x3_g), c, 0.92);
            c = c + vec3(smoothstep(0.55, 0.85, x3_g)) * 0.06;
        }
    #endif

    #ifdef ENABLE_CRT_SIMULATION
        {
            float cp = fract(v_uv.x * u_resolution.x * 0.3333333);
            float mr = mix(0.7, 1.0, step(cp, 0.3333333));
            float mg = mix(0.7, 1.0, step(0.3333333, cp) * step(cp, 0.6666666));
            float mb = mix(0.7, 1.0, step(0.6666666, cp));
            float sc = 1.0 - 0.25 * (0.5 + 0.5 * (abs(fract(v_uv.y * u_resolution.y * 0.5) - 0.5) * 4.0 - 1.0));
            float crt_bright = 1.0 + fast_sin(u_time * 1.7) * 0.04 + fast_sin(u_time * 0.3) * 0.02;
            c = c * vec3(mr, mg, mb) * sc * 1.08 * crt_bright;
        }
    #endif

    #ifdef ENABLE_PHOSPHOR_AMBER
        c = vec3(1.0, 0.75, 0.3) * dot(c, LUMA_BT601) * 1.1;
    #endif

    #ifdef ENABLE_PHOSPHOR_GREEN
        c = vec3(0.2, 1.0, 0.2) * dot(c, LUMA_BT601) * 1.1;
    #endif

    #ifdef ENABLE_PHOSPHOR_RED
        c = vec3(1.0, 0.2, 0.2) * dot(c, LUMA_BT601) * 1.1;
    #endif

    #ifdef ENABLE_SCANLINE_DARKEN
        {
            float tri = abs(fract(v_uv.y * u_resolution.y * 0.5) - 0.5) * 2.0;
            c = c * (1.0 - 0.3 * tri);
        }
    #endif

    #ifdef ENABLE_OLED_SIMULATION
        {
            float ol = dot(c, LUMA_BT601);
            float oc = smoothstep(0.0, 0.05, ol);
            c = (c + (c - vec3(ol)) * 0.1) * oc;
        }
    #endif

    #ifdef ENABLE_VHS_SIMULATION
        {
            float vline = floor(v_uv.y * u_resolution.y);
            float vhs_tn = hash21(vec2(vline, floor(u_time * 3.0) + 0.5)) - 0.5;
            c = c + vec3(vhs_tn * 0.09);
            float vn = hash21(v_uv + vec2(u_time * 10.0)) - 0.5;
            c = c + vec3(vn) * 0.07;
            float vhs_g = dot(c, LUMA_BT601);
            c = mix(vec3(vhs_g), c, 0.7);
            c = c * vec3(1.04, 1.0, 0.88);
            float vhs_dh = hash21(vec2(vline, floor(u_time * 30.0)));
            float vhs_dv = step(0.985, vhs_dh);
            c = mix(c, vec3(step(0.5, fract(vhs_dh * 3.17))), vhs_dv * 0.7);
        }
    #endif

    #ifdef ENABLE_LCD_SUBPIXEL
        {
            float lc = 3.0 * res_scale;
            vec2 lp = fract(frag_coord / lc);
            float lm = step(0.2, lp.x) * step(0.2, lp.y);
            c = c * mix(0.6, 1.0, lm) * 1.1;
        }
    #endif

    #ifdef ENABLE_FPS_HUD
        {
            float hs = 26.0 * res_scale;
            vec2 hu = v_uv;
            vec2 raw_hp = (hu - vec2(0.012)) * u_resolution / hs;
            float in_x = step(-0.6, raw_hp.x) * step(raw_hp.x, 3.5);
            float in_y = step(-0.6, raw_hp.y) * step(raw_hp.y, 1.6);
            float in_box = in_x * in_y;
            vec2 hp = raw_hp;
            hp.x += (1.0 - hp.y) * 0.15;
            float fps_clamped = clamp(u_fps, 0.0, 9999.0);
            float fps_digit3 = mod(floor(fps_clamped * 0.001), 10.0);
            float fps_digit2 = mod(floor(fps_clamped * 0.01), 10.0);
            float fps_digit1 = mod(floor(fps_clamped * 0.1), 10.0);
            float fps_digit0 = mod(floor(fps_clamped), 10.0);
            float show3 = step(1.0, fps_digit3);
            float show2 = step(1.0, fps_digit3 + fps_digit2);
            float show1 = step(1.0, fps_digit3 + fps_digit2 + fps_digit1);
            float d3 = mix(999.0, hud_digit(hp, fps_digit3), show3);
            float d2 = mix(999.0, hud_digit(hp - vec2(0.7, 0.0), fps_digit2), show2);
            float d1 = mix(999.0, hud_digit(hp - vec2(1.4, 0.0), fps_digit1), show1);
            float d0 = hud_digit(hp - vec2(2.1, 0.0), fps_digit0);
            float d_text = min(min(d3, d2), min(d1, d0));
            float text_core = (1.0 - smoothstep(0.045, 0.075, d_text)) * in_box;
            float text_glow = (1.0 - smoothstep(0.06, 0.35, d_text)) * in_box;
            vec2 bg_d = abs(raw_hp - vec2(1.55, 0.45)) - vec2(1.4, 0.2);
            float bg_dist = length(max(bg_d, 0.0)) + min(max(bg_d.x, bg_d.y), 0.0);
            float bg_alpha = (1.0 - smoothstep(0.1, 0.5, bg_dist)) * 0.75 * in_box;
            c = mix(c, vec3(0.01, 0.02, 0.05), bg_alpha);
            c = mix(c, vec3(0.0, 0.5, 1.0), text_glow * 0.85);
            c = mix(c, vec3(0.85, 0.95, 1.0), text_core);
        }
    #endif

    #ifdef ENABLE_CROSSHAIR_OVERLAY
        {
            float cs = res_scale;
            vec2 cp = abs(frag_coord - u_resolution * 0.5);
            float in_box = step(max(cp.x, cp.y), 16.0 * cs);
            float d_dot = max(0.0, length(cp) - 0.5 * cs);
            vec2 pa_h = cp - vec2(4.0 * cs, 0.0);
            float h_h = clamp(pa_h.x / (10.0 * cs), 0.0, 1.0);
            float d_arm_h = length(pa_h - vec2(10.0 * cs * h_h, 0.0));
            vec2 pa_v = cp - vec2(0.0, 4.0 * cs);
            float h_v = clamp(pa_v.y / (10.0 * cs), 0.0, 1.0);
            float d_arm_v = length(pa_v - vec2(0.0, 10.0 * cs * h_v));
            float d_cross = min(d_dot, min(d_arm_h, d_arm_v));
            float cross_core = (1.0 - smoothstep(0.5 * cs, 1.5 * cs, d_cross)) * in_box;
            float cross_glow = (1.0 - smoothstep(1.0 * cs, 4.0 * cs, d_cross)) * in_box;
            c = mix(c, vec3(0.0, 0.5, 1.0), cross_glow * 0.85);
            c = mix(c, vec3(0.85, 0.95, 1.0), cross_core);
        }
    #endif

    BONES_WRITE_OUT(c);
}
"#;
