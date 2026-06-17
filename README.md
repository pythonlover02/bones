> [!NOTE]
> Bug reports and pull requests are welcome, but please understand that development happens in my free time and progress may be slow at times. The project is still maintained even if the last commit was made a while ago.

# Bones – GLSL ubershader post‑processing tool (OpenGL/Vulkan)

Bones is a realtime post‑processing overlay for Linux games, written in Rust. It intercepts OpenGL and Vulkan swap buffers, applies a single‑pass ubershader with a wide range of effects, and then presents the modified frame. The tool is designed for **performance** and **simplicity**: no custom LUTs, no external shader loading, no custom values, and no ping‑pong. All effects are combined into one shader pass, making memory usage O(1) regardless of how many effects are enabled.

## Table of Contents

- [Minimum Requirements](#minimum-requirements)
- [How It Works](#how-it-works)
- [Features](#features)
- [Installation](#installation)
- [Flatpak](#flatpak)
- [Usage](#usage)
- [Effect Categories](#effect-categories)
- [Performance & Limitations](#performance--limitations)
- [Credits & License](#credits--license)

## Minimum Requirements

### OpenGL path
- **OpenGL 3.0** or newer (released 2008; supported by essentially all discrete and integrated GPUs made in the last 15 years)
- Mesa 7.9+ or any proprietary driver with GL 3.0 support
- GLX or EGL windowing

Contexts that report GL 3.0 or higher automatically satisfy all requirements (core FBO, VAO, GLSL 1.30). Pure GL 2.1 contexts without `GL_ARB_vertex_array_object` will have post‑FX silently disabled for that context; the game runs normally, just without the overlay.

### Vulkan path
- **Vulkan 1.0** or newer
- A driver that supports `VK_KHR_swapchain`

### OpenGL ES path (EGL)
- **OpenGL ES 2.0** or newer

### General
- Linux x86_64
- `glibc 2.17` or newer for native builds; `glibc 2.36` for portable release builds (built inside a Debian Bookworm container)

## How It Works

Bones has two interception paths that work simultaneously:

- **OpenGL** – uses `LD_PRELOAD` to hook `glXSwapBuffers` (GLX) and `eglSwapBuffers` (EGL). Before the real swap, Bones copies the default framebuffer into a texture, draws a full‑screen triangle with the ubershader, and writes the result back.
- **Vulkan** – registers as an explicit Vulkan layer (`VK_LAYER_BONES_overlay`), enabled via `VK_INSTANCE_LAYERS`. At `vkQueuePresentKHR`, the layer copies the swapchain image into an offscreen texture, renders the ubershader into a second offscreen image, and copies the result back to the swapchain image before presentation.

The launcher activates both paths by setting, for the target process: `LD_PRELOAD` (the library path), `VK_ADD_LAYER_PATH` (the directory holding the layer manifest), and `VK_INSTANCE_LAYERS` (to enable the layer). The library and its manifest live together in one directory, and the manifest references the library by a relative path, so the loader resolves it without relying on `ldconfig`.

In both paths the pipeline is the same:

1. Copy the current frame into an input texture.
2. Run a single **ubershader** that combines all enabled effects.
3. Optionally read a history texture for temporal effects.
4. Write the result back and copy it into the history texture for the next frame.

Unlike traditional reshade pipelines that chain multiple shader passes (ping‑pong between FBOs), Bones **never uses more than one draw call**. All effects are compiled into one large shader with `#ifdef` guards. This keeps VRAM usage constant and avoids the overhead of multiple passes.

Because effects are executed in a fixed order (UV warp → spatial → temporal → inline → colour grading), some combinations may produce unexpected results if you enable two effects from the same category. **For best results, enable only one effect per category, unless they are intended to be used together.**

## Features

- **118 built‑in effects** – from geometric warps to anti‑aliasing (FXAA, SMAA‑style), sharpening (CAS, RCAS), temporal smoothing (TAA, optical flow, variance accumulation), bloom, film grain, CRT/OLED/VHS simulation, colour grading, and colour blindness simulation/correction.
- **Ubershader architecture** – all effects are compiled into a single shader. Enabling an effect simply toggles a `#define`.
- **Hot reload** – edit your configuration TOML file while the game is running and the shader recompiles automatically (if `hot_reload = true`).
- **Inline configuration** – pass enabled effects through the `BONES_CONFIG` environment variable for file‑free, reproducible launches.
- **Temporal effects** – many temporal smoothing modes, including motion‑compensated accumulation inspired by DLSS/FSR/XeSS research.
- **Performance‑first** – no custom LUT loading, no external shader scripts, no ping‑pong rendering. Everything is O(1) in VRAM and draw calls.
- **Cross‑API** – works with OpenGL (GLX/EGL) and Vulkan applications.

## Installation

### Pre‑built binaries (if available)

Download the latest release from the [Releases](../../releases) page. Extract it, open a terminal inside the extracted directory, and run:

    sudo make install

This installs the `bones` launcher, `libbones.so`, the Vulkan layer manifest, and any Flatpak extensions included in the release bundle.

### Building from source

    git clone https://github.com/pythonlover02/bones.git
    cd bones
    make

Produces `target/release/bones` and `target/release/libbones.so`.

### Portable release build

Builds the library inside a Debian Bookworm container (for maximum glibc compatibility) and all Flatpak extensions. Requires `podman` or `docker`, `flatpak`, `ostree`, and `python3`.

    make release

### Installing

Run as root after building:

    sudo make install

Default install locations:

| File | Path |
|------|------|
| Launcher | `/usr/local/bin/bones` |
| Library | `/usr/local/lib/bones/libbones.so` |
| Vulkan layer manifest | `/usr/local/lib/bones/VkLayer_bones.json` |

The library and manifest are installed together in `/usr/local/lib/bones/`; the launcher points the Vulkan loader at this directory at runtime, so no `ldconfig` step is required.

To change the prefix:

    sudo make install PREFIX=/opt/bones

To stage into a directory (e.g. for packaging):

    sudo make install DESTDIR=./package

If Flatpak extensions were built beforehand (via `make flatpak` or `make release`) they are installed per‑user for the invoking user as part of `sudo make install`.

### Integrated build (for Proton forks and custom launchers)

If you only need the library and manifest (no launcher) for example to wire Bones into a Proton forks own launch script use:

    make integrated

This builds the library (via `cargo build --release`) and copies `VkLayer_bones.json` into the same directory as `libbones.so` (`target/release/`). No files are installed to system locations. Your launcher is then responsible for setting `LD_PRELOAD`, `VK_ADD_LAYER_PATH`, and `VK_INSTANCE_LAYERS` to point to this build directory or to the DIR where you copy the lib and layer.

This is done to decouple the manifest from the repo source layout. Even if the manifest file is later moved into a subfolder (e.g., `layer/`), the `integrated` target will still place it in `target/release/`, so your build output stays consistent and doesnt breaks.

### Uninstalling

    sudo make remove

This removes the launcher, the library, and the manifest; uninstalls the Flatpak extension (system scope always, and user scope for the invoking user); and removes the per‑user runtime/config directory (`~/.config/bones`). If run directly as root without `sudo`, it prints the remaining `--user` cleanup commands, since it cannot determine which user's Flatpak and config to clean.

### Cleaning build artifacts

    make clean

Runs `cargo clean` and removes built Flatpak bundles and the `.flatpak-work/` staging directory. To clean only Flatpak bundles:

    make flatpak-clean

## Flatpak

Flatpak applications run inside a sandbox and cannot see `LD_PRELOAD`, `VK_ADD_LAYER_PATH`, or `VK_INSTANCE_LAYERS` set on the host those environment variables do not cross the sandbox boundary. Bones supports Flatpak through a **Flatpak extension**: a bundle that mounts the library, the layer manifest, and a small wrapper script (`bones-flatpak`) inside the `org.freedesktop.Platform` runtime, so they are reachable from within the sandbox. The wrapper sets the activation environment **from inside** the sandbox, which is the only place it survives.

Extensions are built for the following runtime versions: `23.08`, `24.08`, `25.08`.

### Build requirements

- `flatpak`
- `ostree`
- `python3`
- A completed `make` or `make release` build

### Building the extensions

    make
    make flatpak

This produces one `.flatpak` bundle per supported runtime version:

    org.freedesktop.Platform.VulkanLayer.bones-23.08.flatpak
    org.freedesktop.Platform.VulkanLayer.bones-24.08.flatpak
    org.freedesktop.Platform.VulkanLayer.bones-25.08.flatpak

To build everything at once use the release target:

    make release

### Installing the extensions

Install the bundle that matches your `org.freedesktop.Platform` runtime version. If unsure which you have, run `flatpak list` and look for `org.freedesktop.Platform`.

    flatpak install --user org.freedesktop.Platform.VulkanLayer.bones-24.08.flatpak

Multiple runtime versions can be installed side by side. If you ran `sudo make install` and the extensions were already built, they are installed automatically for the invoking user. To uninstall:

    flatpak uninstall --user org.freedesktop.Platform.VulkanLayer.bones

### Using Bones with a Flatpak application

Use the `bones` launcher with `flatpak run`:

    bones -- flatpak run com.example.Game

Bones detects the `flatpak run` invocation automatically, reads the application metadata to find its entry point, and rewrites the command to run `bones-flatpak` inside the sandbox. The wrapper then sets the activation environment from within and execs the application. You can pass `flatpak run` flags normally:

    bones -- flatpak run --branch=stable com.example.Game

With a named profile:

    bones myprofile -- flatpak run com.example.Game

### Steam launch option (Flatpak)

For a Flatpak game launched through Steam, set the game's **Launch Options** to run the in‑sandbox wrapper using its absolute path inside the extension mount:

    /usr/lib/extensions/vulkan/bones/bin/bones-flatpak %command%

To use a named profile, prepend `BONES_CONFIG_NAME`:

    BONES_CONFIG_NAME=myprofile /usr/lib/extensions/vulkan/bones/bin/bones-flatpak %command%

This requires the Bones Flatpak extension matching the game's runtime to be installed.

> [!NOTE]
> Inside the sandbox, `bones-flatpak` sets `LD_PRELOAD` to the bundled library (OpenGL path) and `VK_ADD_LAYER_PATH` + `VK_INSTANCE_LAYERS` for the explicit Vulkan layer. Host environment variables do not cross the Flatpak boundary, which is why the wrapper sets them from within. The library and its manifest are mounted together, so the manifest's relative library reference resolves correctly.

## Usage

### Profiles

A profile is a named configuration stored in `~/.config/bones/<name>-config.toml`. The default profile is `bones` (file `bones-config.toml`). On this example it will do and load `~/.config/bones/retro-config.toml`.

    bones retro -- ~/games/retro-game

### Configuration

The default config file is generated at `~/.config/bones/bones-config.toml` on first run, with every effect listed and documented. Set any effect to `true` under its category section to enable it, for example `mirror_horizontal = true` under `[geometric]` or `contrast_adaptive_sharpen = true` under `[sharpening]`. Set `hot_reload = true` under `[general]` to enable live reloading.

### Inline configuration (`BONES_CONFIG`)

Instead of a config file, you can pass enabled effects directly through the `BONES_CONFIG` environment variable as a semicolon‑separated list. When set, it **overrides the config file entirely**, disables hot reload, and enables exactly the listed effects:

    BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen;vibrance_boost" bones -- ~/games/game

- Setting `BONES_CONFIG` even to an empty string takes over from the file. An empty value means "no effects", useful to force a clean pass.
- Unknown effect names are ignored with a warning.
- Hot reload is always off when `BONES_CONFIG` is used.

This is ideal for reproducible launches and for integrating Bones into other launchers without managing config files. It also works with Flatpak pass it on the `bones -- flatpak run …` command line and it is forwarded into the sandbox.

### Hot Reload

When `hot_reload = true`, Bones uses `inotify` to watch the config directory. Save your changes and the shader recompiles and reloads without restarting the game. Hot reload is automatically disabled when configuration comes from `BONES_CONFIG`.

## Effect Categories

Effects are processed in this fixed order (see the TOML file for full lists): Geometric, Denoise, Anti‑aliasing, Sharpening, Local contrast, Blur, Image quality, Display simulation, Overlay, Temporal, Inline, Exposure, Tonemapping, White balance, Colour grading, Channel curves, Colour balance, Selective colour, Stylization, Accessibility.

> [!IMPORTANT]
> **Enable at most one effect per category, unless specified otherwise.** For example, do not enable two different tonemappers or two anti‑aliasing methods. Some combinations (e.g., a sharpener + midtone clarity) are safe and intended. For temporal, pick one primary mode, optionally with `surface_disocclusion_guard` and/or `convergent_detail_recovery`.

## Performance & Limitations

- **VRAM**: O(1) – only three textures (input, output, history) regardless of how many effects are enabled.
- **Draw calls**: Always 1 full‑screen triangle.
- **CPU overhead**: Minimal.
- **Limitations**:
  - No custom shaders or LUTs.
  - No multi‑pass effects (e.g., advanced bloom with downsample chains).
  - Per‑object motion vectors (like true DLSS) are approximated by optical flow from neighbour pixels.
  - Fast motion may cause ghosting on some temporal modes; use `surface_disocclusion_guard` to reduce it.
  - OpenGL contexts below 3.0 without `GL_ARB_vertex_array_object` have post‑FX disabled for that context; the application still runs.

The order is fixed and cannot be changed at runtime.

## Credits & License

The project is heavily inspired by existing reshade tools, but the ubershader implementation and the majority of the effect code were written from scratch (or ported from GLSL snippets found in public‑domain and open‑source shader repositories).

All 118 effect implementations are original or derived from well‑known algorithms (FXAA by Timothy Lottes, CAS/RCAS from AMD GPUOpen, etc.) under their respective licenses (mostly MIT/BSD). The overall project is released under the **GNU General Public License v3.0**. Full text is in the [LICENSE](LICENSE) file.
