> [!NOTE]
> Bug reports and pull requests are welcome, but please understand that development happens in my free time and progress may be slow at times. The project is still maintained even if the last commit was made a while ago.

# Bones:

**A GLSL ubershader post‑processing tool for Linux games (OpenGL · OpenGL ES · Vulkan).**

Bones is a realtime post‑processing tool written in Rust. It intercepts OpenGL and Vulkan buffer swaps, runs a single‑pass ubershader containing 125 effects, and presents the modified frame. It is built for **performance** and **simplicity**: no external shader loading, no custom LUTs, no user‑tunable parameters, no ping‑pong. Every enabled effect is compiled into one shader pass, so VRAM usage and draw‑call count stay constant no matter how many effects you turn on.

### At a glance:

- **125 effects** across 20 categories, geometry, AA, sharpening, 22 temporal modes, 9 console GPU simulations (PS1 through Xbox 360), CRT/OLED/VHS, colour grading, colourblind correction, and more.
- **One shader, one draw call**, effects are `#ifdef`‑toggled in a single ubershader. VRAM is O(1) (three textures total).
- **OpenGL + Vulkan** in one tool, native GLX/EGL hooking *and* an explicit Vulkan layer, simultaneously.
- **Hot reload**, edit the config while the game runs; the shader recompiles live.
- **Reproducible launches**, define an entire effect stack in a single environment variable.
- **Flatpak support**, first‑class, via a runtime extension.

## Quick Start:

```sh
# 1. Build and install
git clone https://github.com/pythonlover02/bones.git
cd bones
make
sudo make install

# 2. Run a game with an effect stack (inline, no config file needed)
BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen;vibrance_boost" bones -- ~/games/mygame
```

Prefer a config file? On first run Bones writes a fully documented `~/.config/bones/bones-config.toml`. Set any effect to `true`, then:

```sh
bones -- ~/games/mygame
```

**Steam (native game)**, set the game Launch Options to:

```
bones -- %command%
```

**Steam (Flatpak game)**, see [Flatpak](#flatpak).

## Table of Contents:

- [Minimum Requirements](#minimum-requirements)
- [How It Works](#how-it-works)
- [Installation](#installation)
- [Flatpak](#flatpak)
- [Usage](#usage)
- [Environment Variables](#environment-variables)
- [Effect Catalogue](#effect-catalogue)
- [Recommended Combinations](#recommended-combinations)
- [Performance & Limitations](#performance--limitations)
- [Credits & License](#credits--license)

## Minimum Requirements:

| Path | Requirement |
|------|-------------|
| **OpenGL** | OpenGL 3.0+ (2008‑era and newer GPUs); Mesa 7.9+ or any proprietary driver; GLX or EGL |
| **Vulkan** | Vulkan 1.0+ with `VK_KHR_swapchain` |
| **OpenGL ES** | OpenGL ES 2.0+ (via EGL) |
| **System** | Linux x86_64; `glibc 2.17`+ for native builds, `glibc 2.36` for portable release builds |

Contexts reporting GL 3.0 or higher automatically satisfy all requirements (core FBO, VAO, GLSL 1.30). Pure GL 2.1 contexts without `GL_ARB_vertex_array_object` have post‑FX silently disabled for that context, the game runs normally, just without effects.

## How It Works:

Bones has two interception paths that run simultaneously:

- **OpenGL**, uses `LD_PRELOAD` to hook `glXSwapBuffers` (GLX) and `eglSwapBuffers` (EGL). Before the real swap, Bones copies the default framebuffer into a texture, draws a full‑screen triangle with the ubershader, and writes the result back.
- **Vulkan**, registers as an explicit layer (`VK_LAYER_BONES_overlay`), enabled via `VK_INSTANCE_LAYERS`. At `vkQueuePresentKHR`, the layer copies the swapchain image into an offscreen texture, renders the ubershader into a second offscreen image, and copies the result back before presentation.

The launcher activates both paths by setting `LD_PRELOAD`, `VK_ADD_LAYER_PATH`, and `VK_INSTANCE_LAYERS` for the target process. The library and its manifest live in one directory, and the manifest references the library by a relative path, so the loader resolves it without `ldconfig`.

In both paths the pipeline is identical:

1. Copy the current frame into an input texture.
2. Run a single **ubershader** combining all enabled effects.
3. Optionally read a history texture for temporal effects.
4. Write the result back and copy it into the history texture for the next frame.

Unlike traditional reshade pipelines that chain multiple passes (ping‑pong between FBOs), Bones **never uses more than one draw call**. All effects are compiled into one large shader guarded by `#ifdef`. This keeps VRAM constant and avoids multi‑pass overhead.

Effects run in a **fixed order** designed around three semantic stages: *render the ideal image* (geometric warps, denoise, AA, sharpening, blur, image quality, temporal, exposure, tonemap, grading, accessibility), *apply lens/film effects* (inline grain, vignette, dither), then *show it on a chosen monitor* (hardware sims act as the display device). Overlays render last. Enabling two effects from the same category can produce unexpected results, so for best results enable only one per category unless they are designed to combine.

> [!NOTE]
> Hardware simulations are **terminal display effects**. AA, sharpen, and TAA run *before* the sim, so they operate on the ideal frame; the sim then interprets that frame as if it were going through PS1/CRT/VHS hardware. This matches how real emulator shader chains work.

## Installation:

### Pre‑built binaries (if available):

Download the latest release from the [Releases](../../releases) page, extract it, and from inside the extracted directory run:

```sh
sudo make install
```

This installs the `bones` launcher, `libbones.so`, the Vulkan layer manifest, and any bundled Flatpak extensions.

### Building from source:

```sh
git clone https://github.com/pythonlover02/bones.git
cd bones
make
```

Produces `target/release/bones` and `target/release/libbones.so`.

### Portable release build:

Builds the library inside a Debian Bookworm container (for maximum glibc compatibility) plus all Flatpak extensions. Requires `podman` or `docker`, `flatpak`, `ostree`,and `python3`.

```sh
make release
```

### Installing:

Run as root after building:

```sh
sudo make install
```

| File | Default path |
|------|--------------|
| Launcher | `/usr/local/bin/bones` |
| Library | `/usr/local/lib/bones/libbones.so` |
| Vulkan layer manifest | `/usr/local/lib/bones/VkLayer_bones.json` |

Library and manifest are installed together in `/usr/local/lib/bones/`; the launcher points the Vulkan loader at this directory at runtime, so no `ldconfig` step is needed.

```sh
sudo make install PREFIX=/opt/bones    # change prefix
sudo make install DESTDIR=./package    # stage into a directory (packaging)
```

If Flatpak extensions were built beforehand (`make flatpak` or `make release`), they are installed per‑user for the invoking user as part of `sudo make install`.

### Integrated build (for Proton forks and custom launchers):

If you only need the library and manifest (no launcher), for example, to wire Bones into a Proton fork own launch script, use:

```sh
make integrated
```

This builds the library and stages everything you need into `target/release/integrated/`:

```
target/release/integrated/
├── libbones.so
├── VkLayer_bones.json
├── LICENSE
└── dist.LICENSE
```

Nothing is installed system‑wide. Copy the contents of that directory wherever your launcher expects them, then set `LD_PRELOAD`, `VK_ADD_LAYER_PATH`, and `VK_INSTANCE_LAYERS` to point at the destination. Shipping the two license files alongside the binary keeps redistribution GPL compliant out of the box.

### Uninstalling:

```sh
sudo make remove
```

Removes the launcher, library, and manifest; uninstalls the Flatpak extension (system scope always, user scope for the invoking user); and removes `~/.config/bones`. If run directly as root without `sudo`, it prints the remaining `--user` cleanup commands, since it cannot determine which user Flatpak and config to clean.

### Cleaning build artifacts:

```sh
make clean          # cargo clean + remove built Flatpak bundles and staging
make flatpak-clean  # remove only Flatpak bundles
```

## Flatpak:

Flatpak applications run sandboxed and cannot see `LD_PRELOAD`, `VK_ADD_LAYER_PATH`, or `VK_INSTANCE_LAYERS` set on the host, those variables do not cross the sandbox boundary. Bones supports Flatpak through a **Flatpak extension**: a bundle that mounts the library, the layer manifest, and a small wrapper script (`bones-flatpak`) inside the `org.freedesktop.Platform` runtime, so they are reachable from within the sandbox. The wrapper sets the activation environment **from inside** the sandbox, the only place it survives.

Extensions are built for runtime versions `23.08`, `24.08`, and `25.08`.

### Building the extensions:

Requires `flatpak`, `ostree`, `python3`, and a completed `make`/`make release` build.

```sh
make
make flatpak
```

This produces one bundle per supported runtime:

```
org.freedesktop.Platform.VulkanLayer.bones-23.08.flatpak
org.freedesktop.Platform.VulkanLayer.bones-24.08.flatpak
org.freedesktop.Platform.VulkanLayer.bones-25.08.flatpak
```

`make release` builds all of these at once.

### Installing the extensions:

Install the bundle matching your `org.freedesktop.Platform` runtime. Run `flatpak list` and look for `org.freedesktop.Platform` if unsure.

```sh
flatpak install --user org.freedesktop.Platform.VulkanLayer.bones-24.08.flatpak
```

Multiple runtime versions can coexist. If you ran `sudo make install` with the extensions already built, they are installed automatically for the invoking user.

```sh
flatpak uninstall --user org.freedesktop.Platform.VulkanLayer.bones
```

### Running a Flatpak application through Bones:

```sh
bones -- flatpak run com.example.Game
bones -- flatpak run --branch=stable com.example.Game   # flags pass through
bones myprofile -- flatpak run com.example.Game          # named profile
```

Bones detects the `flatpak run` invocation, reads the application metadata to find its entry point, and rewrites the command to run `bones-flatpak` inside the sandbox, which sets the environment and execs the app.

### Steam launch option (Flatpak):

Set the game **Launch Options** to the in‑sandbox wrapper absolute path:

```
/usr/lib/extensions/vulkan/bones/bin/bones-flatpak %command%
```

With a named profile:

```
BONES_CONFIG_NAME=myprofile /usr/lib/extensions/vulkan/bones/bin/bones-flatpak %command%
```

This requires the Bones Flatpak extension matching the game runtime to be installed.

## Usage:

### Profiles:

A profile is a named config at `~/.config/bones/<name>-config.toml`. The default profile is `bones` (`bones-config.toml`). Pass a name to load a different one:

```sh
bones retro -- ~/games/retro-game   # loads ~/.config/bones/retro-config.toml
```

### Configuration file:

The default config is generated at `~/.config/bones/bones-config.toml` on first run, with every effect listed and documented. Set any effect to `true` under its category section (e.g. `mirror_horizontal = true` under `[geometric]`), and `hot_reload = true` under `[general]` for live reloading. Effects are **toggle‑only**, all parameters are baked into the shader.

### Inline configuration (`BONES_CONFIG`):

Pass enabled effects directly as a semicolon‑separated list. When set, it **overrides the config file entirely**, disables hot reload, and enables exactly the listed effects:

```sh
BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen;vibrance_boost" bones -- ~/games/game
```

- Setting `BONES_CONFIG`, even to an empty string, takes over from the file. An empty value means "no effects" (a forced clean pass).
- Unknown effect names are ignored with a warning.
- Hot reload is always off when `BONES_CONFIG` is used.
- Works with Flatpak too: pass it on the `bones -- flatpak run …` line and it is forwarded into the sandbox.

### Hot reload:

With `hot_reload = true`, Bones uses `inotify` to watch the config directory. Save changes and the shader recompiles and reloads without restarting the game. If a reload fails to compile, the previous working shader is kept.

## Environment Variables:

| Variable | Purpose | Values | Default |
|----------|---------|--------|---------|
| `BONES_CONFIG_NAME` | Profile to load | any profile name | `bones` |
| `BONES_CONFIG` | Inline effect list (overrides the file) | semicolon‑separated effect names | *(unset → use file)* |
| `BONES_LOG` | Log verbosity (written to stderr) | `off`, `error`, `warn`, `info` | `warn` |

`LD_PRELOAD`, `VK_ADD_LAYER_PATH`, and `VK_INSTANCE_LAYERS` are set automatically by the launcher; you only set them manually with the `integrated` build.

## Effect Catalogue:

125 effects in 21 categories, applied in the fixed order below. The selection column is the rule of thumb, see the note after the table.

| Order | Category | Section | Count | What it does | Selection |
|------:|----------|---------|------:|--------------|-----------|
| 1 | Geometric | `[geometric]` | 12 | UV / coordinate warps | mostly combinable |
| 2 | Denoise | `[denoise]` | 1 | clean noise before processing | single |
| 3 | Anti‑aliasing | `[anti_aliasing]` | 4 | smooth jagged edges | **pick one** |
| 4 | Sharpening | `[sharpening]` | 9 | enhance detail | combinable* |
| 5 | Local contrast | `[local_contrast]` | 1 | per‑pixel "pop" | single |
| 6 | Blur | `[blur]` | 5 | creative / soften | pick one |
| 7 | Image quality | `[image_quality]` | 3 | deband, bloom, lens flare | combinable |
| 8 | Temporal | `[temporal]` | 22 | frame blending / TAA | **one primary** (guards auto) |
| 9 | Exposure | `[exposure]` | 1 | pre‑tonemap exposure | single |
| 10 | Tonemapping | `[tonemapping]` | 8 | HDR → SDR curves | **pick one** |
| 11 | White balance | `[white_balance]` | 3 | colour temperature | **pick one** |
| 12 | Colour grading | `[color_grading]` | 8 | grade / saturation / curves | combinable |
| 13 | Channel curves | `[channel_curves]` | 3 | per‑channel S‑curves | combinable |
| 14 | Colour balance | `[color_balance]` | 1 | teal/orange tint | single |
| 15 | Selective colour | `[selective_color]` | 3 | per‑channel saturation | combinable |
| 16 | Stylization | `[stylization]` | 9 | creative looks | mostly pick one |
| 17 | Accessibility | `[accessibility]` | 6 | CVD simulate / correct | pick one |
| 18 | Inline | `[inline]` | 7 | grain, vignette, letterbox… | stackable |
| 19 | Hardware simulation | `[hardware_simulation]` | 17 | Console GPU sims + CRT / OLED / VHS | one console + one display |
| 20 | Overlay (HUD) | `[overlay]` | 2 | FPS, crosshair | combinable |

Sharpeners combine (e.g. one sharpener + `midtone_clarity`), but stacking multiple *strong* sharpeners produces halos.

> [!IMPORTANT]
> **Enable at most one effect per category unless noted otherwise.** Dont run two tonemappers or two AA methods. For temporal, pick one primary mode; a fused stabilizer (`convergent_detail_recovery`) activates automatically whenever any temporal mode is enabled, combining a lightweight disocclusion gate with a convergence pull. For hardware simulation, pick one console sim and optionally one display sim.

> [!NOTE]
> Combining a temporal mode with a hardware sim is supported but the history texture captures the post‑sim image; TAA reads PS1/CRT distorted history on the next frame. This produces a "weird but not broken" look. If you want clean TAA, dont stack a sim on top.

<details>
<summary><b>1. Geometric</b>, UV warps (12)</summary>

| Effect | Description |
|--------|-------------|
| `identity` | No‑op passthrough (baseline/debug) |
| `mirror_horizontal` | Flip left ↔ right |
| `mirror_vertical` | Flip top ↔ bottom |
| `rotate_90` | Rotate 90° clockwise |
| `rotate_180` | Rotate 180° |
| `rotate_270` | Rotate 270° clockwise |
| `center_zoom` | 1.5× magnify from screen centre |
| `polynomial_distort` | Brown–Conrady barrel lens distortion (k1/k2) |
| `barrel_undistort` | Inverse barrel correction (straighten curves) |
| `fisheye_warp` | Wide‑angle fisheye projection (atan remap) |
| `trapezoid_warp` | Perspective keystone (wider at bottom) |
| `sharp_bilinear` | Pixel‑art sharpener via Hermite UV snap |
</details>

<details>
<summary><b>2. Denoise</b> (1)</summary>

| Effect | Description |
|--------|-------------|
| `bilateral_denoise` | Edge‑preserving noise reduction (Tomasi & Manduchi 1998) |
</details>

<details>
<summary><b>3. Anti‑aliasing</b>, pick one (4)</summary>

| Effect | Description |
|--------|-------------|
| `luma_edge_aa` | FXAA (Timothy Lottes, NVIDIA) |
| `normal_filter_aa` | NFAA, lightweight gradient AA |
| `morphological_aa` | Conservative CMAA‑style AA |
| `subpixel_aa` | SMAA‑style subpixel morphological AA |
</details>

<details>
<summary><b>4. Sharpening</b>, combinable, dont stack strong ones (9)</summary>

| Effect | Description |
|--------|-------------|
| `contrast_adaptive_sharpen` | AMD CAS (FidelityFX) |
| `robust_contrast_sharpen` | AMD RCAS (FSR 1.0), halo‑free |
| `edge_directed_sharpen` | NVIDIA NIS‑style edge‑aware sharpen |
| `laplacian_sharpen` | Classic simple sharpen |
| `luminance_sharpen` | Luma‑only, no colour fringing |
| `midtone_clarity` | Lightroom‑style "Clarity" |
| `falloff_sharpen` | Adaptive, gated by local edge energy |
| `power_curve_sharpen` | Filmic power‑curve response |
| `unsharp_mask` | Classic darkroom unsharp mask |
</details>

<details>
<summary><b>5. Local contrast</b> (1)</summary>

| Effect | Description |
|--------|-------------|
| `local_contrast` | Per‑pixel local contrast enhancement |
</details>

<details>
<summary><b>6. Blur</b>, pick one (5)</summary>

| Effect | Description |
|--------|-------------|
| `gaussian_blur` | 3×3 Gaussian |
| `box_blur` | Kawase dual‑filter wide blur |
| `bokeh_blur` | Circular DoF with highlight bokeh |
| `tilt_shift_blur` | Miniature/diorama band blur |
| `radial_blur` | Zoom/speed blur from centre |
</details>

<details>
<summary><b>7. Image quality</b>, combinable (3)</summary>

| Effect | Description |
|--------|-------------|
| `gradient_deband` | Remove banding in gradients (mpv‑style) |
| `threshold_bloom` | Bright‑pixel glow |
| `ghost_flare` | Lens‑flare ghost reflections |
</details>

<details>
<summary><b>8. Temporal</b>, one primary, guards automatic (22)</summary>

22 primary frame blending modes. A fused stabilizer (`convergent_detail_recovery`) activates automatically whenever any temporal mode is enabled; it gates history influence on large per‑pixel changes (acting as a lightweight disocclusion guard) and pulls the current frame toward the converged history to recover detail. Ordered roughly simplest to most sophisticated. I mostly use them on my potato PC.

| Effect | Description |
|--------|-------------|
| `neighborhood_clamp_aa` | TAA, neighbourhood min/max clamp |
| `motion_reject_denoise` | Motion‑gated accumulation |
| `motion_detect_blur` | Temporal motion blur |
| `constant_blend_smooth` | 50/50 blend (baseline) |
| `shutter_angle_smooth` | 180° camera shutter simulation |
| `spline_interp_smooth` | Cubic Hermite temporal reconstruction |
| `variance_decay_smooth` | Variance‑gated IIR filter |
| `dualrate_smooth` | Dual‑rate adaptive blend |
| `luminance_gate_smooth` | Smooth dark areas more (scotopic vision) |
| `contrast_gate_smooth` | Smooth low‑contrast areas more |
| `gradient_gate_smooth` | Edge‑aware, strong single anti‑ghosting technique |
| `sigma_clip_smooth` | Variance clipping (modern TAA core, Salvi 2016) |
| `mitchell_kernel_smooth` | Mitchell–Netravali kernel + temporal |
| `ycocg_clip_smooth` | YCoCg AABB clamp, the technique at the core of FSR 2 |
| `bilateral_history_smooth` | Bilateral filter applied in time |
| `perceptual_chroma_smooth` | YCbCr, aggressive chroma smoothing |
| `frequency_split_smooth` | Low/high frequency band split blend |
| `horn_schunck_smooth` | Optical‑flow warp (Horn & Schunck 1981) |
| `convergent_accumulate` | Detail accumulation, inspired by the principle behind DLSS 2 |
| `dualwarp_flow_smooth` | Dual‑warp interpolation, inspired by FSR 3 frame generation |
| `variance_flow_accumulate` | Motion‑compensated, triple‑gated, inspired by the research behind DLSS 2 |
| `edge_reconstruct_smooth` | Edge‑directed reconstruction, inspired by XeSS DP4a path |
</details>

<details>
<summary><b>9. Exposure</b> (1)</summary>

| Effect | Description |
|--------|-------------|
| `linear_exposure` | Pre‑tonemap exposure multiplier |
</details>

<details>
<summary><b>10. Tonemapping</b>, pick one (8)</summary>

| Effect | Description |
|--------|-------------|
| `aces_tonemap` | ACES filmic (Narkowicz 2015) |
| `agx_tonemap` | AgX (Blender default), neutral |
| `reinhard_tonemap` | Reinhard extended (white point 4.0) |
| `hable_tonemap` | Uncharted 2 filmic (John Hable) |
| `lottes_tonemap` | Lottes tunable curve |
| `uchimura_tonemap` | Gran Turismo curve (Uchimura) |
| `tony_tonemap` | Tony McMapface, lightweight |
| `khronos_tonemap` | Khronos PBR Neutral |
</details>

<details>
<summary><b>11. White balance</b>, pick one (3)</summary>

| Effect | Description |
|--------|-------------|
| `neutral_white_balance` | D65 reference correction |
| `warm_temperature` | Warm shift (tungsten) |
| `cool_temperature` | Cool shift (daylight) |
</details>

<details>
<summary><b>12. Colour grading</b>, combinable (8)</summary>

| Effect | Description |
|--------|-------------|
| `saturation_contrast_grade` | Combined auto‑enhance pass |
| `levels_remap` | Black/white point remap |
| `gamma_correct` | Gamma power curve |
| `vibrance_boost` | Smart saturation (Lightroom Vibrance) |
| `hsl_transform` | Hue / saturation / lightness in HSL |
| `split_tone` | Orange‑and‑teal split toning |
| `lift_gamma_gain` | Three‑way colour corrector |
| `hermite_curves` | Smooth S‑curve contrast |
</details>

<details>
<summary><b>13. Channel curves</b>, combinable (3)</summary>

| Effect | Description |
|--------|-------------|
| `red_channel_curve` | S‑curve on red |
| `green_channel_curve` | S‑curve on green |
| `blue_channel_curve` | S‑curve on blue |
</details>

<details>
<summary><b>14. Colour balance</b> (1)</summary>

| Effect | Description |
|--------|-------------|
| `trizone_color_balance` | Three‑zone teal/orange tint |
</details>

<details>
<summary><b>15. Selective colour</b>, combinable (3)</summary>

| Effect | Description |
|--------|-------------|
| `red_selective_saturate` | Boost saturation of red‑dominant pixels |
| `green_selective_saturate` | Boost saturation of green‑dominant pixels |
| `blue_selective_saturate` | Boost saturation of blue‑dominant pixels |
</details>

<details>
<summary><b>16. Stylization</b>, mostly pick one (9)</summary>

| Effect | Description |
|--------|-------------|
| `dynamic_range_crush` | Flat "Instagram" range crush |
| `duotone_map` | Two‑colour luminance map |
| `color_wash_tint` | Subtle palette‑unifying tint |
| `posterize_quantize` | 8‑level posterize |
| `bleach_bypass` | Desaturated high‑contrast film look |
| `technicolor_process` | Three‑strip Technicolor |
| `midpoint_contrast` | Simple contrast boost |
| `color_invert` | Negative image |
| `luminance_grayscale` | BT.709 grayscale |
</details>

<details>
<summary><b>17. Accessibility</b>, colour vision (6)</summary>

| Effect | Description |
|--------|-------------|
| `protanopia_simulation` | Simulate red‑blind vision |
| `deuteranopia_simulation` | Simulate green‑blind vision |
| `tritanopia_simulation` | Simulate blue‑blind vision |
| `protanopia_correct` | Daltonize for protanopia |
| `deuteranopia_correct` | Daltonize for deuteranopia |
| `tritanopia_correct` | Daltonize for tritanopia |
</details>

<details>
<summary><b>18. Inline</b>, stackable (7)</summary>

Lens and film effects applied after colour grading, before hardware simulation. These belong to the source image the virtual monitor receives.

| Effect | Description |
|--------|-------------|
| `gaussian_grain` | Animated film grain (Box–Muller noise) |
| `chromatic_aberration` | Lateral colour fringing |
| `red_halation` | Red highlight bloom (film halation) |
| `anamorphic_streak` | Horizontal blue lens streaks |
| `radial_vignette` | Darkened edges |
| `cinematic_letterbox` | 2.35:1 black bars |
| `ordered_dither` | Bayer 4×4 ordered dithering |
</details>

<details>
<summary><b>19. Hardware simulation</b>, one console + one display (17)</summary>

Hardware sims act as the **display device** showing the finished image. AA and sharpen run before the sim so they see the ideal frame.

**Console GPU simulation** (pick one):

| Effect | Description |
|--------|-------------|
| `ps1_simulation` | PS1, UV grid quantization, nearest‑neighbour feel, 15‑bit colour with Bayer dither |
| `saturn_simulation` | Saturn, luminance banding, dark desaturated palette, warm brown shift |
| `n64_simulation` | N64, low‑res quantization softness, radial fog, warm shift |
| `dreamcast_simulation` | Dreamcast, over‑bright response, boosted saturation, specular highlights |
| `ps2_simulation` | PS2, pixel quantization, soft scanline modulation, threshold bloom |
| `xbox_simulation` | Xbox, highlight boost, midtone lift bloom, plastic specular, warm/green bias |
| `psp_simulation` | PSP, 480×272 phase, LCD washout gamma, dark banding, slight desaturation |
| `ps3_simulation` | PS3, sub‑HD quantization, crushed shadows, cool shift |
| `xbox360_simulation` | Xbox 360, eDRAM tile seams, HDR banding, lifted warm gamma, desat with specular boost |

**Display simulation** (pick one, combinable with console sims):

| Effect | Description |
|--------|-------------|
| `crt_simulation` | CRT, barrel warp + RGB phosphor mask + scanlines + brightness pulsing |
| `phosphor_amber` | Amber monochrome CRT |
| `phosphor_green` | Green monochrome CRT (classic terminal) |
| `phosphor_red` | Red monochrome CRT |
| `scanline_darken` | Scanlines only (no warp/mask) |
| `oled_simulation` | Black crush + saturation boost |
| `vhs_simulation` | VHS, horizontal ripple, per‑line tracking noise, luma noise, desaturation, warm shift, dropout |
| `lcd_subpixel` | Visible RGB subpixel grid |
</details>

<details>
<summary><b>20. Overlay (HUD)</b>, combinable (2)</summary>

Overlays render after all processing (temporal, colour grading, accessibility, hardware sims) so they are never affected by other effects.

| Effect | Description |
|--------|-------------|
| `fps_hud` | On‑screen FPS counter (7‑segment) |
| `crosshair_overlay` | Centred neon crosshair |
</details>

## Recommended Combinations:

Copy‑paste these as `BONES_CONFIG` values, or set the same keys in your config file. Each respects the one‑per‑category rule.

```sh
# Crisp modern look, AA, sharpen, smart saturation
BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen;vibrance_boost" bones -- ~/games/game

# Cinematic, filmic tonemap, split tone, and layered inline effects
BONES_CONFIG="hable_tonemap;split_tone;radial_vignette;cinematic_letterbox;gaussian_grain" bones -- ~/games/game

# PS1 on a CRT
BONES_CONFIG="ps1_simulation;crt_simulation" bones -- ~/games/game

# N64 on a CRT
BONES_CONFIG="n64_simulation;crt_simulation" bones -- ~/games/game

# PSP handheld look
BONES_CONFIG="psp_simulation;lcd_subpixel" bones -- ~/games/game

# Anti‑flicker temporal stability (stabilizer activates automatically)
BONES_CONFIG="gradient_gate_smooth" bones -- ~/games/game

# VHS found footage
BONES_CONFIG="vhs_simulation" bones -- ~/games/game

# Colourblind correction (deuteranopia)
BONES_CONFIG="deuteranopia_correct" bones -- ~/games/game
```

The cinematic example deliberately stacks several `[inline]` effects, grain, vignette, and letterbox operate independently and are designed to layer.

## Performance & Limitations

- **VRAM**: O(1), only three textures (input, output, history) regardless of effect count.
- **Draw calls**: always one full‑screen triangle.
- **CPU overhead**: minimal.

Limitations:

- No custom shaders or LUTs.
- No multi‑pass effects (e.g. advanced bloom with downsample chains).
- Per‑object motion vectors (as in true DLSS) are approximated by optical flow from neighbouring pixels.
- Fast motion can cause ghosting on some temporal modes; the automatic `convergent_detail_recovery` stabilizer includes a motion gate that softens history influence on large per‑pixel changes to reduce it.
- Combining a temporal mode with a hardware sim makes the history texture capture the post‑sim image, which the next frame reads back through TAA; the result is stable but visually unusual.
- OpenGL contexts below 3.0 without `GL_ARB_vertex_array_object` have post‑FX disabled for that context; the application still runs.

The effect order is fixed and cannot be changed at runtime.

## Credits & License

Heavily inspired by existing reshade tools, but the ubershader implementation and the majority of the effect code were written from scratch (or ported from GLSL snippets in public‑domain and open‑source shader repositories).

All 125 effect implementations are original or derived from well‑known algorithms (FXAA by Timothy Lottes, CAS/RCAS from AMD GPUOpen, AgX, ACES, and others) under their respective licenses (mostly MIT/BSD). The project itself is released under the **GNU General Public License v3.0**, full text in [LICENSE](LICENSE).
