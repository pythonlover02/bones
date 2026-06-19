pub(crate) const LAYER_NAME: &str = "VK_LAYER_BONES_overlay";
pub(crate) const LAYER_DESC: &str = "Bones -> Linux GL/Vulkan shader post-FX";

pub(crate) const ENV_CONFIG_NAME: &str = "BONES_CONFIG_NAME";
pub(crate) const ENV_CONFIG: &str = "BONES_CONFIG";
pub(crate) const ENV_LOG: &str = "BONES_LOG";

pub(crate) const ENV_PRELOAD: &str = "LD_PRELOAD";
pub(crate) const ENV_VK_ADD_LAYER_PATH: &str = "VK_ADD_LAYER_PATH";
pub(crate) const ENV_VK_INSTANCE_LAYERS: &str = "VK_INSTANCE_LAYERS";

pub(crate) const INSTALL_LIB: &str = "/usr/local/lib/bones/libbones.so";
pub(crate) const INSTALL_DIR: &str = "/usr/local/lib/bones";
pub(crate) const DEV_LIB: &str = "target/release/libbones.so";
pub(crate) const DEV_DIR: &str = "target/release";

pub(crate) const FLATPAK_CMD: &str = "flatpak";
pub(crate) const FLATPAK_RUN: &str = "run";
pub(crate) const FLATPAK_INJECT: &str = "/usr/lib/extensions/vulkan/bones/bin/bones-flatpak";
pub(crate) const FLATPAK_META_KEY: &str = "command=";
pub(crate) const FLATPAK_INFO: &str = "info";
pub(crate) const FLATPAK_SHOW_META: &str = "--show-metadata";

pub(crate) const CONFIG_SEP: char = ';';
pub(crate) const ENV_SEP: char = ':';
pub(crate) const POLL_BLOCK: i32 = -1;
pub(crate) const HOT_RELOAD_KEY: &str = "hot_reload";

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

#[cfg(target_arch = "x86_64")]
pub(crate) const DLSYM_VERSION: &[u8] = b"GLIBC_2.2.5\0";
#[cfg(not(target_arch = "x86_64"))]
pub(crate) const DLSYM_VERSION: &[u8] = b"GLIBC_2.17\0";

pub(crate) const USAGE: &str = "usage: bones [PROFILE] -- COMMAND [ARGS...]\n  bones -- CMD            run CMD with the default profile (~/.config/bones/bones-config.toml)\n  bones NAME -- CMD       run CMD with profile ~/.config/bones/NAME-config.toml\n";

pub(crate) const HEAD: &str = r#"##bones default profile
# effect process in the order listed to ubershader pipeline: UV to Spatial to Temporal to Inline to Color
# pick ONE anti aliasing ONE tonemapper and ONE primary temporal smoother for best result

[general]
hot_reload = true

[geometric]
# warp texture coordinate before any pixel be sampled

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
#   edge magnitude with a soft threshold. more conservative than FXAA to
#   preserve more detail but catch fewer edge.
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

# power_curve_sharpen to filmic sharpen using a power curve response
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

# box_blur to uniform 5x5 box blur with resolution scaled radius. every
#   pixel in the 5x5 neighborhood contribute equally. more aggressive
#   and cheaper than gaussian_blur but produce a slightly blockier result.
box_blur = false

# bokeh_blur to circular 8 sample ring blur simulating shallow depth of
#   field (the blurry background you get with a wide aperture camera lens).
#   bright spot get extra weight to simulate the characteristic "bokeh
#   circle" of out of focus highlight.
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

[display_simulation]
# simulate various display hardware characteristic. useful for retro
# gaming aesthetic or matching specific display look.

# crt_simulation to full CRT television simulation: barrel warp distortion
#   RGB phosphor triad mask (the three colored dot that make up each CRT
#   pixel) horizontal scanline darkening and brightness compensation.
#   make modern game look like they run on a 90s TV.
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

# vhs_simulation to VHS videotape degradation: per scanline horizontal
#   jitter (tracking error) chroma channel separation (color bleed)
#   luma noise (static) and vertical ripple distortion. all distance
#   be resolution scaled. great for horror game or retro aesthetic.
vhs_simulation = false

# lcd_subpixel to visible LCD subpixel grid overlay. show the RGB
#   subpixel structure you would see examining an LCD screen with a magnifying
#   glass. cell size scale with resolution so it look correct at any
#   display resolution.
lcd_subpixel = false

[overlay]
# HUD element drawn on top of the processed image.

# fps_hud to performance overlay showing FPS as a uniquely
#   colored 7 segment display. Just measures FPS.
fps_hud = false

# crosshair_overlay to centered crosshair with gap arm and center dot
#   all resolution scaled. styled with a neon blue core and glow to
#   match the fps hud. useful for game that lack a built in crosshair
#   or when you want a consistent crosshair across different game.
crosshair_overlay = false

[temporal]
# blend current frame with previous frame to reduce flicker noise and judder.
# this be bones signature feature to 24 temporal processing mode from simple
# blending to motion compensated accumulation inspired by the research behind
# DLSS FSR and XeSS.
#
# pick ONE primary mode. stacking multiple temporal effect cause compounding
# ghosting. you CAN pair any primary mode with surface_disocclusion_guard
# (reduce ghosting) and/or convergent_detail_recovery (add sharpness back).
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

# shutter_angle_smooth to physical camera shutter simulation. a 180°
#   shutter angle mean the film/sensor be exposed for half the frame
#   duration naturally integrating two exposure. this be how real
#   cinema camera work and why 24fps film look smoother than 24fps
#   game to the motion blur be baked into each frame. produce the
#   most cinematically "correct" temporal blending.
shutter_angle_smooth = false

# spline_interp_smooth to Catmull Rom spline reconstruction between the
#   previous and current frame. unlike linear blending the spline curve
#   can overshoot slightly producing a sharper temporal response. the
#   Catmull Rom spline be the standard interpolation curve used in
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

# mitchell_kernel_smooth to Mitchell Netravali spatial reconstruction
#   kernel blended with temporal history. the Mitchell Netravali filter
#   (1988) be a cubic reconstruction filter used in high quality image
#   resampling (Pixar RenderMan Blender etc). the B parameter trade
#   sharpness for ringing. this combine that spatial quality with
#   temporal accumulation.
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
#   consistency check (estimate flow A to B then B to A; if they do not cancel
#   the flow be unreliable). clamp warped history to the cardinal
#   neighborhood AABB. fall back to the current frame on disocclusion.
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

# surface_disocclusion_guard to history rejection for newly revealed
#   surface. when the camera move or an object move previously hidden
#   pixel become visible. the history for those pixel be from a
#   completely different surface so blending with it create ghosting.
#   this detect the situation by thresholding the per pixel difference
#   between current and history and fall back to the current frame when
#   exceeded. pair with ANY temporal mode to reduce ghosting. the same
#   concept as the disocclusion handling in FSR 2 and DLSS 2.
surface_disocclusion_guard = false

# convergent_detail_recovery to temporal sharpening. the opposite of
#   smoothing: use converged history to ENHANCE detail in stable area.
#   when the history have accumulated over several stable frame it
#   contain more detail than any single frame (from sub pixel
#   accumulation). this pull the current frame toward that converged
#   representation effectively sharpening using temporal information.
#   motion rejected so it only activate in stable area. unique to
#   bones to no other reshade style tool do temporal sharpening.
convergent_detail_recovery = false

[inline]
# per pixel effect that sample raw input or operate on the processed
# color directly. run after temporal before color grading.

# gaussian_grain to film grain using the Box Muller transform to generate
#   true Gaussian distributed noise from two uniform pseudorandom input.
#   animated via u_time so the grain pattern change every frame like
#   real film grain. the Box Muller method be the standard way to generate
#   Gaussian noise in shader and scientific computing.
gaussian_grain = false

# chromatic_aberration to lateral chromatic aberration. shift the red
#   channel outward from center and the blue channel inward (or vice
#   versa) simulating the color fringing of a cheap camera lens that
#   cannot focus all wavelength to the same point. subtle amount add
#   realism; strong amount create a stylized look.
chromatic_aberration = false

# red_halation to halation glow. in real film bright highlight scatter
#   light through the film base and re expose the emulsion from behind
#   creating a red glow around bright object. this sample bright
#   neighbor and add their excess brightness to the red channel only.
#   distinctive warm glow that digital rendering lack.
red_halation = false

# anamorphic_streak to horizontal streak flare from bright highlight.
#   simulate the horizontal lens flare streak produced by anamorphic
#   (widescreen) cinema lens. 6 tap bilateral horizontal sampling with
#   brightness threshold tinted blue to match real anamorphic coating.
#   all distance resolution scaled.
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
#   midtone while compressing highlight and shadow.
hermite_curves = false

[channel_curves]
# per channel s curve for fine color control

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

# protanopia_simulate to simulate protanopia (red blind) vision using
#   a 3x3 color transformation matrix. show what the image look like
#   to someone with red cone deficiency (~1% of male).
protanopia_simulate = false

# deuteranopia_simulate to simulate deuteranopia (green blind) vision.
#   the most common color vision deficiency (~6% of male).
deuteranopia_simulate = false

# tritanopia_simulate to simulate tritanopia (blue blind) vision.
#   rare (~0.01% of population).
tritanopia_simulate = false

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
"#;

pub(crate) struct EffectDef {
    pub(crate) name: &'static str,
}

pub(crate) const REGISTRY: [EffectDef; 118] = [
    EffectDef { name: "identity" },
    EffectDef { name: "mirror_horizontal" },
    EffectDef { name: "mirror_vertical" },
    EffectDef { name: "rotate_90" },
    EffectDef { name: "rotate_180" },
    EffectDef { name: "rotate_270" },
    EffectDef { name: "center_zoom" },
    EffectDef { name: "polynomial_distort" },
    EffectDef { name: "barrel_undistort" },
    EffectDef { name: "fisheye_warp" },
    EffectDef { name: "trapezoid_warp" },
    EffectDef { name: "sharp_bilinear" },
    EffectDef { name: "bilateral_denoise" },
    EffectDef { name: "luma_edge_aa" },
    EffectDef { name: "normal_filter_aa" },
    EffectDef { name: "morphological_aa" },
    EffectDef { name: "subpixel_aa" },
    EffectDef { name: "contrast_adaptive_sharpen" },
    EffectDef { name: "robust_contrast_sharpen" },
    EffectDef { name: "edge_directed_sharpen" },
    EffectDef { name: "laplacian_sharpen" },
    EffectDef { name: "luminance_sharpen" },
    EffectDef { name: "midtone_clarity" },
    EffectDef { name: "falloff_sharpen" },
    EffectDef { name: "power_curve_sharpen" },
    EffectDef { name: "unsharp_mask" },
    EffectDef { name: "local_contrast" },
    EffectDef { name: "gaussian_blur" },
    EffectDef { name: "box_blur" },
    EffectDef { name: "bokeh_blur" },
    EffectDef { name: "tilt_shift_blur" },
    EffectDef { name: "radial_blur" },
    EffectDef { name: "gradient_deband" },
    EffectDef { name: "threshold_bloom" },
    EffectDef { name: "ghost_flare" },
    EffectDef { name: "crt_simulation" },
    EffectDef { name: "phosphor_amber" },
    EffectDef { name: "phosphor_green" },
    EffectDef { name: "phosphor_red" },
    EffectDef { name: "scanline_darken" },
    EffectDef { name: "oled_simulation" },
    EffectDef { name: "vhs_simulation" },
    EffectDef { name: "lcd_subpixel" },
    EffectDef { name: "fps_hud" },
    EffectDef { name: "crosshair_overlay" },
    EffectDef { name: "neighborhood_clamp_aa" },
    EffectDef { name: "motion_reject_denoise" },
    EffectDef { name: "motion_detect_blur" },
    EffectDef { name: "constant_blend_smooth" },
    EffectDef { name: "shutter_angle_smooth" },
    EffectDef { name: "spline_interp_smooth" },
    EffectDef { name: "variance_decay_smooth" },
    EffectDef { name: "dualrate_smooth" },
    EffectDef { name: "luminance_gate_smooth" },
    EffectDef { name: "contrast_gate_smooth" },
    EffectDef { name: "gradient_gate_smooth" },
    EffectDef { name: "sigma_clip_smooth" },
    EffectDef { name: "mitchell_kernel_smooth" },
    EffectDef { name: "ycocg_clip_smooth" },
    EffectDef { name: "bilateral_history_smooth" },
    EffectDef { name: "perceptual_chroma_smooth" },
    EffectDef { name: "frequency_split_smooth" },
    EffectDef { name: "horn_schunck_smooth" },
    EffectDef { name: "convergent_accumulate" },
    EffectDef { name: "dualwarp_flow_smooth" },
    EffectDef { name: "variance_flow_accumulate" },
    EffectDef { name: "edge_reconstruct_smooth" },
    EffectDef { name: "surface_disocclusion_guard" },
    EffectDef { name: "convergent_detail_recovery" },
    EffectDef { name: "gaussian_grain" },
    EffectDef { name: "chromatic_aberration" },
    EffectDef { name: "red_halation" },
    EffectDef { name: "anamorphic_streak" },
    EffectDef { name: "radial_vignette" },
    EffectDef { name: "cinematic_letterbox" },
    EffectDef { name: "ordered_dither" },
    EffectDef { name: "linear_exposure" },
    EffectDef { name: "aces_tonemap" },
    EffectDef { name: "agx_tonemap" },
    EffectDef { name: "reinhard_tonemap" },
    EffectDef { name: "hable_tonemap" },
    EffectDef { name: "lottes_tonemap" },
    EffectDef { name: "uchimura_tonemap" },
    EffectDef { name: "tony_tonemap" },
    EffectDef { name: "khronos_tonemap" },
    EffectDef { name: "neutral_white_balance" },
    EffectDef { name: "warm_temperature" },
    EffectDef { name: "cool_temperature" },
    EffectDef { name: "saturation_contrast_grade" },
    EffectDef { name: "levels_remap" },
    EffectDef { name: "gamma_correct" },
    EffectDef { name: "vibrance_boost" },
    EffectDef { name: "hsl_transform" },
    EffectDef { name: "split_tone" },
    EffectDef { name: "lift_gamma_gain" },
    EffectDef { name: "hermite_curves" },
    EffectDef { name: "red_channel_curve" },
    EffectDef { name: "green_channel_curve" },
    EffectDef { name: "blue_channel_curve" },
    EffectDef { name: "trizone_color_balance" },
    EffectDef { name: "red_selective_saturate" },
    EffectDef { name: "green_selective_saturate" },
    EffectDef { name: "blue_selective_saturate" },
    EffectDef { name: "dynamic_range_crush" },
    EffectDef { name: "duotone_map" },
    EffectDef { name: "color_wash_tint" },
    EffectDef { name: "posterize_quantize" },
    EffectDef { name: "bleach_bypass" },
    EffectDef { name: "technicolor_process" },
    EffectDef { name: "midpoint_contrast" },
    EffectDef { name: "color_invert" },
    EffectDef { name: "luminance_grayscale" },
    EffectDef { name: "protanopia_simulate" },
    EffectDef { name: "deuteranopia_simulate" },
    EffectDef { name: "tritanopia_simulate" },
    EffectDef { name: "protanopia_correct" },
    EffectDef { name: "deuteranopia_correct" },
    EffectDef { name: "tritanopia_correct" },
];

pub(crate) const TRI_VERTS: [f32; 6] = [-1.0, -1.0, 3.0, -1.0, -1.0, 3.0];
pub(crate) const VBO_COMPONENTS: i32 = 2;
pub(crate) const FULLSCREEN_TRI_VERTS: i32 = 3;

pub(crate) const GL_TEXTURE_2D: u32 = 0x0DE1;
pub(crate) const GL_TEXTURE0: u32 = 0x84C0;
pub(crate) const GL_TEXTURE1: u32 = 0x84C1;
pub(crate) const GL_RGBA8: i32 = 0x8058;
pub(crate) const GL_RGBA: u32 = 0x1908;
pub(crate) const GL_UNSIGNED_BYTE: u32 = 0x1401;
pub(crate) const GL_LINEAR: i32 = 0x2601;
pub(crate) const GL_CLAMP_TO_EDGE: i32 = 0x812F;
pub(crate) const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
pub(crate) const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
pub(crate) const GL_TEXTURE_WRAP_S: u32 = 0x2802;
pub(crate) const GL_TEXTURE_WRAP_T: u32 = 0x2803;
pub(crate) const GL_FRAMEBUFFER: u32 = 0x8D40;
pub(crate) const GL_READ_FRAMEBUFFER: u32 = 0x8CA8;
pub(crate) const GL_DRAW_FRAMEBUFFER: u32 = 0x8CA9;
pub(crate) const GL_COLOR_ATTACHMENT0: u32 = 0x8CE0;
pub(crate) const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
pub(crate) const GL_FRAGMENT_SHADER: u32 = 0x8B30;
pub(crate) const GL_VERTEX_SHADER: u32 = 0x8B31;
pub(crate) const GL_COMPILE_STATUS: u32 = 0x8B81;
pub(crate) const GL_LINK_STATUS: u32 = 0x8B82;
pub(crate) const GL_ARRAY_BUFFER: u32 = 0x8892;
pub(crate) const GL_ARRAY_BUFFER_BINDING: u32 = 0x8894;
pub(crate) const GL_STATIC_DRAW: u32 = 0x88E4;
pub(crate) const GL_FLOAT: u32 = 0x1406;
pub(crate) const GL_TRIANGLES: u32 = 0x0004;
pub(crate) const GL_VIEWPORT: u32 = 0x0BA2;
pub(crate) const GL_SCISSOR_BOX: u32 = 0x0C10;
pub(crate) const GL_CURRENT_PROGRAM: u32 = 0x8B8D;
pub(crate) const GL_ACTIVE_TEXTURE: u32 = 0x84E0;
pub(crate) const GL_TEXTURE_BINDING_2D: u32 = 0x8069;
pub(crate) const GL_READ_FRAMEBUFFER_BINDING: u32 = 0x8CAA;
pub(crate) const GL_DRAW_FRAMEBUFFER_BINDING: u32 = 0x8CA6;
pub(crate) const GL_PIXEL_UNPACK_BUFFER: u32 = 0x88EC;
pub(crate) const GL_PIXEL_UNPACK_BUFFER_BINDING: u32 = 0x88EF;
pub(crate) const GL_COLOR_WRITEMASK: u32 = 0x0C23;
pub(crate) const GL_COLOR_CLEAR_VALUE: u32 = 0x0C22;
pub(crate) const GL_DEPTH_WRITEMASK: u32 = 0x0B72;
pub(crate) const GL_STENCIL_WRITEMASK: u32 = 0x0B98;
pub(crate) const GL_UNPACK_ALIGNMENT: u32 = 0x0CF5;
pub(crate) const GL_UNPACK_ROW_LENGTH: u32 = 0x0CF2;
pub(crate) const GL_BLEND_SRC_RGB: u32 = 0x80C9;
pub(crate) const GL_BLEND_DST_RGB: u32 = 0x80C8;
pub(crate) const GL_BLEND_SRC_ALPHA: u32 = 0x80CB;
pub(crate) const GL_BLEND_DST_ALPHA: u32 = 0x80CA;
pub(crate) const GL_BLEND_EQUATION_RGB: u32 = 0x8009;
pub(crate) const GL_BLEND_EQUATION_ALPHA: u32 = 0x883D;
pub(crate) const GL_DEPTH_TEST: u32 = 0x0B71;
pub(crate) const GL_BLEND: u32 = 0x0BE2;
pub(crate) const GL_CULL_FACE: u32 = 0x0B44;
pub(crate) const GL_SCISSOR_TEST: u32 = 0x0C11;
pub(crate) const GL_STENCIL_TEST: u32 = 0x0B90;
pub(crate) const GL_FRAMEBUFFER_SRGB: u32 = 0x8DB9;
pub(crate) const GL_RASTERIZER_DISCARD: u32 = 0x8C89;
pub(crate) const GL_ALPHA_TEST: u32 = 0x0BC0;
pub(crate) const GL_COLOR_LOGIC_OP: u32 = 0x0BF2;
pub(crate) const GL_VERTEX_PROGRAM_ARB: u32 = 0x8620;
pub(crate) const GL_FRAGMENT_PROGRAM_ARB: u32 = 0x8804;
pub(crate) const GL_VERTEX_ATTRIB_ARRAY_ENABLED: u32 = 0x8622;
pub(crate) const GL_VERTEX_ARRAY_BINDING: u32 = 0x85B5;
pub(crate) const GL_VERSION: u32 = 0x1F02;
pub(crate) const GLX_WIDTH: i32 = 0x801D;
pub(crate) const GLX_HEIGHT: i32 = 0x801E;
pub(crate) const EGL_WIDTH: i32 = 0x3057;
pub(crate) const EGL_HEIGHT: i32 = 0x3056;

pub(crate) const PUSH_BYTES: u32 = 16;
pub(crate) const LAYER_IFACE_VERSION: u32 = 2;
pub(crate) const LAYER_LINK_INFO: i32 = 0;

pub(crate) const NULL_OK: [&str; 4] = [
    "vkCreateInstance",
    "vkEnumerateInstanceVersion",
    "vkEnumerateInstanceExtensionProperties",
    "vkEnumerateInstanceLayerProperties",
];

pub(crate) const VK_HEADER: &str = "#define VULKAN_API 1\nlayout(push_constant) uniform PushBlock { vec2 res; float time; float fps; } pc;\n#define u_resolution pc.res\n#define u_time pc.time\n#define u_fps pc.fps\nlayout(location=0) out vec4 frag_out;";

pub(crate) const VERT_VK_SRC: &str = r#"#version 450
void main() {
    vec2 vk_pos = vec2(float((gl_VertexIndex << 1) & 2), float(gl_VertexIndex & 2));
    gl_Position = vec4(vk_pos * 2.0 - 1.0, 0.0, 1.0);
}
"#;

pub(crate) const VERT_SRC: &str = r#"#version 130
in vec2 a_pos;
void main() { gl_Position = vec4(a_pos, 0.0, 1.0); }
"#;

pub(crate) const UBER_SRC: &str = r#"#version 130

uniform sampler2D u_input;
uniform sampler2D u_history;
uniform vec2 u_resolution;
uniform float u_time;
uniform float u_fps;

out vec4 frag_out;

const vec3 LUMA_BT601 = vec3(0.299, 0.587, 0.114);
const vec3 LUMA_BT709 = vec3(0.2126, 0.7152, 0.0722);

#ifdef ENABLE_CRT_SIMULATION
    vec3 crt_fetch_px(vec2 uv) {
        return texture(u_input, uv).rgb;
    }
#endif

#ifdef ENABLE_FPS_HUD
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

        float d = 999.0;
        if (n < 0.5) d = min(s0, min(s1, min(s2, min(s3, min(s4, s5)))));
        else if (n < 1.5) d = min(s1, s2);
        else if (n < 2.5) d = min(s0, min(s1, min(s6, min(s4, s3))));
        else if (n < 3.5) d = min(s0, min(s1, min(s6, min(s2, s3))));
        else if (n < 4.5) d = min(s5, min(s6, min(s1, s2)));
        else if (n < 5.5) d = min(s0, min(s5, min(s6, min(s2, s3))));
        else if (n < 6.5) d = min(s0, min(s5, min(s6, min(s4, min(s3, s2)))));
        else if (n < 7.5) d = min(s0, min(s1, s2));
        else if (n < 8.5) d = min(s0, min(s1, min(s2, min(s3, min(s4, min(s5, s6))))));
        else d = min(s0, min(s1, min(s2, min(s3, min(s5, s6)))));

        return d;
    }
#endif

#ifdef ENABLE_YCOCG_CLIP_SMOOTH
    vec3 ycocg_encode(vec3 x) {
        return vec3(0.25 * x.r + 0.5 * x.g + 0.25 * x.b,
                    0.5 * x.r - 0.5 * x.b,
                    -0.25 * x.r + 0.5 * x.g - 0.25 * x.b);
    }

    vec3 ycocg_decode(vec3 y) {
        return vec3(y.x + y.y - y.z, y.x + y.z, y.x - y.y - y.z);
    }
#endif

#ifdef ENABLE_PERCEPTUAL_CHROMA_SMOOTH
    vec3 perc_ycc(vec3 x) {
        const float CB_SCALE = 1.8556;
        const float CR_SCALE = 1.5748;
        float y = dot(x, LUMA_BT709);
        return vec3(y, (x.b - y) / max(CB_SCALE, 0.0001),
                       (x.r - y) / max(CR_SCALE, 0.0001));
    }

    vec3 perc_rgb(vec3 y) {
        const float CB_SCALE = 1.8556;
        const float CR_SCALE = 1.5748;
        float r = y.x + y.z * CR_SCALE;
        float b = y.x + y.y * CB_SCALE;
        float g = (y.x - LUMA_BT709.r * r - LUMA_BT709.b * b) / max(LUMA_BT709.g, 0.0001);
        return vec3(r, g, b);
    }
#endif

#ifdef ENABLE_ORDERED_DITHER
    const float bones_bayer4x4[16] = float[16](
        0.0, 8.0, 2.0, 10.0,
        12.0, 4.0, 14.0, 6.0,
        3.0, 11.0, 1.0, 9.0,
        15.0, 7.0, 13.0, 5.0
    );

    float dither_cell(float di) {
        return bones_bayer4x4[int(di)];
    }
#endif

#ifdef ENABLE_HABLE_TONEMAP
    vec3 hable_map(vec3 x) {
        const float HA = 0.15;
        const float HB = 0.5;
        const float HC = 0.1;
        const float HD = 0.2;
        const float HE = 0.02;
        const float HF = 0.3;
        return ((x * (HA * x + HC * HB) + HD * HE) /
                (x * (HA * x + HB) + HD * max(HF, 0.0001))) -
               HE / max(HF, 0.0001);
    }
#endif

#ifdef ENABLE_AGX_TONEMAP
    vec3 agx_contrast(vec3 x) {
        vec3 x2 = x * x;
        vec3 x4 = x2 * x2;
        return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4
             - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
    }
#endif
#ifdef ENABLE_UCHIMURA_TONEMAP
    float uchi_map(float x) {
        const float P = 1.0;
        const float A = 1.0;
        const float M = 0.22;
        const float L = 0.4;
        const float C = 1.33;
        float u_l0 = ((P - M) * L) / max(A, 0.0001);
        float u_s0 = M + u_l0;
        float u_s1 = M + A * u_l0;
        float u_c2 = (A * P) / max(P - u_s1, 0.0001);
        float u_toe = M * pow(max(x, 0.0) / max(M, 0.0001), C);
        float u_lin = M + A * (x - M);
        float u_sho = P - (P - u_s1) * exp(-u_c2 * (x - u_s0) / max(P, 0.0001));
        return mix(mix(u_toe, u_lin, smoothstep(0.0, M, x)),
                   u_sho, smoothstep(M, u_s0, x));
    }
#endif

void main() {
    vec2 inv = 1.0 / u_resolution;
    vec2 v_uv = gl_FragCoord.xy / u_resolution;
    float res_scale = u_resolution.y / 1080.0;

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
        v_uv = vec2(1.0 - v_uv.x, 1.0 - v_uv.y);
    #endif

    #ifdef ENABLE_ROTATE_270
        v_uv = vec2(1.0 - v_uv.y, v_uv.x);
    #endif

    #ifdef ENABLE_CENTER_ZOOM
        {
            const float ZF = 1.5;
            v_uv = vec2(0.5) + (v_uv - vec2(0.5)) / max(ZF, 0.0001);
        }
    #endif

    #ifdef ENABLE_POLYNOMIAL_DISTORT
        {
            const float K1 = 0.1;
            const float K2 = 0.0;
            vec2 dc = v_uv - vec2(0.5);
            float r2 = dot(dc, dc);
            v_uv = vec2(0.5) + dc * (1.0 + K1 * r2 + K2 * r2 * r2);
        }
    #endif

    #ifdef ENABLE_BARREL_UNDISTORT
        {
            const float BS = 0.2;
            vec2 bc = v_uv - vec2(0.5);
            float r2 = dot(bc, bc);
            v_uv = vec2(0.5) + bc / max(1.0 + BS * r2, 0.0001);
        }
    #endif

    #ifdef ENABLE_FISHEYE_WARP
        {
            const float FS = 1.0;
            const float FZ = 1.0;
            vec2 fc = v_uv - vec2(0.5);
            float fr = length(fc);
            float ft = fr * FS;
            float ff = mix(1.0, atan(ft) / max(ft, 0.0001), step(0.0001, fr));
            v_uv = vec2(0.5) + fc * ff * FZ;
        }
    #endif

    #ifdef ENABLE_TRAPEZOID_WARP
        {
            const float PT = 1.0;
            const float PB = 1.2;
            v_uv.x = 0.5 + (v_uv.x - 0.5) * mix(PT, PB, v_uv.y);
        }
    #endif

    #ifdef ENABLE_SHARP_BILINEAR
        {
            const float PA = 4.0;
            vec2 pg = u_resolution / max(PA * res_scale, 1.0);
            vec2 pt = v_uv * pg;
            vec2 pi = floor(pt - 0.5) + 0.5;
            vec2 pf = pt - pi;
            pf = pf * pf * (3.0 - 2.0 * pf);
            v_uv = (pi + pf) / pg;
        }
    #endif

    vec3 tap_0_0 = texture(u_input, v_uv).rgb;

    #if defined(ENABLE_LUMA_EDGE_AA) || defined(ENABLE_NORMAL_FILTER_AA) || defined(ENABLE_MORPHOLOGICAL_AA) || defined(ENABLE_SUBPIXEL_AA) || defined(ENABLE_CONTRAST_ADAPTIVE_SHARPEN) || defined(ENABLE_ROBUST_CONTRAST_SHARPEN) || defined(ENABLE_EDGE_DIRECTED_SHARPEN) || defined(ENABLE_LAPLACIAN_SHARPEN) || defined(ENABLE_LUMINANCE_SHARPEN) || defined(ENABLE_MIDTONE_CLARITY) || defined(ENABLE_FALLOFF_SHARPEN) || defined(ENABLE_POWER_CURVE_SHARPEN) || defined(ENABLE_UNSHARP_MASK) || defined(ENABLE_LOCAL_CONTRAST) || defined(ENABLE_GAUSSIAN_BLUR) || defined(ENABLE_THRESHOLD_BLOOM) || defined(ENABLE_NEIGHBORHOOD_CLAMP_AA) || defined(ENABLE_SIGMA_CLIP_SMOOTH) || defined(ENABLE_DUALRATE_SMOOTH) || defined(ENABLE_HORN_SCHUNCK_SMOOTH) || defined(ENABLE_FREQUENCY_SPLIT_SMOOTH) || defined(ENABLE_GRADIENT_GATE_SMOOTH) || defined(ENABLE_BILATERAL_HISTORY_SMOOTH) || defined(ENABLE_CONTRAST_GATE_SMOOTH) || defined(ENABLE_DUALWARP_FLOW_SMOOTH) || defined(ENABLE_VARIANCE_FLOW_ACCUMULATE) || defined(ENABLE_EDGE_RECONSTRUCT_SMOOTH)
        vec3 tap_1_0   = texture(u_input, v_uv + vec2( 1.0,  0.0) * inv).rgb;
        vec3 tap_m1_0  = texture(u_input, v_uv + vec2(-1.0,  0.0) * inv).rgb;
        vec3 tap_0_1   = texture(u_input, v_uv + vec2( 0.0,  1.0) * inv).rgb;
        vec3 tap_0_m1  = texture(u_input, v_uv + vec2( 0.0, -1.0) * inv).rgb;
    #endif

    #if defined(ENABLE_LUMA_EDGE_AA) || defined(ENABLE_MORPHOLOGICAL_AA) || defined(ENABLE_SUBPIXEL_AA) || defined(ENABLE_CONTRAST_ADAPTIVE_SHARPEN) || defined(ENABLE_FALLOFF_SHARPEN) || defined(ENABLE_UNSHARP_MASK) || defined(ENABLE_GAUSSIAN_BLUR) || defined(ENABLE_THRESHOLD_BLOOM) || defined(ENABLE_NEIGHBORHOOD_CLAMP_AA) || defined(ENABLE_SIGMA_CLIP_SMOOTH)
        vec3 tap_1_1   = texture(u_input, v_uv + vec2( 1.0,  1.0) * inv).rgb;
        vec3 tap_m1_1  = texture(u_input, v_uv + vec2(-1.0,  1.0) * inv).rgb;
        vec3 tap_1_m1  = texture(u_input, v_uv + vec2( 1.0, -1.0) * inv).rgb;
        vec3 tap_m1_m1 = texture(u_input, v_uv + vec2(-1.0, -1.0) * inv).rgb;
    #endif

    vec3 c = tap_0_0;

    #ifdef ENABLE_IDENTITY
    #endif

    #ifdef ENABLE_BILATERAL_DENOISE
        {
            const float DR = 1.0;
            const float DS = 0.01;
            const float DT = 0.6;
            float dr = DR * res_scale;
            vec3 ds = c;
            float dw = 1.0;
            for (int di = -1; di <= 1; di++) {
                for (int dj = -1; dj <= 1; dj++) {
                    vec3 dd = texture(u_input, v_uv + vec2(float(di), float(dj)) * dr * inv).rgb;
                    vec3 de = dd - c;
                    float dk = exp(-dot(de, de) / max(DS, 0.0001));
                    ds += dd * dk;
                    dw += dk;
                }
            }
            c = mix(c, ds / max(dw, 0.0001), DT);
        }
    #endif

    #ifdef ENABLE_LUMA_EDGE_AA
        {
            const float RM = 0.125;
            const float RN = 0.0078125;
            const float SM = 8.0;
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
            float fr = max((lnw + lne + lsw + lse) * RM * 0.25, RN);
            float fp = 1.0 / max(min(abs(fd.x), abs(fd.y)) + fr, 0.0001);
            fd = clamp(fd * fp, vec2(-SM), vec2(SM)) * inv;
            vec3 fa = (texture(u_input, v_uv + fd * (1.0 / 3.0 - 0.5)).rgb +
                       texture(u_input, v_uv + fd * (2.0 / 3.0 - 0.5)).rgb) * 0.5;
            vec3 fb = fa * 0.5 + (texture(u_input, v_uv + fd * -0.5).rgb +
                                  texture(u_input, v_uv + fd * 0.5).rgb) * 0.25;
            float fl = dot(fb, LUMA_BT601);
            c = mix(fb, fa, clamp(step(fl, fmn) + step(fmx, fl), 0.0, 1.0));
        }
    #endif

    #ifdef ENABLE_NORMAL_FILTER_AA
        {
            const float NS = 1.5;
            const float NN = 4.0;
            float ns = NS * res_scale;
            float ne = (tap_1_0.r + tap_1_0.g + tap_1_0.b) / 3.0;
            float nw = (tap_m1_0.r + tap_m1_0.g + tap_m1_0.b) / 3.0;
            float nn = (tap_0_1.r + tap_0_1.g + tap_0_1.b) / 3.0;
            float nd = (tap_0_m1.r + tap_0_m1.g + tap_0_m1.b) / 3.0;
            vec2 ng = vec2(nw - ne, nn - nd);
            vec2 nt = vec2(-ng.y, ng.x) * ns * inv;
            c = mix(c, (texture(u_input, v_uv + nt).rgb +
                        texture(u_input, v_uv - nt).rgb + c) / 3.0,
                    clamp(length(ng) * NN, 0.0, 1.0));
        }
    #endif

    #ifdef ENABLE_MORPHOLOGICAL_AA
        {
            const float CT = 0.1;
            const float CS = 0.1;
            const float CW = 0.7;
            vec3 ca = (tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) * 0.25;
            vec3 cd = (tap_1_1 + tap_m1_1 + tap_1_m1 + tap_m1_m1) * 0.25;
            float ce = (abs(dot(tap_1_0 - tap_m1_0, vec3(1.0))) +
                        abs(dot(tap_0_1 - tap_0_m1, vec3(1.0)))) / 6.0;
            c = mix(c, (ca + cd) * 0.5, smoothstep(CT, CT + CS, ce) * CW);
        }
    #endif

    #ifdef ENABLE_SUBPIXEL_AA
        {
            const float ST = 0.1;
            const float SS = 0.1;
            const float SD = 0.3;
            const float SX = 0.75;
            float sh = abs(tap_1_0.g - c.g) + abs(tap_m1_0.g - c.g);
            float sv = abs(tap_0_1.g - c.g) + abs(tap_0_m1.g - c.g);
            float se = max(sh, sv);
            vec3 sc2 = mix((tap_1_0 + tap_m1_0) * 0.5,
                           (tap_0_1 + tap_0_m1) * 0.5,
                           step(sv, sh));
            vec3 sd2 = (tap_1_1 + tap_m1_1 + tap_1_m1 + tap_m1_m1) * 0.25;
            c = mix(c, mix(sc2, sd2, SD), min(smoothstep(ST, ST + SS, se), SX));
        }
    #endif

    #ifdef ENABLE_CONTRAST_ADAPTIVE_SHARPEN
        {
            const float CS2 = 0.5;
            const float PL = 0.125;
            const float PH = 0.2;
            vec3 mn = min(min(tap_1_0, tap_m1_0), min(tap_0_1, tap_0_m1));
            vec3 mx = max(max(tap_1_0, tap_m1_0), max(tap_0_1, tap_0_m1));
            mn = min(mn, min(min(tap_1_1, tap_m1_1), min(tap_1_m1, tap_m1_m1)));
            mx = max(mx, max(max(tap_1_1, tap_m1_1), max(tap_1_m1, tap_m1_m1)));
            mn = min(mn, c);
            mx = max(mx, c);
            vec3 amp = sqrt(clamp(min(mn, vec3(2.0) - mx) / max(mx, vec3(0.0001)), 0.0, 1.0));
            vec3 w = amp * mix(PL, PH, CS2);
            c = clamp(((tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) * w + c) /
                       max(w * 4.0 + vec3(1.0), vec3(0.0001)), 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_ROBUST_CONTRAST_SHARPEN
        {
            const float RS = 0.25;
            vec3 rmn = min(min(tap_1_0, tap_m1_0), min(tap_0_1, tap_0_m1));
            vec3 rmx = max(max(tap_1_0, tap_m1_0), max(tap_0_1, tap_0_m1));
            vec3 rsu = tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1;
            c = clamp(c + (c * 4.0 - rsu) * RS * 0.25, min(rmn, c), max(rmx, c));
        }
    #endif

    #ifdef ENABLE_EDGE_DIRECTED_SHARPEN
        {
            const float ES = 0.5;
            const float EE = 8.0;
            const float EF = 0.0;
            float eh = ((tap_1_0.r + tap_1_0.g + tap_1_0.b) -
                        (tap_m1_0.r + tap_m1_0.g + tap_m1_0.b)) / 3.0;
            float ev = ((tap_0_1.r + tap_0_1.g + tap_0_1.b) -
                        (tap_0_m1.r + tap_0_m1.g + tap_0_m1.b)) / 3.0;
            float eg = length(vec2(eh, ev));
            float ew = ES * clamp(eg * EE, EF, 1.0);
            c = clamp(c + (c * 4.0 - (tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1)) * ew * 0.25,
                      0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_LAPLACIAN_SHARPEN
        {
            const float LS = 0.5;
            c = c + (c * 4.0 - (tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1)) * LS * 0.25;
        }
    #endif

    #ifdef ENABLE_LUMINANCE_SHARPEN
        {
            const float LS2 = 1.0;
            const float LC = 0.1;
            float ll = dot(c, LUMA_BT601);
            float la = dot(tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1, LUMA_BT601) * 0.25;
            float ld = clamp((ll - la) * LS2, -LC, LC);
            c = c + vec3(ld);
        }
    #endif

    #ifdef ENABLE_MIDTONE_CLARITY
        {
            const float MC = 0.5;
            vec3 mb = (tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) * 0.25;
            float ml = (c.r + c.g + c.b) / 3.0;
            float mm = 1.0 - abs(ml * 2.0 - 1.0);
            c = c + (c - mb) * MC * mm;
        }
    #endif

    #ifdef ENABLE_FALLOFF_SHARPEN
        {
            const float AW = 0.2;
            const float DW = 0.05;
            const float AS = 0.6;
            const float AA = 4.0;
            vec3 ab = (tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) * AW +
                      (tap_1_1 + tap_m1_1 + tap_1_m1 + tap_m1_m1) * DW;
            vec3 ad = c - ab / max(AW * 4.0 + DW * 4.0, 0.0001);
            float ae = abs(ad.r) + abs(ad.g) + abs(ad.b);
            float aw2 = AS / max(1.0 + ae * AA, 0.0001);
            c = c + ad * aw2;
        }
    #endif

    #ifdef ENABLE_POWER_CURVE_SHARPEN
        {
            const float PC = 0.7;
            const float PS = 0.5;
            vec3 pb = (tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) * 0.25;
            vec3 pd = c - pb;
            c = c + sign(pd) * pow(abs(pd), vec3(PC)) * PS;
        }
    #endif

    #ifdef ENABLE_UNSHARP_MASK
        {
            const float UC = 4.0;
            const float UK = 1.0;
            const float UD = 0.5;
            const float UA = 0.5;
            vec3 ub = (c * UC +
                       (tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) * UK +
                       (tap_1_1 + tap_m1_1 + tap_1_m1 + tap_m1_m1) * UD) /
                      max(UC + UK * 4.0 + UD * 4.0, 0.0001);
            c = c + (c - ub) * UA;
        }
    #endif

    #ifdef ENABLE_LOCAL_CONTRAST
        {
            const float LCA = 0.3;
            float ll = (c.r + c.g + c.b) / 3.0;
            vec3 ln2 = (tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) * 0.25;
            float la = (ln2.r + ln2.g + ln2.b) / 3.0;
            c = c * (1.0 + (ll - la) * LCA / max(ll, 0.0001));
        }
    #endif

    #ifdef ENABLE_GAUSSIAN_BLUR
        {
            const float BC = 4.0;
            const float BK = 2.0;
            const float BD = 1.0;
            const float BS = 1.0;
            vec3 bsu = c * BC +
                       (tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) * BK +
                       (tap_1_1 + tap_m1_1 + tap_1_m1 + tap_m1_m1) * BD;
            c = mix(c, bsu / max(BC + BK * 4.0 + BD * 4.0, 0.0001), BS);
        }
    #endif

    #ifdef ENABLE_BOX_BLUR
        {
            const float BR = 1.0;
            const float BS2 = 1.0;
            float br = BR * res_scale;
            vec3 bs = vec3(0.0);
            float bn = 0.0;
            for (int bi = -2; bi <= 2; bi++) {
                for (int bj = -2; bj <= 2; bj++) {
                    bs += texture(u_input, v_uv + vec2(float(bi), float(bj)) * br * inv).rgb;
                    bn += 1.0;
                }
            }
            c = mix(c, bs / max(bn, 1.0), BS2);
        }
    #endif

    #ifdef ENABLE_BOKEH_BLUR
        {
            const float KR = 4.0;
            const float KS = 0.7853982;
            const float KH = 0.5;
            const float KT = 1.0;
            float kr = KR * res_scale;
            vec3 ks = c;
            float kn = 1.0;
            for (int ki = 0; ki < 8; ki++) {
                float ka = float(ki) * KS;
                vec3 kp = texture(u_input, v_uv + vec2(cos(ka), sin(ka)) * kr * inv).rgb;
                float kw = 1.0 + dot(kp, vec3(1.0)) * KH;
                ks += kp * kw;
                kn += kw;
            }
            c = mix(c, ks / max(kn, 0.0001), KT);
        }
    #endif

    #ifdef ENABLE_TILT_SHIFT_BLUR
        {
            const float TC = 0.5;
            const float TB = 0.15;
            const float TF = 0.2;
            const float TR = 2.0;
            const float TS = 1.0;
            float tr = TR * res_scale;
            float td = clamp((abs(v_uv.y - TC) - TB) / max(TF, 0.0001), 0.0, 1.0);
            vec3 ts = c;
            float tn = 1.0;
            for (int ti = 1; ti <= 4; ti++) {
                ts += texture(u_input, v_uv + vec2(float(ti), 0.0) * tr * td * inv).rgb;
                ts += texture(u_input, v_uv - vec2(float(ti), 0.0) * tr * td * inv).rgb;
                tn += 2.0;
            }
            c = mix(c, ts / tn, td * TS);
        }
    #endif

    #ifdef ENABLE_RADIAL_BLUR
        {
            const float RS2 = 0.2;
            const float RM = 0.5;
            vec2 rd = (vec2(0.5) - v_uv) * RS2;
            vec3 rs = c;
            for (int ri = 1; ri <= 7; ri++) {
                rs += texture(u_input, v_uv + rd * float(ri) / 7.0).rgb;
            }
            c = mix(c, rs / 8.0, RM);
        }
    #endif

    #ifdef ENABLE_GRADIENT_DEBAND
        {
            const float DR2 = 8.0;
            const float DT = 0.02;
            const float DS2 = 1.0;
            const float DA = 1.0;
            float dr = DR2 * res_scale;
            float dh = fract(sin(dot(v_uv, vec2(12.9898, 78.233)) + u_time * DA) * 43758.5453);
            float da = dh * 6.2831853;
            vec3 ds = texture(u_input, v_uv + vec2(cos(da), sin(da)) * dr * inv).rgb;
            vec3 dd = abs(ds - c);
            float dm = step(max(dd.r, max(dd.g, dd.b)), DT);
            c = mix(c, (c + ds) * 0.5, dm * DS2);
        }
    #endif

    #ifdef ENABLE_THRESHOLD_BLOOM
        {
            const float BT = 0.7;
            const float BD2 = 12.0;
            const float BI = 0.6;
            vec3 bt = vec3(BT);
            vec3 bs = max(tap_1_0 - bt, vec3(0.0)) * 2.0 +
                      max(tap_m1_0 - bt, vec3(0.0)) * 2.0 +
                      max(tap_0_1 - bt, vec3(0.0)) * 2.0 +
                      max(tap_0_m1 - bt, vec3(0.0)) * 2.0 +
                      max(tap_1_1 - bt, vec3(0.0)) +
                      max(tap_m1_1 - bt, vec3(0.0)) +
                      max(tap_1_m1 - bt, vec3(0.0)) +
                      max(tap_m1_m1 - bt, vec3(0.0));
            c = c + bs / max(BD2, 0.0001) * BI;
        }
    #endif

    #ifdef ENABLE_GHOST_FLARE
        {
            const float GS = 0.2;
            const float GT = 0.7;
            const float GI = 0.4;
            const float GF = 1.0;
            vec2 gc = vec2(0.5) - v_uv;
            vec3 gg = vec3(0.0);
            for (int gi = 1; gi <= 3; gi++) {
                vec3 gs = texture(u_input, v_uv + gc * GS * float(gi)).rgb;
                gg += max(gs - vec3(GT), vec3(0.0));
            }
            c = c + gg * GI * (1.0 - clamp(length(gc) * GF, 0.0, 1.0));
        }
    #endif

    #ifdef ENABLE_CRT_SIMULATION
        {
            const float CC = 0.5;
            const float CWX = 0.031;
            const float CWY = 0.041;
            const float CMS = 0.3333333;
            const float CT1 = 0.3333333;
            const float CT2 = 0.6666666;
            const float CMD = 0.7;
            const float CMB = 1.0;
            const float CSB = 1.0;
            const float CSH = 0.5;
            const float CSS = 0.25;
            const float CSF = 3.1415927;
            const float CBR = 1.12;
            vec2 cc = v_uv - vec2(CC);
            float cr = dot(cc, cc);
            vec2 cu = v_uv + cc * cr * vec2(CWX, CWY);
            float ci = step(0.0, cu.x) * step(cu.x, 1.0) * step(0.0, cu.y) * step(cu.y, 1.0);
            vec3 co = mix(c, crt_fetch_px(cu), 1.0) * ci;
            float cp = fract(cu.x * u_resolution.x * CMS);
            float mr = mix(CMD, CMB, step(cp, CT1));
            float mg = mix(CMD, CMB, step(CT1, cp) * step(cp, CT2));
            float mb = mix(CMD, CMB, step(CT2, cp));
            float sc = CSB - CSS * (CSH + CSH * sin(cu.y * u_resolution.y * CSF));
            c = co * vec3(mr, mg, mb) * sc * CBR;
        }
    #endif

    #ifdef ENABLE_PHOSPHOR_AMBER
        {
            float pl = dot(c, LUMA_BT601);
            c = vec3(1.0, 0.75, 0.3) * pl * 1.1;
        }
    #endif

    #ifdef ENABLE_PHOSPHOR_GREEN
        {
            float pl = dot(c, LUMA_BT601);
            c = vec3(0.2, 1.0, 0.2) * pl * 1.1;
        }
    #endif

    #ifdef ENABLE_PHOSPHOR_RED
        {
            float pl = dot(c, LUMA_BT601);
            c = vec3(1.0, 0.2, 0.2) * pl * 1.1;
        }
    #endif

    #ifdef ENABLE_SCANLINE_DARKEN
        {
            const float SB = 1.0;
            const float SH = 0.5;
            const float SS = 0.3;
            const float SF = 3.1415927;
            c = c * (SB - SS * (SH + SH * sin(v_uv.y * u_resolution.y * SF)));
        }
    #endif

    #ifdef ENABLE_OLED_SIMULATION
        {
            const float OB = 0.05;
            const float OS = 1.1;
            float ol = dot(c, LUMA_BT601);
            float oc = smoothstep(0.0, OB, ol);
            c = mix(vec3(ol), c, OS) * oc;
        }
    #endif

    #ifdef ENABLE_VHS_SIMULATION
        {
            const float VJ = 2.0;
            const float VC = 2.0;
            const float VN = 0.05;
            const float VR = 1.0;
            const float VF = 30.0;
            const float VS = 2.0;
            const float VA = 10.0;
            float vs = res_scale;
            float vj = (fract(sin(floor(v_uv.y * u_resolution.y) + u_time * VA) * 43758.5453) - 0.5) * VJ * vs * inv.x;
            float vr = sin(v_uv.y * VF + u_time * VS) * VR * vs * inv.x;
            vec2 vu = v_uv + vec2(vj + vr, 0.0);
            float vc = VC * vs * inv.x;
            c = vec3(texture(u_input, vu + vec2(vc, 0.0)).r,
                     texture(u_input, vu).g,
                     texture(u_input, vu - vec2(vc, 0.0)).b);
            float vn = fract(sin(dot(vu, vec2(12.9898, 78.233)) + u_time * VA) * 43758.5453) - 0.5;
            c = c + vec3(vn) * VN;
        }
    #endif

    #ifdef ENABLE_LCD_SUBPIXEL
        {
            const float LC2 = 3.0;
            const float LG = 0.2;
            const float LD = 0.6;
            const float LB = 1.1;
            float lc = LC2 * res_scale;
            vec2 lp = fract(gl_FragCoord.xy / lc);
            float lm = step(LG, lp.x) * step(LG, lp.y);
            c = c * mix(LD, 1.0, lm) * LB;
        }
    #endif

    #ifdef ENABLE_FPS_HUD
        {
            const float HX = 0.012;
            const float HY = 0.012;
            const float HS = 26.0;
            float hs = HS * res_scale;

            #ifdef VULKAN_API
                vec2 hu = v_uv;
            #else
                vec2 hu = vec2(v_uv.x, 1.0 - v_uv.y);
            #endif

            vec2 raw_hp = (hu - vec2(HX, HY)) * u_resolution / hs;

            vec2 hp = raw_hp;
            hp.x += (1.0 - hp.y) * 0.15;

            float fps_clamped = clamp(u_fps, 0.0, 9999.0);
            float fps_digit3 = mod(floor(fps_clamped / 1000.0), 10.0);
            float fps_digit2 = mod(floor(fps_clamped / 100.0), 10.0);
            float fps_digit1 = mod(floor(fps_clamped / 10.0), 10.0);
            float fps_digit0 = mod(floor(fps_clamped), 10.0);

            float spacing = 0.7;
            vec2 fps_pos3 = hp;
            vec2 fps_pos2 = hp - vec2(spacing, 0.0);
            vec2 fps_pos1 = hp - vec2(spacing * 2.0, 0.0);
            vec2 fps_pos0 = hp - vec2(spacing * 3.0, 0.0);

            float show3 = step(1.0, fps_digit3);
            float show2 = step(1.0, fps_digit3 + fps_digit2);
            float show1 = step(1.0, fps_digit3 + fps_digit2 + fps_digit1);
            float show0 = 1.0;

            float d3 = mix(999.0, hud_digit(fps_pos3, fps_digit3), show3);
            float d2 = mix(999.0, hud_digit(fps_pos2, fps_digit2), show2);
            float d1 = mix(999.0, hud_digit(fps_pos1, fps_digit1), show1);
            float d0 = mix(999.0, hud_digit(fps_pos0, fps_digit0), show0);

            float d_text = min(min(d3, d2), min(d1, d0));

            float line_thickness = 0.06;
            float glow_thickness = 0.35;

            float text_core = 1.0 - smoothstep(line_thickness - 0.015, line_thickness + 0.015, d_text);

            float text_glow = 1.0 - smoothstep(line_thickness, glow_thickness, d_text);

            vec2 bg_center = vec2(1.55, 0.45);
            vec2 raw_bg_p = raw_hp - bg_center;

            vec2 bg_d = abs(raw_bg_p) - vec2(1.4, 0.2);
            float bg_dist = length(max(bg_d, 0.0)) + min(max(bg_d.x, bg_d.y), 0.0);

            float bg_alpha = (1.0 - smoothstep(0.1, 0.5, bg_dist)) * 0.75;

            vec3 color_bg   = vec3(0.01, 0.02, 0.05);
            vec3 color_glow = vec3(0.0, 0.5, 1.0);
            vec3 color_core = vec3(0.85, 0.95, 1.0);

            c = mix(c, color_bg, bg_alpha);
            c = mix(c, color_glow, text_glow * 0.85);
            c = mix(c, color_core, text_core);
        }
    #endif

    #ifdef ENABLE_CROSSHAIR_OVERLAY
        {
            float cs = res_scale;
            vec2 cp = abs(gl_FragCoord.xy - u_resolution * 0.5);

            float d_dot = max(0.0, length(cp) - 0.5 * cs);

            vec2 pa_h = cp - vec2(4.0 * cs, 0.0);
            float h_h = clamp(pa_h.x / (10.0 * cs), 0.0, 1.0);
            float d_arm_h = length(pa_h - vec2(10.0 * cs * h_h, 0.0));

            vec2 pa_v = cp - vec2(0.0, 4.0 * cs);
            float h_v = clamp(pa_v.y / (10.0 * cs), 0.0, 1.0);
            float d_arm_v = length(pa_v - vec2(0.0, 10.0 * cs * h_v));

            float d_cross = min(d_dot, min(d_arm_h, d_arm_v));

            float line_thickness = 1.0 * cs;
            float glow_thickness = 4.0 * cs;

            float cross_core = 1.0 - smoothstep(line_thickness - 0.5 * cs, line_thickness + 0.5 * cs, d_cross);
            float cross_glow = 1.0 - smoothstep(line_thickness, glow_thickness, d_cross);

            vec3 color_glow = vec3(0.0, 0.5, 1.0);
            vec3 color_core = vec3(0.85, 0.95, 1.0);

            c = mix(c, color_glow, cross_glow * 0.85);
            c = mix(c, color_core, cross_core);
        }
    #endif

    #ifdef ENABLE_NEIGHBORHOOD_CLAMP_AA
        {
            const float TB = 0.1;
            vec3 th = texture(u_history, v_uv).rgb;
            vec3 tmn = min(min(tap_1_0, tap_m1_0), min(tap_0_1, tap_0_m1));
            vec3 tmx = max(max(tap_1_0, tap_m1_0), max(tap_0_1, tap_0_m1));
            tmn = min(tmn, min(min(tap_1_1, tap_m1_1), min(tap_1_m1, tap_m1_m1)));
            tmx = max(tmx, max(max(tap_1_1, tap_m1_1), max(tap_1_m1, tap_m1_m1)));
            tmn = min(tmn, c);
            tmx = max(tmx, c);
            c = mix(clamp(th, tmn, tmx), c, TB);
        }
    #endif

    #ifdef ENABLE_MOTION_REJECT_DENOISE
        {
            const float TA = 0.8;
            const float TM = 8.0;
            vec3 th = texture(u_history, v_uv).rgb;
            vec3 td = abs(c - th);
            float tm = max(td.r, max(td.g, td.b));
            c = mix(c, th, TA * (1.0 - clamp(tm * TM, 0.0, 1.0)));
        }
    #endif

    #ifdef ENABLE_MOTION_DETECT_BLUR
        {
            const float MS = 0.5;
            const float MM = 8.0;
            vec3 mh = texture(u_history, v_uv).rgb;
            vec3 md = abs(c - mh);
            float mm = clamp(max(md.r, max(md.g, md.b)) * MM, 0.0, 1.0);
            c = mix(mh, c, mix(1.0, MS, mm));
        }
    #endif

    #ifdef ENABLE_CONSTANT_BLEND_SMOOTH
        {
            c = mix(c, texture(u_history, v_uv).rgb, 0.5);
        }
    #endif

    #ifdef ENABLE_SHUTTER_ANGLE_SMOOTH
        {
            const float SA = 180.0;
            const float FM = 0.9;
            c = mix(c, texture(u_history, v_uv).rgb, clamp(SA / 360.0, 0.0, FM));
        }
    #endif

    #ifdef ENABLE_SPLINE_INTERP_SMOOTH
        {
            const float CT2 = 0.5;
            const float CB2 = 0.5;
            vec3 ch = texture(u_history, v_uv).rgb;
            vec3 ce = ch + (c - ch) * CT2;
            c = mix(c, ce, CB2);
        }
    #endif

    #ifdef ENABLE_VARIANCE_DECAY_SMOOTH
        {
            const float EB = 0.6;
            const float EV = 50.0;
            const float EM = 0.9;
            vec3 eh = texture(u_history, v_uv).rgb;
            vec3 ed = c - eh;
            float ev = dot(ed, ed);
            float ea = EB / max(1.0 + ev * EV, 0.0001);
            c = mix(c, eh, clamp(ea, 0.0, EM));
        }
    #endif

    #ifdef ENABLE_DUALRATE_SMOOTH
        {
            const float RB = 0.7;
            const float RN = 0.1;
            const float RV = 50.0;
            vec3 rm = (c + tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) / 5.0;
            vec3 r0 = c - rm;
            vec3 r1 = tap_1_0 - rm;
            vec3 r2 = tap_m1_0 - rm;
            vec3 r3 = tap_0_1 - rm;
            vec3 r4 = tap_0_m1 - rm;
            float rv = (dot(r0, r0) + dot(r1, r1) + dot(r2, r2) + dot(r3, r3) + dot(r4, r4)) / 5.0;
            float rw = mix(RB, RN, clamp(rv * RV, 0.0, 1.0));
            c = mix(c, texture(u_history, v_uv).rgb, rw);
        }
    #endif

    #ifdef ENABLE_LUMINANCE_GATE_SMOOTH
        {
            const float LD2 = 0.7;
            const float LB2 = 0.15;
            const float LT2 = 0.3;
            vec3 lh = texture(u_history, v_uv).rgb;
            float ll = dot(c, LUMA_BT709);
            c = mix(c, lh, mix(LD2, LB2, smoothstep(0.0, LT2, ll)));
        }
    #endif

    #ifdef ENABLE_CONTRAST_GATE_SMOOTH
        {
            const float CL = 0.7;
            const float CH = 0.1;
            const float CS3 = 8.0;
            vec3 ch = texture(u_history, v_uv).rgb;
            float cl = dot(c, LUMA_BT709);
            float cn = dot(tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1, LUMA_BT709) * 0.25;
            float cc = abs(cl - cn) * CS3;
            c = mix(c, ch, mix(CL, CH, clamp(cc, 0.0, 1.0)));
        }
    #endif

    #ifdef ENABLE_GRADIENT_GATE_SMOOTH
        {
            const float GB = 0.6;
            const float GS2 = 12.0;
            vec3 gh = texture(u_history, v_uv).rgb;
            float gx = length(tap_1_0 - tap_m1_0);
            float gy = length(tap_0_1 - tap_0_m1);
            float ge = clamp((gx + gy) * GS2, 0.0, 1.0);
            c = mix(c, gh, GB * (1.0 - ge));
        }
    #endif

    #ifdef ENABLE_SIGMA_CLIP_SMOOTH
        {
            const float VG = 1.0;
            const float VB = 0.6;
            vec3 vm1 = (c + tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1 +
                        tap_1_1 + tap_m1_1 + tap_1_m1 + tap_m1_m1) / 9.0;
            vec3 vm2 = (c * c + tap_1_0 * tap_1_0 + tap_m1_0 * tap_m1_0 +
                        tap_0_1 * tap_0_1 + tap_0_m1 * tap_0_m1 +
                        tap_1_1 * tap_1_1 + tap_m1_1 * tap_m1_1 +
                        tap_1_m1 * tap_1_m1 + tap_m1_m1 * tap_m1_m1) / 9.0;
            vec3 vsd = sqrt(max(vm2 - vm1 * vm1, vec3(0.0)));
            vec3 vh = clamp(texture(u_history, v_uv).rgb,
                            vm1 - vsd * VG, vm1 + vsd * VG);
            c = mix(c, vh, VB);
        }
    #endif

    #ifdef ENABLE_MITCHELL_KERNEL_SMOOTH
        {
            const float MB = 0.3333333;
            const float MB2 = 0.5;
            float mc0 = 1.0 - MB / 3.0;
            float mc1 = MB / 6.0;
            vec3 ms = (texture(u_input, v_uv + vec2(1.0, 0.0) * inv).rgb +
                       texture(u_input, v_uv + vec2(-1.0, 0.0) * inv).rgb +
                       texture(u_input, v_uv + vec2(0.0, 1.0) * inv).rgb +
                       texture(u_input, v_uv + vec2(0.0, -1.0) * inv).rgb);
            vec3 mf = (c * mc0 + ms * mc1) / max(mc0 + mc1 * 4.0, 0.0001);
            c = mix(mf, texture(u_history, v_uv).rgb, MB2);
        }
    #endif

    #ifdef ENABLE_YCOCG_CLIP_SMOOTH
        {
            const float FB = 1.0;
            const float FC = 2.0;
            const float FD = 0.7;
            float fb = FB * res_scale;
            vec3 fh = ycocg_encode(texture(u_history, v_uv).rgb);
            vec3 fu = ycocg_encode(c);
            vec3 fe = ycocg_encode(texture(u_input, v_uv + vec2(fb, 0.0) * inv).rgb);
            vec3 fw = ycocg_encode(texture(u_input, v_uv + vec2(-fb, 0.0) * inv).rgb);
            vec3 fn = ycocg_encode(texture(u_input, v_uv + vec2(0.0, fb) * inv).rgb);
            vec3 fs = ycocg_encode(texture(u_input, v_uv + vec2(0.0, -fb) * inv).rgb);
            vec3 fmn = min(fu, min(min(fe, fw), min(fn, fs)));
            vec3 fmx = max(fu, max(max(fe, fw), max(fn, fs)));
            vec3 fcl = clamp(fh, fmn, fmx);
            vec3 fdc = abs(fh - fcl);
            float ff = 1.0 - clamp((fdc.x + fdc.y + fdc.z) * FC, 0.0, 1.0);
            c = ycocg_decode(mix(fu, fcl, FD * ff));
        }
    #endif

    #ifdef ENABLE_BILATERAL_HISTORY_SMOOTH
        {
            const float BB = 0.5;
            const float BS3 = 0.05;
            vec3 bh = texture(u_history, v_uv).rgb;
            vec3 be = texture(u_history, v_uv + vec2(1.0, 0.0) * inv).rgb;
            vec3 bw = texture(u_history, v_uv + vec2(-1.0, 0.0) * inv).rgb;
            vec3 bn = texture(u_history, v_uv + vec2(0.0, 1.0) * inv).rgb;
            vec3 bs = texture(u_history, v_uv + vec2(0.0, -1.0) * inv).rgb;
            vec3 bca = (c + tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) / 5.0;
            vec3 bha = (bh + be + bw + bn + bs) / 5.0;
            vec3 bd = bca - bha;
            float bsm = exp(-dot(bd, bd) / max(BS3, 0.0001));
            c = mix(c, bh, BB * bsm);
        }
    #endif

    #ifdef ENABLE_PERCEPTUAL_CHROMA_SMOOTH
        {
            const float PL = 0.4;
            const float PC = 0.7;
            vec3 pu = perc_ycc(c);
            vec3 ph = perc_ycc(texture(u_history, v_uv).rgb);
            c = perc_rgb(mix(pu, ph, vec3(PL, PC, PC)));
        }
    #endif

    #ifdef ENABLE_FREQUENCY_SPLIT_SMOOTH
        {
            const float FL = 0.7;
            const float FH = 0.2;
            vec3 flo = (c + tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) / 5.0;
            vec3 fhi = c - flo;
            vec3 fhh = texture(u_history, v_uv).rgb;
            vec3 fhl = (fhh +
                        texture(u_history, v_uv + vec2(1.0, 0.0) * inv).rgb +
                        texture(u_history, v_uv + vec2(-1.0, 0.0) * inv).rgb +
                        texture(u_history, v_uv + vec2(0.0, 1.0) * inv).rgb +
                        texture(u_history, v_uv + vec2(0.0, -1.0) * inv).rgb) / 5.0;
            vec3 fhhi = fhh - fhl;
            c = mix(flo, fhl, FL) + mix(fhi, fhhi, FH);
        }
    #endif

    #ifdef ENABLE_HORN_SCHUNCK_SMOOTH
        {
            const float OS = 1.0;
            const float OM = 4.0;
            const float OB = 0.5;
            float om = OM * res_scale;
            float ol = (c.r + c.g + c.b) / 3.0;
            float ox = ((tap_1_0.r + tap_1_0.g + tap_1_0.b) -
                        (tap_m1_0.r + tap_m1_0.g + tap_m1_0.b)) / 6.0;
            float oy = ((tap_0_1.r + tap_0_1.g + tap_0_1.b) -
                        (tap_0_m1.r + tap_0_m1.g + tap_0_m1.b)) / 6.0;
            vec3 oh = texture(u_history, v_uv).rgb;
            float ot = ol - (oh.r + oh.g + oh.b) / 3.0;
            float od = ox * ox + oy * oy;
            vec2 of2 = vec2(ox, oy) * (-ot / max(od, 0.0001));
            of2 = clamp(of2 * OS, vec2(-om), vec2(om));
            c = mix(c, texture(u_history, v_uv + of2 * inv).rgb, OB);
        }
    #endif

    #ifdef ENABLE_CONVERGENT_ACCUMULATE
        {
            const float AN = 0.1;
            const float AX = 0.85;
            const float AR = 15.0;
            vec3 ah = texture(u_history, v_uv).rgb;
            vec3 ad = c - ah;
            float am = dot(ad, ad);
            float as2 = 1.0 - clamp(am * AR, 0.0, 1.0);
            c = mix(c, ah, mix(AN, AX, as2 * as2));
        }
    #endif

    #ifdef ENABLE_DUALWARP_FLOW_SMOOTH
        {
            const float FS2 = 1.0;
            const float FM2 = 4.0;
            const float FB2 = 0.6;
            const float FD2 = 0.12;
            const float FT = 0.5;
            float fm = FM2 * res_scale;
            float fl = (c.r + c.g + c.b) / 3.0;
            float fx = ((tap_1_0.r + tap_1_0.g + tap_1_0.b) -
                        (tap_m1_0.r + tap_m1_0.g + tap_m1_0.b)) / 6.0;
            float fy = ((tap_0_1.r + tap_0_1.g + tap_0_1.b) -
                        (tap_0_m1.r + tap_0_m1.g + tap_0_m1.b)) / 6.0;
            vec3 fh = texture(u_history, v_uv).rgb;
            float ft2 = fl - (fh.r + fh.g + fh.b) / 3.0;
            float fd2 = fx * fx + fy * fy;
            vec2 ff = vec2(fx, fy) * (-ft2 / max(fd2 + 0.001, 0.0001));
            ff = clamp(ff * FS2, vec2(-fm), vec2(fm));
            vec2 fw = v_uv + ff * inv;
            vec3 fa = texture(u_history, fw).rgb;
            float rl = (fa.r + fa.g + fa.b) / 3.0;
            float rt = rl - fl;
            vec2 rv = vec2(fx, fy) * (-rt / max(fd2 + 0.001, 0.0001));
            rv = clamp(rv * FS2, vec2(-fm), vec2(fm));
            float fe = length(ff + rv);
            float fc2 = 1.0 - clamp(fe / max(FT, 0.0001), 0.0, 1.0);
            vec2 fhf = ff * 0.5 * inv;
            vec3 wc = texture(u_input, v_uv - fhf).rgb;
            vec3 wh = texture(u_history, v_uv + fhf).rgb;
            vec3 fmn = min(c, min(min(tap_1_0, tap_m1_0), min(tap_0_1, tap_0_m1)));
            vec3 fmx = max(c, max(max(tap_1_0, tap_m1_0), max(tap_0_1, tap_0_m1)));
            wh = clamp(wh, fmn, fmx);
            vec3 dd = abs(wc - wh);
            float ds = step(FD2, max(dd.r, max(dd.g, dd.b)));
            c = mix(mix(wc, wh, FB2 * fc2), c, ds);
        }
    #endif

    #ifdef ENABLE_VARIANCE_FLOW_ACCUMULATE
        {
            const float DN = 0.05;
            const float DX = 0.85;
            const float DM = 12.0;
            const float DL = 8.0;
            const float DV = 1.25;
            float dl = (c.r + c.g + c.b) / 3.0;
            float dx = ((tap_1_0.r + tap_1_0.g + tap_1_0.b) -
                        (tap_m1_0.r + tap_m1_0.g + tap_m1_0.b)) / 6.0;
            float dy = ((tap_0_1.r + tap_0_1.g + tap_0_1.b) -
                        (tap_0_m1.r + tap_0_m1.g + tap_0_m1.b)) / 6.0;
            vec3 dh = texture(u_history, v_uv).rgb;
            float dt = dl - (dh.r + dh.g + dh.b) / 3.0;
            float dd2 = dx * dx + dy * dy;
            vec2 df = vec2(dx, dy) * (-dt / max(dd2 + 0.001, 0.0001));
            df = clamp(df, vec2(-4.0 * res_scale), vec2(4.0 * res_scale));
            vec3 dw = texture(u_history, v_uv + df * inv).rgb;
            vec3 dm1 = (c + tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) / 5.0;
            vec3 dm2 = (c * c + tap_1_0 * tap_1_0 + tap_m1_0 * tap_m1_0 +
                        tap_0_1 * tap_0_1 + tap_0_m1 * tap_0_m1) / 5.0;
            vec3 ds = sqrt(max(dm2 - dm1 * dm1, vec3(0.0)));
            vec3 dc = clamp(dw, dm1 - ds * DV, dm1 + ds * DV);
            vec3 dcd = abs(dw - dc);
            float dca = max(dcd.r, max(dcd.g, dcd.b));
            float dmc = 1.0 - clamp(length(df) * DM / max(res_scale, 0.0001), 0.0, 1.0);
            float dlc = 1.0 - clamp(abs(dot(c, LUMA_BT709) - dot(dc, LUMA_BT709)) * DL, 0.0, 1.0);
            float dcf = dmc * dlc * (1.0 - clamp(dca * 8.0, 0.0, 1.0));
            c = mix(c, dc, mix(DN, DX, dcf * dcf));
        }
    #endif

    #ifdef ENABLE_EDGE_RECONSTRUCT_SMOOTH
        {
            const float EB2 = 0.6;
            const float EN = 0.08;
            const float EM2 = 10.0;
            const float ES2 = 2.0;
            const float ED = 1.0;
            float egx = dot(tap_1_0 - tap_m1_0, LUMA_BT709);
            float egy = dot(tap_0_1 - tap_0_m1, LUMA_BT709);
            float egm = sqrt(egx * egx + egy * egy);
            vec2 egd = vec2(-egy, egx) / max(egm, 0.0001);
            float eds = ED * res_scale;
            vec3 edp = texture(u_input, v_uv + egd * eds * inv).rgb;
            vec3 edn = texture(u_input, v_uv - egd * eds * inv).rgb;
            vec3 eda = (edp + edn) * 0.5;
            float edw = clamp(egm * ES2, 0.0, 1.0);
            vec3 esp = mix(c, eda, edw * 0.3);
            float el = (c.r + c.g + c.b) / 3.0;
            float elx = ((tap_1_0.r + tap_1_0.g + tap_1_0.b) -
                         (tap_m1_0.r + tap_m1_0.g + tap_m1_0.b)) / 6.0;
            float ely = ((tap_0_1.r + tap_0_1.g + tap_0_1.b) -
                         (tap_0_m1.r + tap_0_m1.g + tap_0_m1.b)) / 6.0;
            vec3 eh = texture(u_history, v_uv).rgb;
            float elt = el - (eh.r + eh.g + eh.b) / 3.0;
            float edn2 = elx * elx + ely * ely;
            vec2 ef = vec2(elx, ely) * (-elt / max(edn2 + 0.001, 0.0001));
            ef = clamp(ef, vec2(-4.0 * res_scale), vec2(4.0 * res_scale));
            vec3 ew = texture(u_history, v_uv + ef * inv).rgb;
            vec3 emn = min(c, min(min(tap_1_0, tap_m1_0), min(tap_0_1, tap_0_m1)));
            vec3 emx = max(c, max(max(tap_1_0, tap_m1_0), max(tap_0_1, tap_0_m1)));
            ew = clamp(ew, emn, emx);
            vec3 edf = abs(esp - ew);
            float emo = max(edf.r, max(edf.g, edf.b));
            float ecf = 1.0 - clamp(emo * EM2, 0.0, 1.0);
            float eoe = clamp(egm * 4.0, 0.0, 1.0);
            float ewt = mix(EB2, EN, 1.0 - ecf) * mix(1.0, 0.5, eoe);
            c = mix(esp, ew, ewt);
        }
    #endif

    #ifdef ENABLE_SURFACE_DISOCCLUSION_GUARD
        {
            const float DT2 = 0.15;
            const float DB2 = 0.5;
            vec3 dh = texture(u_history, v_uv).rgb;
            vec3 dd = abs(c - dh);
            float dm = max(dd.r, max(dd.g, dd.b));
            c = mix(c, dh, DB2 * (1.0 - step(DT2, dm)));
        }
    #endif

    #ifdef ENABLE_CONVERGENT_DETAIL_RECOVERY
        {
            const float TS = 0.3;
            const float TM2 = 10.0;
            vec3 th = texture(u_history, v_uv).rgb;
            vec3 td = th - c;
            float tm = dot(td, td);
            float tw = TS * (1.0 - clamp(tm * TM2, 0.0, 1.0));
            c = clamp(mix(c, th + td * tw, tw), 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_GAUSSIAN_GRAIN
        {
            const float GS2 = 0.05;
            const float GA = 10.0;
            float g1 = max(fract(sin(dot(v_uv, vec2(12.9898, 78.233)) + u_time * GA) * 43758.5453), 0.0001);
            float g2 = fract(sin(dot(v_uv, vec2(39.3468, 11.135)) + u_time * GA) * 24634.6345);
            float gg = sqrt(-2.0 * log(g1)) * cos(6.2831853 * g2);
            c = c + vec3(gg) * GS2;
        }
    #endif

    #ifdef ENABLE_CHROMATIC_ABERRATION
        {
            const float CA = 0.005;
            vec2 cd = (v_uv - vec2(0.5)) * CA;
            c.r = texture(u_input, v_uv + cd).r;
            c.b = texture(u_input, v_uv - cd).b;
        }
    #endif

    #ifdef ENABLE_RED_HALATION
        {
            const float HR = 3.0;
            const float HT = 0.6;
            const float HS2 = 0.5;
            float hr = HR * res_scale;
            vec3 hs = texture(u_input, v_uv + vec2(hr, 0.0) * inv).rgb +
                      texture(u_input, v_uv - vec2(hr, 0.0) * inv).rgb +
                      texture(u_input, v_uv + vec2(0.0, hr) * inv).rgb +
                      texture(u_input, v_uv - vec2(0.0, hr) * inv).rgb;
            c.r = c.r + max(hs.r * 0.25 - HT, 0.0) * HS2;
        }
    #endif

    #ifdef ENABLE_ANAMORPHIC_STREAK
        {
            const float AW = 6.0;
            const float AT = 0.7;
            const float AI = 0.3;
            float aw = AW * res_scale;
            vec3 as2 = vec3(0.0);
            for (int aj = 1; aj <= 6; aj++) {
                float ai = float(aj);
                vec2 ao = vec2(ai * aw, 0.0) * inv;
                vec3 aa = max(texture(u_input, v_uv + ao).rgb - vec3(AT), vec3(0.0));
                vec3 ab = max(texture(u_input, v_uv - ao).rgb - vec3(AT), vec3(0.0));
                as2 += (aa + ab) / ai;
            }
            c = c + as2 * AI * vec3(0.4, 0.5, 1.0);
        }
    #endif

    #ifdef ENABLE_RADIAL_VIGNETTE
        {
            const float VI = 0.3;
            const float VO = 0.8;
            const float VS2 = 0.5;
            c = c * (1.0 - smoothstep(VI, VO, length(v_uv - vec2(0.5))) * VS2);
        }
    #endif

    #ifdef ENABLE_CINEMATIC_LETTERBOX
        {
            const float LR = 2.35;
            float la = u_resolution.x / max(u_resolution.y, 0.0001);
            float lv = clamp(la / LR, 0.0, 1.0);
            float lb = (1.0 - lv) * 0.5;
            c = mix(c, vec3(0.0), step(v_uv.y, lb) + step(1.0 - lb, v_uv.y));
        }
    #endif

    #ifdef ENABLE_ORDERED_DITHER
        {
            const float DS3 = 0.01;
            int di = int(mod(floor(gl_FragCoord.x), 4.0)) + int(mod(floor(gl_FragCoord.y), 4.0)) * 4;
            float dt = (dither_cell(di) + 0.5) / 16.0 - 0.5;
            c = c + vec3(dt) * DS3;
        }
    #endif

    #ifdef ENABLE_LINEAR_EXPOSURE
        c = c * 1.3;
    #endif

    #ifdef ENABLE_ACES_TONEMAP
        {
            const float A = 2.51;
            const float B = 0.03;
            const float C = 2.43;
            const float D = 0.59;
            const float E = 0.14;
            c = clamp((c * (A * c + vec3(B))) / max(c * (C * c + vec3(D)) + vec3(E), vec3(0.0001)),
                      0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_AGX_TONEMAP
        {
            const mat3 AGX_IN = mat3(
                0.842479062253094, 0.0423282422610123, 0.0423756549057051,
                0.0784335999999992, 0.878468636469772, 0.0784336,
                0.0792237451477643, 0.0791661274605434, 0.879142973793104);
            const mat3 AGX_OUT = mat3(
                1.19687900512017, -0.0528968517574562, -0.0529716355144438,
                -0.0980208811401368, 1.15190312990417, -0.0980434501171241,
                -0.0990297440797205, -0.0989611768448433, 1.15107367264116);
            const float MN = -12.47393;
            const float MX = 4.026069;
            vec3 av = AGX_IN * c;
            av = clamp(log2(max(av, vec3(0.0001))), MN, MX);
            av = (av - MN) / (MX - MN);
            av = agx_contrast(av);
            c = clamp(AGX_OUT * av, 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_REINHARD_TONEMAP
        {
            const float RW = 4.0;
            c = c * (vec3(1.0) + c / max(RW * RW, 0.0001)) / (vec3(1.0) + c);
        }
    #endif

    #ifdef ENABLE_HABLE_TONEMAP
        {
            const float HW = 11.2;
            const float HE = 2.0;
            vec3 hn = hable_map(c * HE);
            vec3 hd = hable_map(vec3(HW));
            c = clamp(hn / max(hd, vec3(0.0001)), 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_LOTTES_TONEMAP
        {
            const float LA = 1.6;
            const float LB = 0.977;
            const float LC = 0.18;
            const float LD = 0.93;
            vec3 lx = max(c, vec3(0.0001));
            vec3 la = pow(lx, vec3(LA));
            c = la / max(pow(lx, vec3(LA * LD)) * LB + vec3(LC), vec3(0.0001));
        }
    #endif

    #ifdef ENABLE_UCHIMURA_TONEMAP
        c = vec3(uchi_map(c.r), uchi_map(c.g), uchi_map(c.b));
    #endif

    #ifdef ENABLE_TONY_TONEMAP
        {
            const float TO = 0.155;
            const float TS2 = 1.19;
            vec3 tx = max(c, vec3(0.0));
            c = clamp(tx / (tx + vec3(TO)) * TS2, 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_KHRONOS_TONEMAP
        {
            const float KSC = 0.76;
            const float KDS = 0.15;
            float kx = min(c.r, min(c.g, c.b));
            float ko = mix(0.04, kx - 6.25 * kx * kx, step(kx, 0.08));
            c -= ko;
            float kp = max(c.r, max(c.g, c.b));
            float kd = 1.0 - KSC;
            float knp = 1.0 - kd * kd / max(kp + kd - KSC, 0.0001);
            float kgate = step(KSC, kp);
            c *= mix(1.0, knp / max(kp, 0.0001), kgate);
            float kg = 1.0 - 1.0 / max(KDS * (kp - knp) + 1.0, 0.0001);
            c = clamp(mix(c, vec3(knp), kgate * kg), 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_NEUTRAL_WHITE_BALANCE
        {
            const float NR = 0.985;
            const float NG = 1.000;
            const float NB = 1.030;
            c = c * vec3(NR, NG, NB);
        }
    #endif

    #ifdef ENABLE_WARM_TEMPERATURE
        c = c * vec3(1.05, 1.0, 0.92);
    #endif

    #ifdef ENABLE_COOL_TEMPERATURE
        c = c * vec3(0.92, 1.0, 1.05);
    #endif

    #ifdef ENABLE_SATURATION_CONTRAST_GRADE
        {
            const float GS = 1.1;
            const float GC = 1.05;
            const float GB = 0.0;
            float gl = dot(c, LUMA_BT601);
            c = mix(vec3(gl), c, GS);
            c = (c - vec3(0.5)) * GC + vec3(0.5) + vec3(GB);
        }
    #endif

    #ifdef ENABLE_LEVELS_REMAP
        {
            const float IB = 0.02;
            const float IW = 0.98;
            const float OB = 0.0;
            const float OW = 1.0;
            c = clamp((c - vec3(IB)) / max(vec3(IW - IB), vec3(0.0001)), 0.0, 1.0) *
                (OW - OB) + vec3(OB);
        }
    #endif

    #ifdef ENABLE_GAMMA_CORRECT
        {
            const float GV = 1.1;
            c = pow(max(c, vec3(0.0)), vec3(1.0 / max(GV, 0.0001)));
        }
    #endif

    #ifdef ENABLE_VIBRANCE_BOOST
        {
            const float VA = 0.5;
            float vmx = max(c.r, max(c.g, c.b));
            float vmn = min(c.r, min(c.g, c.b));
            float vst = vmx - vmn;
            float vl = dot(c, LUMA_BT601);
            c = mix(vec3(vl), c, 1.0 + VA * (1.0 - vst));
        }
    #endif

    #ifdef ENABLE_HSL_TRANSFORM
        {
            const float HH = 0.0;
            const float HS2 = 1.1;
            const float HL = 1.0;
            float hmx = max(c.r, max(c.g, c.b));
            float hmn = min(c.r, min(c.g, c.b));
            float hl = (hmx + hmn) * 0.5;
            float hd = hmx - hmn;
            float is_r = step(hmx - c.r, 0.0);
            float is_g = step(hmx - c.g, 0.0) * (1.0 - is_r);
            float is_b = (1.0 - is_r) * (1.0 - is_g);
            float hh_r = mod((c.g - c.b) / max(hd, 0.0001), 6.0);
            float hh_g = (c.b - c.r) / max(hd, 0.0001) + 2.0;
            float hh_b = (c.r - c.g) / max(hd, 0.0001) + 4.0;
            float hh = (hh_r * is_r + hh_g * is_g + hh_b * is_b) / 6.0;
            hh = mix(hh, 0.0, step(hd, 0.0001));
            hh = fract(hh + HH);
            float hs = clamp(hd / max(1.0 - abs(2.0 * hl - 1.0), 0.0001) * HS2, 0.0, 1.0);
            float hl2 = clamp(hl * HL, 0.0, 1.0);
            float hc = (1.0 - abs(2.0 * hl2 - 1.0)) * hs;
            float hx = hc * (1.0 - abs(mod(hh * 6.0, 2.0) - 1.0));
            float hm = hl2 - hc * 0.5;
            vec3 hr = vec3(hc, hx, 0.0);
            hr = mix(hr, vec3(hx, hc, 0.0), step(1.0, hh * 6.0));
            hr = mix(hr, vec3(0.0, hc, hx), step(2.0, hh * 6.0));
            hr = mix(hr, vec3(0.0, hx, hc), step(3.0, hh * 6.0));
            hr = mix(hr, vec3(hx, 0.0, hc), step(4.0, hh * 6.0));
            hr = mix(hr, vec3(hc, 0.0, hx), step(5.0, hh * 6.0));
            c = hr + vec3(hm);
        }
    #endif

    #ifdef ENABLE_SPLIT_TONE
        {
            const float SB2 = 0.3;
            float sl = dot(c, LUMA_BT601);
            c = c + mix(vec3(-0.1, 0.0, 0.1), vec3(0.1, 0.0, -0.1),
                        smoothstep(0.0, 1.0, sl)) * SB2;
        }
    #endif

    #ifdef ENABLE_LIFT_GAMMA_GAIN
        {
            const float LL = 0.03;
            const float LG2 = 1.0;
            const float LN = 1.0;
            c = pow(max(c * LN + vec3(LL) * (vec3(1.0) - c), vec3(0.0)),
                    vec3(1.0 / max(LG2, 0.0001)));
        }
    #endif

    #ifdef ENABLE_HERMITE_CURVES
        {
            const float CS = 0.5;
            vec3 cv = c * c * (vec3(3.0) - 2.0 * c);
            c = mix(c, cv, CS);
        }
    #endif

    #ifdef ENABLE_RED_CHANNEL_CURVE
        {
            const float RS = 0.5;
            float rv = c.r * c.r * (3.0 - 2.0 * c.r);
            c.r = mix(c.r, rv, RS);
        }
    #endif

    #ifdef ENABLE_GREEN_CHANNEL_CURVE
        {
            const float GS2 = 0.5;
            float gv = c.g * c.g * (3.0 - 2.0 * c.g);
            c.g = mix(c.g, gv, GS2);
        }
    #endif

    #ifdef ENABLE_BLUE_CHANNEL_CURVE
        {
            const float BS = 0.5;
            float bv = c.b * c.b * (3.0 - 2.0 * c.b);
            c.b = mix(c.b, bv, BS);
        }
    #endif

    #ifdef ENABLE_TRIZONE_COLOR_BALANCE
        {
            const float SE = 0.4;
            const float HS3 = 0.6;
            float cl = dot(c, LUMA_BT601);
            float cs = 1.0 - smoothstep(0.0, SE, cl);
            float ch = smoothstep(HS3, 1.0, cl);
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
        {
            const float DB = 0.03;
            const float DW = 0.97;
            c = (clamp(c, vec3(DB), vec3(DW)) - vec3(DB)) /
                max(vec3(DW - DB), vec3(0.0001));
        }
    #endif

    #ifdef ENABLE_DUOTONE_MAP
        {
            float dl = dot(c, LUMA_BT601);
            c = mix(vec3(0.1, 0.1, 0.3), vec3(1.0, 0.9, 0.7), dl);
        }
    #endif

    #ifdef ENABLE_COLOR_WASH_TINT
        {
            c = mix(c, vec3(0.5, 0.5, 0.6), 0.2);
        }
    #endif

    #ifdef ENABLE_POSTERIZE_QUANTIZE
        {
            const float PL = 8.0;
            c = floor(c * max(PL, 1.0)) / max(PL, 1.0);
        }
    #endif

    #ifdef ENABLE_BLEACH_BYPASS
        {
            const float BB2 = 0.7;
            float bl = dot(c, LUMA_BT601);
            vec3 bb = 2.0 * c * vec3(bl);
            vec3 bs = vec3(1.0) - 2.0 * (vec3(1.0) - c) * (vec3(1.0) - vec3(bl));
            c = mix(c, mix(bb, bs, step(0.5, bl)), BB2);
        }
    #endif

    #ifdef ENABLE_TECHNICOLOR_PROCESS
        {
            const float TS3 = 0.5;
            const float TT = 0.7;
            vec3 tn = vec3(1.0) - c;
            vec3 tp = vec3(tn.g + tn.b, tn.r + tn.b, tn.r + tn.g) * TS3;
            c = mix(c, clamp(vec3(1.0) - tp * 0.5, 0.0, 1.0), TT);
        }
    #endif

    #ifdef ENABLE_MIDPOINT_CONTRAST
        {
            c = clamp((c - vec3(0.5)) * 1.5 + vec3(0.5), 0.0, 1.0);
        }
    #endif

    #ifdef ENABLE_COLOR_INVERT
        c = vec3(1.0) - c;
    #endif

    #ifdef ENABLE_LUMINANCE_GRAYSCALE
        {
            float gl = dot(c, LUMA_BT709);
            c = vec3(gl);
        }
    #endif

    #ifdef ENABLE_PROTANOPIA_SIMULATE
        c = vec3(dot(c, vec3(0.567, 0.433, 0.0)),
                 dot(c, vec3(0.558, 0.442, 0.0)),
                 dot(c, vec3(0.0, 0.242, 0.758)));
    #endif

    #ifdef ENABLE_DEUTERANOPIA_SIMULATE
        c = vec3(dot(c, vec3(0.625, 0.375, 0.0)),
                 dot(c, vec3(0.7, 0.3, 0.0)),
                 dot(c, vec3(0.0, 0.3, 0.7)));
    #endif

    #ifdef ENABLE_TRITANOPIA_SIMULATE
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

    frag_out = vec4(c, 1.0);
}
"#;
