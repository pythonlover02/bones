> [!NOTE]
> Bug reports and pull requests are welcome, but please understand that development happens in my free time and progress may be slow at times, the project is still maintained even if the last commit was made a while ago.

# Bones – GLSL ubershader post‑processing tool (OpenGL/Vulkan)

Bones is a realtime post‑processing overlay for Linux games, written in Rust. It intercepts OpenGL and Vulkan swap buffers, applies a single pass ubershader with a wide range of effects, and then presents the modified frame. The tool is designed for **performance** and **simplicity**; no custom LUTs, no external shader loading, no custom values, and no ping pong. All effects are combined into one shader pass, making memory usage O(1) regardless of how many effects are enabled.

## Table of Contents

- [Minimum Requirements](#minimum-requirements)
- [How It Works](#how-it-works)
- [Features](#features)
- [Installation](#installation)
- [Flatpak](#flatpak)
- [Usage](#usage)
  - [Profiles](#profiles)
  - [Configuration](#configuration)
  - [Hot Reload](#hot-reload)
- [Effect Categories](#effect-categories)
- [Performance & Limitations](#performance--limitations)
- [Building from Source](#building-from-source)
- [Credits & License](#credits--license)

## Minimum Requirements

### OpenGL path
- **OpenGL 3.0** or newer (released 2008; supported by essentially all discrete and integrated GPUs made in the last 15 years)
- Mesa 7.9+ or any proprietary driver with GL 3.0 support
- GLX or EGL windowing

Contexts that report GL 3.0 or higher automatically satisfy all requirements (core FBO, VAO, GLSL 1.30). Pure GL 2.1 contexts without `GL_ARB_vertex_array_object` will have post‑FX silently disabled for that context the game runs normally, just without the overlay.

### Vulkan path
- **Vulkan 1.0** or newer
- A driver that supports `VK_KHR_swapchain`

### OpenGL ES path (EGL)
- **OpenGL ES 2.0** or newer

### General
- Linux x86_64 (32‑bit builds available but require additional tooling see [Building from Source](#building-from-source))
- `glibc 2.17` or newer for native builds; `glibc 2.36` for portable release builds (built inside a Debian Bookworm container)

## How It Works

Bones has two interception paths that work simultaneously:

- **OpenGL** – uses `LD_PRELOAD` to hook `glXSwapBuffers` (GLX) and `eglSwapBuffers` (EGL). Before the real swap, Bones copies the default framebuffer into a texture, draws a full‑screen triangle with the ubershader, and writes the result back.
- **Vulkan** – registers as an implicit Vulkan layer (`VK_LAYER_BONES_overlay`). At `vkQueuePresentKHR`, the layer copies the swapchain image into an offscreen texture, renders the ubershader into a second offscreen image, and copies the result back to the swapchain image before presentation.

In both paths the pipeline is the same:

1. Copy the current frame into an input texture.
2. Run a single **ubershader** that combines all enabled effects.
3. Optionally read a history texture for temporal effects.
4. Write the result back and copy it into the history texture for the next frame.

Unlike traditional reshade pipelines that chain multiple shader passes (ping‑pong between FBOs), Bones **never uses more than one draw call**. All effects are compiled into one large shader with `#ifdef` guards. This keeps VRAM usage constant and avoids the overhead of multiple passes.

Because effects are executed in a fixed order (UV warp → spatial → temporal → inline → colour grading), some combinations may produce unexpected results if you enable two effects from the same category (e.g., two different tonemappers). **For best results, enable only one effect per category, unless they are intended to be used together.**

## Features

- **118 built‑in effects** – from geometric warps (mirror, rotate, lens distortion) to anti‑aliasing (FXAA, SMAA‑style), sharpening (CAS, RCAS), temporal smoothing (TAA, optical flow, variance accumulation), bloom, film grain, CRT/OLED/VHS simulation, colour grading, and even colour blindness simulation/correction.
- **Ubershader architecture** – all effects are compiled into a single shader. Enabling an effect simply toggles a `#define`.
- **Hot reload** – edit your configuration TOML file while the game is running and the shader recompiles automatically (if `hot_reload = true`).
- **Temporal effects** – many temporal smoothing modes, including motion‑compensated accumulation inspired by DLSS/FSR/XeSS research.
- **Performance‑first** – no custom LUT loading, no external shader scripts, no ping pong rendering. Everything is O(1) in VRAM and draw calls.
- **Cross‑API** – works with OpenGL (GLX/EGL) and Vulkan applications.

## Installation

### Pre‑built binaries (if available)

Download the latest release from the [Releases](../../releases) page. Extract it, open a terminal inside the extracted directory, and run:

```bash
sudo make install
```

This installs the `bones` launcher, `libbones.so`, and any Flatpak extensions that were included in the release bundle.

### Building from source

```bash
git clone https://github.com/pythonlover02/bones.git
cd bones
```

#### 64‑bit (default)

```bash
make
```

Produces `target/release/bones` and `target/release/libbones.so`.

#### 32‑bit (optional)

Requires `rustup`, `cmake`, `python` or `python3`, and `git`.

```bash
make 32
```

Produces `target/i686-unknown-linux-gnu/release/libbones.so`. The `i686-unknown-linux-gnu` Rust target is added automatically via `rustup` if not already installed.

#### Portable release build

Builds the 64‑bit library inside a Debian Bookworm container (for maximum glibc compatibility), then builds the 32‑bit library and all Flatpak extensions. Requires `podman` or `docker`, `flatpak`, `ostree`, and `python3`.

```bash
make release
```

### Installing

Run as root after building:

```bash
sudo make install
```

Default install locations:

| File | Path |
|------|------|
| Launcher | `/usr/local/bin/bones` |
| 64‑bit library | `/usr/local/lib/libbones.so` |
| 32‑bit library | `/usr/local/lib32/libbones.so` (if built) |

To change the prefix:

```bash
sudo make install PREFIX=/opt/bones
```

To stage into a directory (e.g. for packaging):

```bash
sudo make install DESTDIR=./package
```

`ldconfig` is run automatically after install. If Flatpak extensions were built beforehand (via `make flatpak` or `make release`) they are installed per‑user for the invoking user as part of `sudo make install`.

### Uninstalling

```bash
sudo make remove
```

or equivalently:

```bash
sudo make uninstall
```

This removes the launcher, both libraries, runs `ldconfig`, and uninstalls the Flatpak extension for the invoking user.

### Cleaning build artifacts

```bash
make clean
```

Runs `cargo clean` and removes the `flatpak/` output directory and the `.flatpak-work/` staging directory.

To clean only Flatpak artifacts without touching the Cargo build:

```bash
make flatpak-clean
```

## Flatpak

Flatpak applications run inside a sandbox and cannot see `LD_PRELOAD` or `VK_LAYER_PATH` from the host. Bones supports this through a **Flatpak extension** a bundle that installs into the `org.freedesktop.Platform` runtime and is visible to all Flatpak apps automatically.

Extensions are built for the following runtime versions: `23.08`, `24.08`, `25.08`.

### Build requirements

- `flatpak`
- `ostree`
- `python3`
- A completed `make` or `make release` build (the 64‑bit library must exist before running `make flatpak`)

### Building the extensions

First build the native 64‑bit library:

```bash
make
```

Optionally build the 32‑bit library too (it will be bundled inside the extension if present):

```bash
make 32
```

Then build the Flatpak extensions:

```bash
make flatpak
```

This produces one `.flatpak` bundle per supported runtime version inside the `flatpak/` directory:

```
flatpak/org.freedesktop.Platform.VulkanLayer.bones-23.08.flatpak
flatpak/org.freedesktop.Platform.VulkanLayer.bones-24.08.flatpak
flatpak/org.freedesktop.Platform.VulkanLayer.bones-25.08.flatpak
```

To build everything at once (portable 64‑bit, 32‑bit, and all Flatpak extensions) use the release target:

```bash
make release
```

### Installing the extensions

Install the bundle that matches your `org.freedesktop.Platform` runtime version. If you are unsure which version you have, run `flatpak list` and look for `org.freedesktop.Platform`.

```bash
flatpak install --user flatpak/org.freedesktop.Platform.VulkanLayer.bones-24.08.flatpak
```

Multiple runtime versions can be installed side by side without conflict. If you ran `sudo make install` and Flatpak extensions were already built, they are installed automatically for the invoking user you do not need to run the command above manually.

To uninstall:

```bash
flatpak uninstall --user org.freedesktop.Platform.VulkanLayer.bones
```

### Using Bones with a Flatpak application

#### Any Flatpak application

Use the `bones` launcher with `flatpak run`:

```bash
bones -- flatpak run com.example.Game
```

Bones detects the `flatpak run` invocation automatically, reads the application metadata to find its entry point, and rewrites the command to inject via `bones-inject` inside the sandbox. You can pass `flatpak run` flags normally:

```bash
bones -- flatpak run --branch=stable com.example.Game
```

#### Profiles with Flatpak

Profiles work the same way as with native applications. The config directory (`~/.config/bones/`) is shared into the sandbox automatically by the launcher:

```bash
bones myprofile -- flatpak run com.example.Game
```

> [!NOTE]
> The Vulkan layer is registered as an implicit layer inside the extension and activates automatically when `BONES_ACTIVE=1` is set by `bones-inject`. For OpenGL applications `bones-inject` sets `LD_PRELOAD` to the bundled library path inside the extension mount.

## Usage

### Profiles

A profile is a named configuration stored in `~/.config/bones/<name>-config.toml`. The default profile is `bones` (file `bones-config.toml`).

```bash
bones retro -- ~/games/retro-game
```

### Configuration

Example `~/.config/bones/bones-config.toml`:

```toml
[general]
hot_reload = true

[geometric]
mirror_horizontal = true

[anti_aliasing]
subpixel_aa = true

[sharpening]
contrast_adaptive_sharpen = true

[temporal]
surface_disocclusion_guard = true   # works alongside any temporal mode
convergent_accumulate = true        # primary temporal mode

[color_grading]
vibrance_boost = true
```

All effects are listed and documented in the default config file generated at `~/.config/bones/bones-config.toml` on first run. Set any effect to `true` to enable it.

### Hot Reload

When `hot_reload = true`, Bones uses `inotify` to watch the config directory. Save your changes and the shader recompiles and reloads without restarting the game.

## Effect Categories

Effects are processed in this order (see the TOML file for full lists):

1. **Geometric** – UV coordinate warps (mirror, rotate, distortion, sharp bilinear)
2. **Denoise** – Bilateral denoise
3. **Anti‑aliasing** – FXAA, NFAA, morphological, subpixel (pick **one**)
4. **Sharpening** – CAS, RCAS, edge‑directed, etc. (can combine with midtone clarity)
5. **Local contrast** – Single effect
6. **Blur** – Gaussian, box, bokeh, tilt‑shift, radial
7. **Image quality** – Deband, bloom, ghost flare
8. **Display simulation** – CRT, phosphor, OLED, VHS, LCD subpixel
9. **Overlay** – FPS HUD, crosshair
10. **Temporal** – Many modes (TAA, motion gated, shutter angle, optical flow, convergent accumulation, variance flow, edge reconstruct, etc.) – pick **one** primary mode, optionally with `surface_disocclusion_guard` and/or `convergent_detail_recovery`
11. **Inline** – Grain, chromatic aberration, halation, vignette, letterbox, dither
12. **Exposure** – Linear exposure multiplier
13. **Tonemapping** – ACES, AgX, Reinhard, Hable, etc. – pick **one**
14. **White balance** – Neutral, warm, cool
15. **Colour grading** – Saturation/contrast, levels, gamma, vibrance, HSL, split tone, lift/gamma/gain, Hermite curves
16. **Channel curves** – Per‑channel s‑curve
17. **Colour balance** – Tri‑zone (teal/orange)
18. **Selective colour** – Boost saturation per dominant channel
19. **Stylization** – Dynamic range crush, duotone, wash, posterize, bleach bypass, Technicolor, invert, grayscale
20. **Accessibility** – Colour blindness simulation & correction (protanopia, deuteranopia, tritanopia)

> [!IMPORTANT]
> **Enable at most one effect per category, unless specified otherwise.**
> For example, do not enable two different tonemappers or two different anti‑aliasing methods. Some combinations (e.g., sharpener + midtone clarity) are safe and intended.

## Performance & Limitations

- **VRAM**: O(1) – only three textures (input, output, history) regardless of how many effects are enabled.
- **Draw calls**: Always 1 full‑screen triangle.
- **CPU overhead**: Minimal; the launcher only sets up hooks and reloads the config.
- **Limitations**:
  - No custom shaders or LUTs – you cannot load external `.fx` files or PNG LUTs.
  - No multi‑pass effects (e.g., advanced bloom with downsample chains).
  - Effects that depend on per‑object motion vectors (like true DLSS) are approximated by optical flow from neighbour pixels.
  - Some temporal modes may cause ghosting if motion is too fast; use `surface_disocclusion_guard` to reduce it.
  - OpenGL contexts below 3.0 without `GL_ARB_vertex_array_object` will have post‑FX disabled for that context. The application still runs normally.

Because the ubershader combines all effects into one code block, some effects may interfere with each other if not designed to be layered. The order is fixed and cannot be changed at runtime.

## Credits & License

The project is heavily inspired by existing reshade tools, but the ubershader implementation and the majority of the effect code were written from scratch (or ported from GLSL snippets found in public domain and open‑source shader repositories).

All 118 effect implementations are original or derived from well‑known algorithms (FXAA by Timothy Lottes, CAS/RCAS from AMD GPUOpen, etc.) and are used under the terms of their respective licenses (mostly MIT/BSD). The overall project is released under the **GNU General Public License v3.0**.

Full text of the GPLv3 is available in the [LICENSE](LICENSE) file.
