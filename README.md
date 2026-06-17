> [!NOTE]
> Bug reports and pull requests are welcome, but please understand that development happens in my free time and progress may be slow at times. The project is still maintained even if the last commit was made a while ago.

# Bones – GLSL ubershader post‑processing tool (OpenGL/Vulkan)

Bones is a realtime post‑processing overlay for Linux games, written in Rust. It intercepts OpenGL and Vulkan swap buffers, applies a single‑pass ubershader with a wide range of effects, and then presents the modified frame. The tool is designed for **performance** and **simplicity**: no custom LUTs, no external shader loading, no custom values, and no ping‑pong. All effects are combined into one shader pass, making memory usage O(1) regardless of how many effects are enabled.

## Table of Contents

- [Quick start](#quick-start)
- [Installation](#installation)
- [Requirements](#requirements)
- [How It Works](#how-it-works)
- [Flatpak](#flatpak)
- [Configuration](#configuration)
- [Effect Categories](#effect-categories)
- [Performance & Limitations](#performance--limitations)
- [Credits & License](#credits--license)

## Quick start

```bash
git clone https://github.com/pythonlover02/bones.git
cd bones
make
sudo make install
bones -- ./your-game
```

## Installation

### From source
```bash
make
sudo make install
```
The launcher goes to `/usr/local/bin/bones`. The library and Vulkan layer manifest are installed together in `/usr/local/lib/bones/`. No `ldconfig` step is needed.

Optional targets:
- `make release`
- `make flatpak`
- `make clean` / `make flatpak-clean`

### Integrated install (library + manifest only)
```bash
sudo make integrated
```
Your launcher must then set `LD_PRELOAD`, `VK_ADD_LAYER_PATH`, and `VK_INSTANCE_LAYERS`.

### Uninstalling
```bash
sudo make remove
```
Removes the launcher, library, manifest, Flatpak extensions, and `~/.config/bones`.

## Requirements

### OpenGL path
- **OpenGL 3.0** or newer (released 2008)
- Mesa 7.9+ or any proprietary driver with GL 3.0 support
- GLX or EGL

Contexts below 3.0 without `GL_ARB_vertex_array_object` disable the overlay.

### Vulkan path
- **Vulkan 1.0** or newer
- A driver that supports `VK_KHR_swapchain`

### OpenGL ES path (EGL)
- **OpenGL ES 2.0** or newer

### General
- Linux x86_64
- `glibc 2.17` or newer for native builds; `glibc 2.36` for portable release builds

## How It Works

Bones has two interception paths that work simultaneously:

- **OpenGL** – uses `LD_PRELOAD` to hook `glXSwapBuffers` (GLX) and `eglSwapBuffers` (EGL). Before the real swap, Bones copies the default framebuffer into a texture, draws a full‑screen triangle with the ubershader, and writes the result back.
- **Vulkan** – registers as an explicit Vulkan layer (`VK_LAYER_BONES_overlay`), enabled via `VK_INSTANCE_LAYERS`. At `vkQueuePresentKHR`, the layer copies the swapchain image into an offscreen texture, renders the ubershader into a second offscreen image, and copies the result back to the swapchain image before presentation.

The launcher activates both paths by setting, for the target process: `LD_PRELOAD` (the library path), `VK_ADD_LAYER_PATH` (the directory holding the layer manifest), and `VK_INSTANCE_LAYERS` (to enable the layer). The library and its manifest live together in one directory, and the manifest references the library by a relative path, so the loader resolves it without relying on `ldconfig`.

## Flatpak

Flatpak applications run inside a sandbox and cannot see host environment variables. Bones supports Flatpak through a **Flatpak extension**: a bundle that mounts the library, the layer manifest, and a small wrapper script (`bones-flatpak`) inside the `org.freedesktop.Platform` runtime.

Extensions are built for runtime versions `23.08`, `24.08`, `25.08`.

### Build requirements
- `flatpak`, `ostree`, `python3`, and a completed `make` or `make release` build.

### Building the extensions
```bash
make flatpak
```

### Installing the extensions
```bash
flatpak install --user org.freedesktop.Platform.VulkanLayer.bones-24.08.flatpak
```

If you ran `sudo make install` and the extensions were already built, they are installed automatically for the invoking user. To uninstall:
```bash
flatpak uninstall --user org.freedesktop.Platform.VulkanLayer.bones
```

### Using Bones with a Flatpak application
```bash
bones -- flatpak run com.example.Game
```

Bones detects `flatpak run` automatically and rewrites the command to run the wrapper inside the sandbox. You can pass `flatpak run` flags normally. With a named profile:
```bash
bones myprofile -- flatpak run com.example.Game
```

### Steam launch option (Flatpak)
For a Flatpak game launched through Steam, set the game's **Launch Options** to:
```
/usr/lib/extensions/vulkan/bones/bin/bones-flatpak %command%
```
To use a named profile, prepend `BONES_CONFIG_NAME`:
```
BONES_CONFIG_NAME=myprofile /usr/lib/extensions/vulkan/bones/bin/bones-flatpak %command%
```
> Inside the sandbox, `bones-flatpak` sets `LD_PRELOAD` to the bundled library (OpenGL path) and `VK_ADD_LAYER_PATH` + `VK_INSTANCE_LAYERS` for the explicit Vulkan layer.

## Configuration

### Profiles
A profile is a named configuration stored in `~/.config/bones/<name>-config.toml`. The default profile is `bones` (file `bones-config.toml`).
```bash
bones retro -- ~/games/retro-game
```

### TOML file
The default config file is generated at `~/.config/bones/bones-config.toml` on first run. Set any effect to `true` under its category section. Enable `hot_reload = true` under `[general]` to watch for changes.

### Inline configuration (`BONES_CONFIG`)
Instead of a config file, you can pass enabled effects directly through the `BONES_CONFIG` environment variable as a semicolon‑separated list. When set, it **overrides the config file entirely**, disables hot reload, and enables exactly the listed effects:
```bash
BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen;vibrance_boost" bones -- ~/games/game
```
- An empty value means "no effects".
- Unknown effect names are ignored with a warning.
- Hot reload is always off when `BONES_CONFIG` is used.

This also works with Flatpak.

### Hot Reload
When `hot_reload = true`, Bones uses `inotify` to watch the config directory. Save your changes and the shader recompiles and reloads without restarting the game. Hot reload is automatically disabled when configuration comes from `BONES_CONFIG`.

## Effect Categories

Effects are processed in this fixed order (see the TOML file for full lists): Geometric, Denoise, Anti‑aliasing, Sharpening, Local contrast, Blur, Image quality, Display simulation, Overlay, Temporal, Inline, Exposure, Tonemapping, White balance, Colour grading, Channel curves, Colour balance, Selective colour, Stylization, Accessibility.

> [!IMPORTANT]
> Enable at most one effect per category, unless specified otherwise.

## Performance & Limitations

- **VRAM**: O(1) – only three textures (input, output, history).
- **Draw calls**: Always 1 full‑screen triangle.
- **CPU overhead**: Minimal.
- **Limitations**:
  - No custom shaders or LUTs.
  - No multi‑pass effects.
  - Motion vectors are approximated by optical flow; fast motion may cause ghosting.
  - OpenGL contexts below 3.0 without VAO support disable the overlay.

The order is fixed and cannot be changed at runtime.

## Credits & License

The project is heavily inspired by existing reshade tools, but the ubershader implementation and the majority of the effect code were written from scratch (or ported from GLSL snippets found in public‑domain and open‑source shader repositories).

All 118 effect implementations are original or derived from well‑known algorithms (FXAA by Timothy Lottes, CAS/RCAS from AMD GPUOpen, etc.) under their respective licenses (mostly MIT/BSD). The overall project is released under the **GNU General Public License v3.0**. Full text is in the [LICENSE](LICENSE) file.
