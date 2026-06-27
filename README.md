> [!NOTE]
> Bug reports and pull requests are welcome, but please understand that development happens in my free time and progress may be slow at times. The project is still maintained even if the last commit was made a while ago.

# Bones

**Performance first Vulkan ubershader Post Processing layer for Linux.**

Bones is a realtime post-processing layer for Vulkan games on Linux, written in Rust. It registers as an explicit Vulkan layer (`VK_LAYER_BONES_overlay`), intercepts `vkQueuePresentKHR`, runs a single ubershader containing 125 effects, and presents the modified frame. It is built around an explicit performance stance: one shader, one pass, no ping-pong, with optional GPU-extension fast paths layered on top.

### At a glance

- **125 effects** across 20 categories: geometry, AA, sharpening, 22 temporal modes, 9 console GPU simulations (PS1 through Xbox 360), CRT / OLED / VHS, colour grading, colourblind correction, more.
- **One shader, one dispatch.** Effects are `#ifdef`-toggled in a single ubershader. VRAM is O(1) three textures total, regardless of how many effects you stack.
- **Compute path by default**, fragment fallback. Compute skips the rasterizer for tighter scheduling and better extension composition; fragment kicks in automatically when the device doesnt support the storage-image features the compute path needs.
- **Vulkan extension fast paths**: `VK_KHR_shader_float16_int8`, `VK_KHR_dynamic_rendering`, `VK_KHR_push_descriptor`, `VK_KHR_synchronization2`, Vulkan 1.1 subgroup ops, `VK_KHR_swapchain_mutable_format`. Each is queried at device creation; missing pieces log a single line and the layer keeps running.
- **Async compute**: when the compute path is active and rendering at native 1:1 resolution (`resolution_scale = 1.0`), Bones automatically discovers and utilizes a dedicated async compute queue if the device exposes one not used by the game, allowing post-fx to run concurrently with the game next-frame rendering.
- **Resolution scale knob**: render the whole pipeline at any fraction of the swapchain (≥0.05) and upscale at the final blit. Big win on weak GPUs and heavy stacks.
- **Skipped input copy**: the swap image is sampled directly via per-image views no spare full-frame copy per frame.
- **Lazy postfx allocation**: when no effects are enabled, nothing is built. Toggle one on at runtime via hot reload and it spins up on the next present.
- **Env-mode**: define an entire effect stack and every general setting in environment variables. Used together, env vars completely bypass the config file (no read, no write, no watch) perfect for reproducible launches, containers, and Steam Decks.
- **Hot reload** (file mode only): edit the config while the game runs; the shader recompiles live.
- **Dual-arch**: x86_64 + i686 builds in one install, so 32-bit games on Steam / Wine / Proton get the layer too.
- **Flatpak support**, first-class, via a runtime extension.

## Quick Start

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

## Table of Contents

- [Minimum Requirements](#minimum-requirements)
- [How It Works](#how-it-works)
- [Performance Architecture](#performance-architecture)
- [Installation](#installation)
- [Flatpak](#flatpak)
- [Usage](#usage)
- [Environment Variables](#environment-variables)
- [Effect Catalogue](#effect-catalogue)
- [Recommended Combinations](#recommended-combinations)
- [Performance & Limitations](#performance--limitations)
- [Credits & License](#credits--license)

## Minimum Requirements

| Path | Requirement |
|------|-------------|
| **Vulkan** | Vulkan 1.0+ with `VK_KHR_swapchain` |
| **System** | Linux x86_64 (and optionally i686 for 32-bit games); `glibc 2.17`+ for native builds, `glibc 2.36` for portable release builds |

Bones is Vulkan-only. OpenGL, OpenGL ES, and other graphics APIs are out of scope: every modern Linux game ships a Vulkan path (native, DXVK, VKD3D-Proton, MoltenVK on translation layers), and focusing on Vulkan lets the layer concentrate every optimization on a single, well-defined pipeline.

The compute path additionally requires the device feature `shaderStorageImageWriteWithoutFormat` and a swap format that supports `VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT`. Missing either of these silently downgrades the swapchain to the fragment-shader path.

## How It Works

Bones registers as an explicit Vulkan layer (`VK_LAYER_BONES_overlay`), enabled via `VK_INSTANCE_LAYERS` (set automatically by the launcher). At `vkQueuePresentKHR`, the layer:

1. Samples the swap image directly through a per-image `ImageView` (no input copy).
2. Runs the **ubershader** (compute by default, fragment when unsupported) into an offscreen `tex_output` at `fx_extent = swap_extent × resolution_scale`.
3. Optionally samples a history texture for temporal effects.
4. Copies `tex_output` to `tex_history` for the next frame, then blits it back to the swap image (linear filter when `resolution_scale < 1.0`, plain copy when 1.0).

The pipeline is built lazily when no effects are enabled, nothing is allocated. Enabling one via hot reload triggers the build on the next present.

Effects run in a **fixed order** across three semantic stages: *render the ideal image* (geometric warps, denoise, AA, sharpening, blur, image quality, temporal, exposure, tonemap, grading, accessibility), *apply lens / film effects* (inline grain, vignette, dither), then *show it on a chosen monitor* (hardware sims act as the display device). Overlays render last.

> [!NOTE]
> Hardware simulations are **terminal display effects**. AA, sharpen, and TAA run *before* the sim, so they operate on the ideal frame; the sim then interprets that frame as if it were going through PS1 / CRT / VHS hardware.

## Performance Architecture

Bones differentiates itself from ReShade-style postfx pipelines through a series of deliberate architectural choices. None of them are individually exotic; whats unusual is composing all of them in one tool from day one.

### Ubershader instead of pass chaining

Traditional postfx tools chain N passes for N effects, ping-ponging between FBOs. Bones compiles every enabled effect into one large shader gated by `#ifdef ENABLE_<effect>`. The cost: longer first-run shader compile time. The win: when you stack 5+ effects, the bandwidth saved by not reading and writing a fullscreen render target between each effect is significant on bandwidth-bound GPUs (Steam Deck, mobile dGPUs, older discrete cards). VRAM stays at three textures regardless of effect count.

### Compute by default, fragment fallback

The compute shader path skips the rasterizer entirely no vertex shader, no triangle setup, no ROP path and gives explicit control over workgroup size for shared-memory tap reuse and subgroup ops. On every modern desktop GPU and on Steam Deck the compute path wins consistently when more than a handful of effects are active. Workgroup size defaults to 8×8 and is tunable per axis (`compute_x`, `compute_y`) for hardware where 16×16 or 32×4 wins.

If the device doesnt support `shaderStorageImageWriteWithoutFormat`, or the swap format lacks `VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT`, the layer logs the reason and silently builds the fragment-shader pipeline for that swapchain instead. The fragment path is always present and always working.

### Optional Vulkan extension fast paths

Each is opt-in via TOML (`optimize_*` keys) or env var (`BONES_OPTIMIZE_*`), defaults on, checked at device creation:

| Extension / Feature | What it enables | If missing |
|---|---|---|
| `VK_KHR_shader_float16_int8` | fp16 math in the shader | logs warning, runs fp32 |
| `VK_KHR_dynamic_rendering` | skip render passes / framebuffers | logs warning, uses render passes |
| `VK_KHR_push_descriptor` | inline descriptor writes per draw | logs warning, uses descriptor sets |
| `VK_KHR_synchronization2` | efficient batched CPU pipeline barriers | logs warning, uses legacy barriers |
| Vulkan 1.1 subgroup ops | shared-memory reductions in the shader | logs warning, runs scalar |
| `VK_KHR_shader_subgroup_extended_types` | subgroup ops on 8/16/64-bit and bool types | logs warning, runs without them |
| `VK_KHR_shader_subgroup_uniform_control_flow` | tighter driver optimization for subgroup ops | logs warning, runs without it |
| `VK_KHR_swapchain_mutable_format` | sample sRGB swap images as UNORM (skip input copy) | logs warning, samples native format |
| Dedicated async compute queue | concurrent post-fx dispatch alongside graphics (at 1.0 scale) | logs warning or disabled by scaling, falls back to present queue |

The log line for a missing extension always looks like:

```
[bones] optimization not applied: <feature> requires <extension>
```

So you know exactly what the device is missing.

### Resolution scale

A single multiplier (`resolution_scale`, default 1.0, minimum 0.05) controls the size of the entire postfx render target relative to the swapchain. Setting it to 0.5 renders all 125 effects at quarter resolution and bilinear-upscales to native at the final blit. On expensive effect stacks this routinely doubles framerate on weaker GPUs at a barely-perceptible cost in sharpness. The history texture follows the same scale, so temporal effects stay aligned.

Because hardware blitting strictly requires a graphics-capable queue, setting `resolution_scale < 1.0` automatically routes submission to the presentation queue, bypassing async compute. If the physical device or surface format fundamentally lacks blitting capabilities (`BLIT_DST` / `BLIT_SRC`), the layer safely falls back to an unscaled 1:1 pixel copy.

### Skip input copy

When the device supports `VK_KHR_swapchain_mutable_format`, sRGB swap formats are viewed as their UNORM equivalents directly, so the shader samples the swap image in linear space with no intermediate copy. When the extension isnt available, the swap image is still sampled directly via a same-format view no input copy regardless. This saves one full-frame copy per present compared to the obvious implementation.

### Lazy postfx allocation

Effects → off means resources → not built. The layer registers with the swapchain but allocates nothing until the first present where at least one effect is enabled. Toggling effects on via hot reload triggers the lazy build transparently. This makes the layer essentially free to install a user can leave it active on every game and pay no cost on games where they dont use it.

### Env-mode (full file bypass)

Setting any of the following bypasses the config file completely no read, no write, no inotify watch:

`BONES_CONFIG`, `BONES_RESOLUTION_SCALE`, `BONES_OPTIMIZE_FP16`, `BONES_OPTIMIZE_DYNAMIC_RENDERING`, `BONES_OPTIMIZE_PUSH_DESCRIPTORS`, `BONES_OPTIMIZE_SUBGROUP_OPS`, `BONES_OPTIMIZE_SYNC2`, `BONES_OPTIMIZE_SUBGROUP_EXTENDED_TYPES`, `BONES_OPTIMIZE_SUBGROUP_UNIFORM_FLOW`, `BONES_OPTIMIZE_ASYNC_COMPUTE`, `BONES_COMPUTE`, `BONES_COMPUTE_X`, `BONES_COMPUTE_Y`

This is the reproducibility story: paste one launch command into a Steam launch option, get one exact behavior, every time. `BONES_LOG` and `BONES_CONFIG_NAME` do not trigger bypass; they are meta-config (log verbosity and which profile to load), not pipeline config.

## Installation

Each Make target does one thing. Nothing builds implicitly except the build targets themselves.

| Command | What it does |
|---------|--------------|
| `make` | Native cargo build → `target/x86_64-unknown-linux-gnu/release/{bones,libbones.so}` |
| `make 32` | 32-bit (i686) build → `target/i686-unknown-linux-gnu/release/libbones.so` |
| `make release` | Same artifacts (64-bit + 32-bit), built in a Debian Bookworm container (glibc 2.36) |
| `make flatpak` | Build `.flatpak` bundles for runtimes 23.08 / 24.08 / 25.08 (errors if not built) |
| `sudo make install` | Install native x64 and x32 binaries that exist in `target/`. **Never builds.** No-op if nothing is there. |
| `sudo make flatpak-install` | Install built `.flatpak` extensions for the invoking user. |
| `sudo make uninstall` | Remove the launcher, library, manifest, system + user flatpak extension, and `~/.config/bones` |
| `make clean` | `cargo clean` + remove `flatpak/`, container stamps |
| `make flatpak-clean` | Remove flatpak bundles and workdir only |

### Pre-built binaries (if available)

Download the latest release from the [Releases](../../releases) page, extract, and from inside the extracted directory:

```sh
sudo make install
```

### Building from source

```sh
git clone https://github.com/pythonlover02/bones.git
cd bones
make                 # native 64-bit build
make 32              # optional: 32-bit build for 32-bit games
make release         # portable 64-bit + 32-bit build in container (needs podman/docker)
make flatpak         # optional: build flatpak extensions
sudo make install    # install native x64 and x32 binaries
sudo make flatpak-install # optional: install flatpak extensions
```

The 32-bit build requires `rustup target add i686-unknown-linux-gnu`, `gcc-multilib`, `g++-multilib`, `cmake`, `python3`, and `git`. `make 32` checks for these and prints what is missing. `make release` runs both 64-bit and 32-bit container builds for portable glibc-2.36 artifacts.

### Install paths

| File | Default path |
|------|--------------|
| Launcher | `/usr/bin/bones` |
| Library (64-bit) | `/usr/lib/x86_64-linux-gnu/libbones.so` or `/usr/lib64/libbones.so` or `/usr/lib/libbones.so` |
| Library (32-bit) | `/usr/lib/i386-linux-gnu/libbones.so` or `/usr/lib/libbones.so` or `/usr/lib32/libbones.so` |
| Vulkan layer manifest | `/usr/share/vulkan/implicit_layer.d/VkLayer_bones.json` |
| Docs & Licenses | `/usr/share/doc/bones/` |

Because the manifest is installed to the system-wide Vulkan directory and libraries are installed to standard distro-specific library paths, 32-bit games discover the 32-bit layer and 64-bit games discover the 64-bit layer automatically without custom `VK_ADD_LAYER_PATH` mapping required.

```sh
sudo make install PREFIX=/opt/bones    # change prefix
sudo make install DESTDIR=./package    # stage into a directory (packaging)
```

To install flatpak bundles built with `make flatpak`, run `sudo make flatpak-install`. This installs them per-user for the invoking user.

### Uninstalling

```sh
sudo make uninstall
```

Removes launcher, library (both architectures), manifest, system-scope flatpak extension, user-scope flatpak extension for the invoking user, and `~/.config/bones`. If run directly as root without `sudo` (no `SUDO_USER`), it skips the user-scope steps and prints the commands to run as your user.

### Cleaning build artifacts

```sh
make clean          # cargo clean + remove flatpak/, container stamps
make flatpak-clean  # remove only flatpak bundles + workdir
```

## Flatpak

Flatpak applications run sandboxed and cannot see `LD_PRELOAD`, `VK_ADD_LAYER_PATH`, or `VK_INSTANCE_LAYERS` set on the host those variables do not cross the sandbox boundary. Bones supports Flatpak through a **Flatpak extension**: a bundle that mounts the library, the layer manifest, and a small wrapper script (`bones-flatpak`) inside the `org.freedesktop.Platform` runtime, so they are reachable from within the sandbox. The wrapper sets the activation environment **from inside** the sandbox, the only place it survives.

Extensions are built for runtime versions `23.08`, `24.08`, and `25.08`.

### Building the extensions

Requires `flatpak`, `ostree`, `python3`, and a completed `make` (or `make release`) build. `make flatpak` errors out at parse time if `target/release/libbones.so` doesnt exist.

```sh
make            # or: make release
make flatpak
```

This produces one bundle per supported runtime:

```
org.freedesktop.Platform.VulkanLayer.bones-23.08.flatpak
org.freedesktop.Platform.VulkanLayer.bones-24.08.flatpak
org.freedesktop.Platform.VulkanLayer.bones-25.08.flatpak
```

If you also ran `make 32` before `make flatpak`, the bundles include the 32-bit library too, so Flatpak games that run 32-bit Vulkan binaries (uncommon but real for some Wine setups inside Flatpak) get the layer.

### Installing the extensions

Install the bundle matching your `org.freedesktop.Platform` runtime. Run `flatpak list` and look for `org.freedesktop.Platform` if unsure.

```sh
flatpak install --user org.freedesktop.Platform.VulkanLayer.bones-24.08.flatpak
```

Multiple runtime versions can coexist. If you ran `sudo make flatpak-install` with the extensions already built, theyre installed automatically for the invoking user.

```sh
flatpak uninstall --user org.freedesktop.Platform.VulkanLayer.bones
```

### Running a Flatpak application through Bones

```sh
bones -- flatpak run com.example.Game
bones -- flatpak run --branch=stable com.example.Game   # flags pass through
bones myprofile -- flatpak run com.example.Game          # named profile
```

Bones detects the `flatpak run` invocation, reads the application metadata to find its entry point, and rewrites the command to run `bones-flatpak` inside the sandbox, which sets the environment and execs the app.

### Steam launch option (Flatpak)

Set the game **Launch Options** to the in-sandbox wrapper absolute path:

```
/usr/lib/extensions/vulkan/bones/bin/bones-flatpak %command%
```

With a named profile:

```
BONES_CONFIG_NAME=myprofile /usr/lib/extensions/vulkan/bones/bin/bones-flatpak %command%
```

This requires the Bones Flatpak extension matching the game runtime to be installed.

## Usage

### Profiles

A profile is a named config at `~/.config/bones/<name>-config.toml`. The default profile is `bones` (`bones-config.toml`). Pass a name to load a different one:

```sh
bones retro -- ~/games/retro-game   # loads ~/.config/bones/retro-config.toml
```

### Configuration file

The default config is generated at `~/.config/bones/bones-config.toml` on first run, with every effect listed and documented. Set any effect to `true` under its category section (e.g. `mirror_horizontal = true` under `[geometric]`), and `hot_reload = true` under `[general]` for live reloading. Effects are **toggle-only**, all parameters are baked into the shader.

The `[general]` section also controls the architectural settings: `resolution_scale`, `optimize_fp16`, `optimize_dynamic_rendering`, `optimize_push_descriptors`, `optimize_subgroup_ops`, `compute`, `compute_x`, `compute_y`. Every one of these has documentation in the generated config and a corresponding environment variable (see [Environment Variables](#environment-variables)).

### Env-mode (full file bypass)

Setting any of the BONES_* general environment variables takes over from the file entirely no read, no write, no inotify watch. Hot reload is implicitly off in env-mode (nothing to watch). Use this for reproducible launches:

```sh
BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen;vibrance_boost" \
BONES_RESOLUTION_SCALE=0.75 \
BONES_COMPUTE=true \
bones -- ~/games/game
```

- Setting `BONES_CONFIG`, even to an empty string, takes over from the file. An empty value means "no effects" (a forced clean pass).
- Unknown effect names are ignored with a warning.
- Works with Flatpak too: pass any of them on the `bones -- flatpak run …` line and they are forwarded into the sandbox.

### Hot reload (file mode only)

With `hot_reload = true` and no `BONES_*` env vars set, Bones uses `inotify` to watch the config directory. Save changes and the shader recompiles and reloads without restarting the game. If a reload fails to compile, the previous working shader is kept.

## Environment Variables

### Triggers env-mode (any of these bypasses the file completely)

| Variable | Purpose | Values | Default |
|----------|---------|--------|---------|
| `BONES_CONFIG` | Inline effect list | semicolon-separated effect names | *(unset → use file)* |
| `BONES_RESOLUTION_SCALE` | Postfx render scale | float ≥ 0.05 | `1.0` |
| `BONES_OPTIMIZE_FP16` | Enable fp16 path if device supports it | `1`/`true`, `0`/`false` | `true` |
| `BONES_OPTIMIZE_DYNAMIC_RENDERING` | Enable `VK_KHR_dynamic_rendering` | `1`/`true`, `0`/`false` | `true` |
| `BONES_OPTIMIZE_PUSH_DESCRIPTORS` | Enable `VK_KHR_push_descriptor` | `1`/`true`, `0`/`false` | `true` |
| `BONES_OPTIMIZE_SUBGROUP_OPS` | Enable Vulkan 1.1 subgroup ops | `1`/`true`, `0`/`false` | `true` |
| `BONES_OPTIMIZE_SYNC2` | Enable `VK_KHR_synchronization2` | `1`/`true`, `0`/`false` | `true` |
| `BONES_OPTIMIZE_SUBGROUP_EXTENDED_TYPES` | Enable extended subgroup types | `1`/`true`, `0`/`false` | `true` |
| `BONES_OPTIMIZE_SUBGROUP_UNIFORM_FLOW` | Enable subgroup uniform control flow | `1`/`true`, `0`/`false` | `true` |
| `BONES_OPTIMIZE_ASYNC_COMPUTE` | Submit to dedicated async compute queue if available | `1`/`true`, `0`/`false` | `true` |
| `BONES_COMPUTE` | Use compute shader path | `1`/`true`, `0`/`false` | `true` |
| `BONES_COMPUTE_X` | Compute workgroup X | positive integer | `8` |
| `BONES_COMPUTE_Y` | Compute workgroup Y | positive integer | `8` |

### Does not trigger env-mode

| Variable | Purpose | Values | Default |
|----------|---------|--------|---------|
| `BONES_CONFIG_NAME` | Which profile to load (in file mode) | any profile name | `bones` |
| `BONES_LOG` | Log verbosity (written to stderr) | `off`, `error`, `warn`, `info` | `warn` |

Empty or unset means "use default". `LD_PRELOAD`, `VK_ADD_LAYER_PATH`, and `VK_INSTANCE_LAYERS` are set automatically by the launcher when used manually.

## Effect Catalogue

125 effects in 21 categories, applied in the fixed order below. The selection column is the rule of thumb; see the note after the table.

| Order | Category | Section | Count | What it does | Selection |
|------:|----------|---------|------:|--------------|-----------|
| 1 | Geometric | `[geometric]` | 12 | UV / coordinate warps | mostly combinable |
| 2 | Denoise | `[denoise]` | 1 | clean noise before processing | single |
| 3 | Anti-aliasing | `[anti_aliasing]` | 4 | smooth jagged edges | **pick one** |
| 4 | Sharpening | `[sharpening]` | 9 | enhance detail | combinable\* |
| 5 | Local contrast | `[local_contrast]` | 1 | per-pixel "pop" | single |
| 6 | Blur | `[blur]` | 5 | creative / soften | pick one |
| 7 | Image quality | `[image_quality]` | 3 | deband, bloom, lens flare | combinable |
| 8 | Temporal | `[temporal]` | 22 | frame blending / TAA | **one primary** (guards auto) |
| 9 | Exposure | `[exposure]` | 1 | pre-tonemap exposure | single |
| 10 | Tonemapping | `[tonemapping]` | 8 | HDR → SDR curves | **pick one** |
| 11 | White balance | `[white_balance]` | 3 | colour temperature | **pick one** |
| 12 | Colour grading | `[color_grading]` | 8 | grade / saturation / curves | combinable |
| 13 | Channel curves | `[channel_curves]` | 3 | per-channel S-curves | combinable |
| 14 | Colour balance | `[color_balance]` | 1 | teal / orange tint | single |
| 15 | Selective colour | `[selective_color]` | 3 | per-channel saturation | combinable |
| 16 | Stylization | `[stylization]` | 9 | creative looks | mostly pick one |
| 17 | Accessibility | `[accessibility]` | 6 | CVD simulate / correct | pick one |
| 18 | Inline | `[inline]` | 7 | grain, vignette, letterbox… | stackable |
| 19 | Hardware simulation | `[hardware_simulation]` | 17 | Console GPU sims + CRT / OLED / VHS | one console + one display |
| 20 | Overlay (HUD) | `[overlay]` | 2 | FPS, crosshair | combinable |

\* Sharpeners combine (e.g. one sharpener + `midtone_clarity`), but stacking multiple *strong* sharpeners produces halos.

> [!IMPORTANT]
> **Enable at most one effect per category unless noted otherwise.** Dont run two tonemappers or two AA methods. For temporal, pick one primary mode; a fused stabilizer (`convergent_detail_recovery`) activates automatically whenever any temporal mode is enabled. For hardware simulation, pick one console sim and optionally one display sim.

> [!NOTE]
> Combining a temporal mode with a hardware sim is supported but the history texture captures the post-sim image; TAA reads PS1 / CRT distorted history on the next frame. This produces a "weird but not broken" look. If you want clean TAA, dont stack a sim on top.

<details>
<summary><b>1. Geometric</b> UV warps (12)</summary>

| Effect | Description |
|--------|-------------|
| `identity` | No-op passthrough (baseline / debug) |
| `mirror_horizontal` | Flip left ↔ right |
| `mirror_vertical` | Flip top ↔ bottom |
| `rotate_90` | Rotate 90° clockwise |
| `rotate_180` | Rotate 180° |
| `rotate_270` | Rotate 270° clockwise |
| `center_zoom` | 1.5× magnify from screen centre |
| `polynomial_distort` | Brown–Conrady barrel lens distortion (k1 / k2) |
| `barrel_undistort` | Inverse barrel correction (straighten curves) |
| `fisheye_warp` | Wide-angle fisheye projection (atan remap) |
| `trapezoid_warp` | Perspective keystone (wider at bottom) |
| `sharp_bilinear` | Pixel-art sharpener via Hermite UV snap |
</details>

<details>
<summary><b>2. Denoise</b> (1)</summary>

| Effect | Description |
|--------|-------------|
| `bilateral_denoise` | Edge-preserving noise reduction (Tomasi & Manduchi 1998) |
</details>

<details>
<summary><b>3. Anti-aliasing</b> pick one (4)</summary>

| Effect | Description |
|--------|-------------|
| `luma_edge_aa` | FXAA (Timothy Lottes, NVIDIA) |
| `normal_filter_aa` | NFAA, lightweight gradient AA |
| `morphological_aa` | Conservative CMAA-style AA |
| `subpixel_aa` | SMAA-style subpixel morphological AA |
</details>

<details>
<summary><b>4. Sharpening</b> combinable, dont stack strong ones (9)</summary>

| Effect | Description |
|--------|-------------|
| `contrast_adaptive_sharpen` | AMD CAS (FidelityFX) |
| `robust_contrast_sharpen` | AMD RCAS (FSR 1.0), halo-free |
| `edge_directed_sharpen` | NVIDIA NIS-style edge-aware sharpen |
| `laplacian_sharpen` | Classic simple sharpen |
| `luminance_sharpen` | Luma-only, no colour fringing |
| `midtone_clarity` | Lightroom-style "Clarity" |
| `falloff_sharpen` | Adaptive, gated by local edge energy |
| `power_curve_sharpen` | Filmic power-curve response |
| `unsharp_mask` | Classic darkroom unsharp mask |
</details>

<details>
<summary><b>5. Local contrast</b> (1)</summary>

| Effect | Description |
|--------|-------------|
| `local_contrast` | Per-pixel local contrast enhancement |
</details>

<details>
<summary><b>6. Blur</b> pick one (5)</summary>

| Effect | Description |
|--------|-------------|
| `gaussian_blur` | 3×3 Gaussian |
| `box_blur` | Kawase dual-filter wide blur |
| `bokeh_blur` | Circular DoF with highlight bokeh |
| `tilt_shift_blur` | Miniature / diorama band blur |
| `radial_blur` | Zoom / speed blur from centre |
</details>

<details>
<summary><b>7. Image quality</b> combinable (3)</summary>

| Effect | Description |
|--------|-------------|
| `gradient_deband` | Remove banding in gradients (mpv-style) |
| `threshold_bloom` | Bright-pixel glow |
| `ghost_flare` | Lens-flare ghost reflections |
</details>

<details>
<summary><b>8. Temporal</b> one primary, guards automatic (22)</summary>

22 primary frame-blending modes. A fused stabilizer (`convergent_detail_recovery`) activates automatically whenever any temporal mode is enabled; it gates history influence on large per-pixel changes (acting as a lightweight disocclusion guard) and pulls the current frame toward the converged history to recover detail. Ordered roughly simplest to most sophisticated.

| Effect | Description |
|--------|-------------|
| `neighborhood_clamp_aa` | TAA, neighbourhood min / max clamp |
| `motion_reject_denoise` | Motion-gated accumulation |
| `motion_detect_blur` | Temporal motion blur |
| `constant_blend_smooth` | 50/50 blend (baseline) |
| `shutter_angle_smooth` | 180° camera shutter simulation |
| `spline_interp_smooth` | Cubic Hermite temporal reconstruction |
| `variance_decay_smooth` | Variance-gated IIR filter |
| `dualrate_smooth` | Dual-rate adaptive blend |
| `luminance_gate_smooth` | Smooth dark areas more (scotopic vision) |
| `contrast_gate_smooth` | Smooth low-contrast areas more |
| `gradient_gate_smooth` | Edge-aware, strong single anti-ghosting technique |
| `sigma_clip_smooth` | Variance clipping (modern TAA core, Salvi 2016) |
| `mitchell_kernel_smooth` | Mitchell–Netravali kernel + temporal |
| `ycocg_clip_smooth` | YCoCg AABB clamp, the technique at the core of FSR 2 |
| `bilateral_history_smooth` | Bilateral filter applied in time |
| `perceptual_chroma_smooth` | YCbCr, aggressive chroma smoothing |
| `frequency_split_smooth` | Low / high frequency band split blend |
| `horn_schunck_smooth` | Optical-flow warp (Horn & Schunck 1981) |
| `convergent_accumulate` | Detail accumulation, inspired by the principle behind DLSS 2 |
| `dualwarp_flow_smooth` | Dual-warp interpolation, inspired by FSR 3 frame generation |
| `variance_flow_accumulate` | Motion-compensated, triple-gated, inspired by the research behind DLSS 2 |
| `edge_reconstruct_smooth` | Edge-directed reconstruction, inspired by XeSS DP4a path |
</details>

<details>
<summary><b>9. Exposure</b> (1)</summary>

| Effect | Description |
|--------|-------------|
| `linear_exposure` | Pre-tonemap exposure multiplier |
</details>

<details>
<summary><b>10. Tonemapping</b> pick one (8)</summary>

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
<summary><b>11. White balance</b> pick one (3)</summary>

| Effect | Description |
|--------|-------------|
| `neutral_white_balance` | D65 reference correction |
| `warm_temperature` | Warm shift (tungsten) |
| `cool_temperature` | Cool shift (daylight) |
</details>

<details>
<summary><b>12. Colour grading</b> combinable (8)</summary>

| Effect | Description |
|--------|-------------|
| `saturation_contrast_grade` | Combined auto-enhance pass |
| `levels_remap` | Black / white point remap |
| `gamma_correct` | Gamma power curve |
| `vibrance_boost` | Smart saturation (Lightroom Vibrance) |
| `hsl_transform` | Hue / saturation / lightness in HSL |
| `split_tone` | Orange-and-teal split toning |
| `lift_gamma_gain` | Three-way colour corrector |
| `hermite_curves` | Smooth S-curve contrast |
</details>

<details>
<summary><b>13. Channel curves</b> combinable (3)</summary>

| Effect | Description |
|--------|-------------|
| `red_channel_curve` | S-curve on red |
| `green_channel_curve` | S-curve on green |
| `blue_channel_curve` | S-curve on blue |
</details>

<details>
<summary><b>14. Colour balance</b> (1)</summary>

| Effect | Description |
|--------|-------------|
| `trizone_color_balance` | Three-zone teal / orange tint |
</details>

<details>
<summary><b>15. Selective colour</b> combinable (3)</summary>

| Effect | Description |
|--------|-------------|
| `red_selective_saturate` | Boost saturation of red-dominant pixels |
| `green_selective_saturate` | Boost saturation of green-dominant pixels |
| `blue_selective_saturate` | Boost saturation of blue-dominant pixels |
</details>

<details>
<summary><b>16. Stylization</b> mostly pick one (9)</summary>

| Effect | Description |
|--------|-------------|
| `dynamic_range_crush` | Flat "Instagram" range crush |
| `duotone_map` | Two-colour luminance map |
| `color_wash_tint` | Subtle palette-unifying tint |
| `posterize_quantize` | 8-level posterize |
| `bleach_bypass` | Desaturated high-contrast film look |
| `technicolor_process` | Three-strip Technicolor |
| `midpoint_contrast` | Simple contrast boost |
| `color_invert` | Negative image |
| `luminance_grayscale` | BT.709 grayscale |
</details>

<details>
<summary><b>17. Accessibility</b> colour vision (6)</summary>

| Effect | Description |
|--------|-------------|
| `protanopia_simulation` | Simulate red-blind vision |
| `deuteranopia_simulation` | Simulate green-blind vision |
| `tritanopia_simulation` | Simulate blue-blind vision |
| `protanopia_correct` | Daltonize for protanopia |
| `deuteranopia_correct` | Daltonize for deuteranopia |
| `tritanopia_correct` | Daltonize for tritanopia |
</details>

<details>
<summary><b>18. Inline</b> stackable (7)</summary>

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
<summary><b>19. Hardware simulation</b> one console + one display (17)</summary>

Hardware sims act as the **display device** showing the finished image. AA and sharpen run before the sim so they see the ideal frame.

**Console GPU simulation** (pick one):

| Effect | Description |
|--------|-------------|
| `ps1_simulation` | PS1, UV grid quantization, nearest-neighbour feel, 15-bit colour with Bayer dither |
| `saturn_simulation` | Saturn, luminance banding, dark desaturated palette, warm brown shift |
| `n64_simulation` | N64, low-res quantization softness, radial fog, warm shift |
| `dreamcast_simulation` | Dreamcast, over-bright response, boosted saturation, specular highlights |
| `ps2_simulation` | PS2, pixel quantization, soft scanline modulation, threshold bloom |
| `xbox_simulation` | Xbox, highlight boost, midtone lift bloom, plastic specular, warm / green bias |
| `psp_simulation` | PSP, 480×272 phase, LCD washout gamma, dark banding, slight desaturation |
| `ps3_simulation` | PS3, sub-HD quantization, crushed shadows, cool shift |
| `xbox360_simulation` | Xbox 360, eDRAM tile seams, HDR banding, lifted warm gamma, desat with specular boost |

**Display simulation** (pick one, combinable with console sims):

| Effect | Description |
|--------|-------------|
| `crt_simulation` | CRT, barrel warp + RGB phosphor mask + scanlines + brightness pulsing |
| `phosphor_amber` | Amber monochrome CRT |
| `phosphor_green` | Green monochrome CRT (classic terminal) |
| `phosphor_red` | Red monochrome CRT |
| `scanline_darken` | Scanlines only (no warp / mask) |
| `oled_simulation` | Black crush + saturation boost |
| `vhs_simulation` | VHS, horizontal ripple, per-line tracking noise, luma noise, desaturation, warm shift, dropout |
| `lcd_subpixel` | Visible RGB subpixel grid |
</details>

<details>
<summary><b>20. Overlay (HUD)</b> combinable (2)</summary>

Overlays render after all processing (temporal, colour grading, accessibility, hardware sims) so they are never affected by other effects.

| Effect | Description |
|--------|-------------|
| `fps_hud` | On-screen FPS counter (7-segment) |
| `crosshair_overlay` | Centred neon crosshair |
</details>

## Recommended Combinations

Copy-paste these as `BONES_CONFIG` values, or set the same keys in your config file. Each respects the one-per-category rule.

```sh
# Crisp modern look: AA + sharpen + smart saturation
BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen;vibrance_boost" bones -- ~/games/game

# Cinematic: filmic tonemap, split tone, layered inline effects
BONES_CONFIG="hable_tonemap;split_tone;radial_vignette;cinematic_letterbox;gaussian_grain" bones -- ~/games/game

# PS1 on a CRT
BONES_CONFIG="ps1_simulation;crt_simulation" bones -- ~/games/game

# N64 on a CRT
BONES_CONFIG="n64_simulation;crt_simulation" bones -- ~/games/game

# PSP handheld look
BONES_CONFIG="psp_simulation;lcd_subpixel" bones -- ~/games/game

# Anti-flicker temporal stability (stabilizer activates automatically)
BONES_CONFIG="gradient_gate_smooth" bones -- ~/games/game

# VHS found footage
BONES_CONFIG="vhs_simulation" bones -- ~/games/game

# Colourblind correction (deuteranopia)
BONES_CONFIG="deuteranopia_correct" bones -- ~/games/game

# Half-res render for weak GPUs (combine with any stack above)
BONES_RESOLUTION_SCALE=0.5 BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen" bones -- ~/games/game
```

The cinematic example deliberately stacks several `[inline]` effects grain, vignette, and letterbox operate independently and are designed to layer.

## Performance & Limitations

**Strengths:**
- **VRAM**: O(1), only three textures (output, history, swap-image view) regardless of effect count.
- **Dispatch / draw calls**: always one `vkCmdDispatch` on the compute path, one full-screen triangle on fragment.
- **CPU overhead**: minimal descriptor sets pre-built, command buffers per swap image.
- **No input copy**: shader samples the swap image directly via per-image views.
- **Lazy build**: zero allocation when no effects are enabled.

**Limitations:**
- Vulkan only. OpenGL and OpenGL ES are out of scope by design.
- No custom shaders or LUTs every effect is built into the ubershader at compile time.
- No multi-pass effects (e.g. advanced bloom with downsample chains). The single-pass discipline rules them out.
- Per-object motion vectors (as in true DLSS) are approximated by optical flow from neighbouring pixels useful but not pixel-perfect on fast motion.
- Fast motion can cause ghosting on some temporal modes; the automatic `convergent_detail_recovery` stabilizer includes a motion gate that softens history influence on large per-pixel changes to reduce it.
- Combining a temporal mode with a hardware sim makes the history texture capture the post-sim image, which the next frame reads back through TAA; the result is stable but visually unusual.
- Compute path requires `shaderStorageImageWriteWithoutFormat` and a swap format with `VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT`. Missing either silently falls back to fragment with a log line.

The effect order is fixed and cannot be changed at runtime.

## Credits & License

Heavily inspired by existing postfx tools, but the ubershader implementation and the majority of the effect code were written from scratch (or ported from GLSL snippets in public-domain and open-source shader repositories).

All 125 effect implementations are original or derived from well-known algorithms (FXAA by Timothy Lottes, CAS / RCAS from AMD GPUOpen, AgX, ACES, and others) under their respective licenses (mostly MIT / BSD). Full attribution and external license texts are in [dist.LICENSE](dist.LICENSE).

The project itself is released under the **GNU General Public License v3.0**, full text in [LICENSE](LICENSE).
