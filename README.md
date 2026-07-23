> [!NOTE]
> Bug reports and pull requests are welcome, but please understand that development happens in my free time and progress may be slow at times. The project is still maintained even if the last commit was made a while ago.

# Bones

**Performance first Vulkan ubershader Post Processing layer for Linux.**

Bones is a realtime post-processing layer for Vulkan games on Linux, written in Rust. It registers as an implicit Vulkan layer (`VK_LAYER_BONES_overlay`), gated by `BONES_ENABLE=1` (set automatically by the launcher for the target process), intercepts `vkQueuePresentKHR`, runs a single ubershader containing 130 effects, and presents the modified frame. It is built around an explicit performance stance: one shader, one pass, no ping-pong, with optional GPU-extension fast paths layered on top.

### At a glance

- **130 effects** across 21 categories: geometry, AA, sharpening, 22 temporal modes, toon / anime rendering, 9 console GPU simulations (PS1 through Xbox 360), CRT / OLED / VHS, colour grading, colourblind correction, more.
- **One shader, one dispatch.** Only the effects you enable are assembled into the ubershader source; disabled effects are never compiled at all. VRAM is O(1) two textures total, regardless of how many effects you stack.
- **Compute path by default**, fragment fallback. Compute skips the rasterizer for tighter scheduling and better extension composition; fragment kicks in automatically when the device doesnt support the storage-image features the compute path needs.
- **Vulkan extension fast paths**: `VK_KHR_dynamic_rendering`, `VK_KHR_push_descriptor`, `VK_KHR_synchronization2`, `VK_KHR_swapchain_mutable_format`. Each is queried at device creation; missing pieces log a single line and the layer keeps running.
- **Async compute**: when the compute path is active, Bones automatically discovers and utilizes a dedicated async compute queue if the device exposes one not used by the game. The compute dispatch runs on the async queue concurrently with the game next-frame rendering; the final blit to the swap image runs on the present queue, with a semaphore between them. Works at any `resolution_scale`.
- **Resolution scale knob**: render the whole pipeline at any fraction of the swapchain (≥0.05) and upscale at the final blit. Big win on weak GPUs and heavy stacks.
- **Skipped input copy**: the swap image is sampled directly via per-image views no spare full-frame copy per frame.
- **Lazy postfx allocation**: when no effects are enabled, nothing is built. Toggle one on at runtime and it spins up on the next present.
- **Env-mode**: define an entire effect stack and every general setting in environment variables. Used together, env vars completely bypass the config file (no read, no write, no watch) perfect for reproducible launches, containers, and Steam Decks.
- **Hot reload** (file mode only): edit the config while the game runs. The shader recompiles live, and pipeline settings (`resolution_scale`, `compute`, etc.) seamlessly rebuild resources on the fly.
- **Dual-arch**: x86_64 + i686 builds in one install, so 32-bit games on Steam / Wine / Proton get the layer too.
- **Flatpak support**, first-class, via a runtime extension. Works with or without the host launcher (atomic distros).

## Quick Start

```
# 1. Build and install
git clone https://github.com/pythonlover02/bones.git
cd bones
make
sudo make install

# 2. Run a game with an effect stack (inline, no config file needed)
BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen;vibrance_boost" bones -- ~/games/mygame
```

Prefer a config file? On first run Bones writes a fully documented `~/.config/bones/bones-config.toml`. Set any effect to `true`, then:

```
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
- [FEX-Emu / Box64](#fex-emu--box64)
- [Flatpak](#flatpak)
- [Flatpak without the launcher (atomic distros)](#flatpak-without-the-launcher-atomic-distros)
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
| **Build** | GNU make 4.3+ (the Makefile uses grouped targets and checks for this) |

Bones is Vulkan-only. OpenGL, OpenGL ES, and other graphics APIs are out of scope: every modern Linux game ships a Vulkan path (native, DXVK, VKD3D-Proton, MoltenVK on translation layers), and focusing on Vulkan lets the layer concentrate every optimization on a single, well-defined pipeline.

The compute path additionally requires the device feature `shaderStorageImageWriteWithoutFormat` and a swap format that supports `VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT`. Missing either of these silently downgrades the swapchain to the fragment-shader path.

Native aarch64 builds are not provided. See [FEX-Emu / Box64](#fex-emu--box64) if you're running aarch64.

## How It Works

Bones registers as an implicit Vulkan layer (`VK_LAYER_BONES_overlay`) via the manifest at `/usr/share/vulkan/implicit_layer.d/VkLayer_bones.json`. The manifest declares `enable_environment = BONES_ENABLE`, so the Vulkan loader always discovers the layer but only activates it when `BONES_ENABLE=1` is set in the target process environment.

The `bones` launcher sets `BONES_ENABLE=1` on the child process before exec on the native path (a plain `Command::exec` with `BONES_ENABLE` and `BONES_CONFIG_NAME` injected into the child env). On the Flatpak path, the launcher rewrites `flatpak run` to add `--env=BONES_CONFIG_NAME=<profile>` and redirects execution through a small wrapper (`bones-flatpak`) shipped inside the Flatpak runtime extension; the wrapper sets `BONES_ENABLE=1`, `VK_ADD_LAYER_PATH`, and `LD_LIBRARY_PATH` inside the sandbox and execs the application entry point.

At `vkQueuePresentKHR`, the layer:

1. Samples the swap image directly through a per-image `ImageView` (no input copy).
2. Runs the **ubershader** (compute by default, fragment when unsupported) into an offscreen `tex_output` at `fx_extent = swap_extent × resolution_scale`.
3. Optionally samples a history texture for temporal effects.
4. Writes `tex_output` back to the swap image: a plain copy at `resolution_scale = 1.0`, otherwise a blit (linear filter when the format advertises `SAMPLED_IMAGE_FILTER_LINEAR`, nearest otherwise). When a temporal mode is enabled, `tex_output` is also copied to `tex_history` for the next frame.

The pipeline is built lazily when no effects are enabled, nothing is allocated. Enabling one via hot reload triggers the build on the next present.

Effects run in a **fixed order** across four semantic stages: *render the ideal image* (geometric warps, denoise, AA, sharpening, blur, image quality, pre-grade light bleed), *grade it deterministically* (exposure, tonemap, white balance, grading, stylization, toon colour, accessibility, vignette / letterbox / dither), *show it on a chosen monitor* (hardware sims act as the display device), then *stabilize what you see* (temporal smoothing runs after all grading and hardware sim, in final display space). Time-varying grain and flicker apply after temporal smoothing so they never enter history, and overlays render last. Temporal history therefore always matches the fully processed frame: exposure, tonemapping, and display masks can never compound through the feedback loop.

> [!NOTE]
> Hardware simulations are **terminal display effects**. AA and sharpen run *before* the sim, so they operate on the ideal frame; the sim then interprets that frame as if it were going through PS1 / CRT / VHS hardware. Temporal smoothing runs *after* the sim, so history always matches the finished, post-sim image.

## Performance Architecture

Bones differentiates itself from ReShade-style postfx pipelines through a series of deliberate architectural choices. None of them are individually exotic; whats unusual is composing all of them in one tool from day one.

### Ubershader instead of pass chaining

Traditional postfx tools chain N passes for N effects, ping-ponging between FBOs. Bones concatenates the GLSL fragment for each enabled effect into one large shader in fixed pipeline order, and compiles that. There is no preprocessor gating and no dead code: a disabled effect never reaches the compiler. The cost: longer first-run shader compile time, and a recompile whenever the effect stack changes. The win: when you stack 5+ effects, the bandwidth saved by not reading and writing a fullscreen render target between each effect is significant on bandwidth-bound GPUs (Steam Deck, mobile dGPUs, older discrete cards). VRAM stays at three textures regardless of effect count.

### Compute by default, fragment fallback

The compute shader path skips the rasterizer entirely no vertex shader, no triangle setup, no ROP path and gives explicit control over workgroup size for shared-memory tap reuse and subgroup ops. On every modern desktop GPU and on Steam Deck the compute path wins consistently when more than a handful of effects are active. Workgroup size defaults to 8×8 and is tunable per axis (`compute_x`, `compute_y`) for hardware where 16×16 or 32×4 wins.

If the device doesnt support `shaderStorageImageWriteWithoutFormat`, or the swap format lacks `VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT`, the layer logs the reason and silently builds the fragment-shader pipeline for that swapchain instead. The fragment path is always present and always working.

### Optional Vulkan extension fast paths

Each is opt-in via TOML (`optimize_*` keys) or env var (`BONES_OPTIMIZE_*`), defaults on, checked at device creation:

| Extension / Feature | What it enables | If missing |
|---|---|---|
| `VK_KHR_dynamic_rendering` | skip render passes / framebuffers | logs warning, uses render passes |
| `VK_KHR_push_descriptor` | inline descriptor writes per draw | logs warning, uses descriptor sets |
| `VK_KHR_synchronization2` | efficient batched CPU pipeline barriers | logs warning, uses legacy barriers |
| `VK_KHR_swapchain_mutable_format` | sample sRGB swap images as UNORM (skip input copy) | logs warning, samples native format |
| Dedicated async compute queue | concurrent post-fx dispatch alongside graphics | logs warning, falls back to present queue |

The log line for a missing extension always looks like:

```
[bones] optimization not applied: <feature> requires <extension>
```

So you know exactly what the device is missing.

### Resolution scale

A single multiplier (`resolution_scale`, default 1.0, minimum 0.05) controls the size of the entire postfx render target relative to the swapchain. Setting it to 0.5 renders all 130 effects at quarter resolution and bilinear-upscales to native at the final blit. On expensive effect stacks this routinely doubles framerate on weaker GPUs at a barely-perceptible cost in sharpness. The history texture follows the same scale, so temporal effects stay aligned.

Because hardware blitting strictly requires a graphics-capable queue, the final blit always runs on the present queue regardless of `resolution_scale`. The compute dispatch itself still runs on the async queue when available, so the two queues overlap. If the physical device or surface format fundamentally lacks blitting capabilities (`BLIT_DST` / `BLIT_SRC`), the layer safely falls back to an unscaled 1:1 pixel copy on the present queue.

### Skip input copy

When the device supports `VK_KHR_swapchain_mutable_format` (toggleable via `optimize_mutable_format`), sRGB swap formats are viewed as their UNORM equivalents directly, so the shader samples the swap image in linear space with no intermediate copy. When the extension isnt available, the swap image is still sampled directly via a same-format view no input copy regardless. This saves one full-frame copy per present compared to the obvious implementation.

### Lazy postfx allocation

Effects → off means resources → not built. The layer registers with the swapchain but allocates nothing until the first present where at least one effect is enabled. Toggling effects on via hot reload triggers the lazy build transparently. This makes the layer essentially free to install a user can leave it active on every game and pay no cost on games where they dont use it.

Shader compilation runs off the present thread, so with effects enabled at launch there is a short window (a second or so, longer with `compute = true` since both the compute and fragment variants are built) where frames present unmodified before postfx spins up. `BONES_LOG=info` logs a single line once the resources are built.

### Env-mode (full file bypass)

Setting any of the following bypasses the config file completely no read, no write, no inotify watch:

`BONES_CONFIG`, `BONES_RESOLUTION_SCALE`, `BONES_OPTIMIZE_DYNAMIC_RENDERING`, `BONES_OPTIMIZE_PUSH_DESCRIPTORS`, `BONES_OPTIMIZE_SYNC2`, `BONES_OPTIMIZE_MUTABLE_FORMAT`, `BONES_OPTIMIZE_ASYNC_COMPUTE`, `BONES_COMPUTE`, `BONES_COMPUTE_X`, `BONES_COMPUTE_Y`

This is the reproducibility story: paste one launch command into a Steam launch option, get one exact behavior, every time. `BONES_LOG` and `BONES_CONFIG_NAME` do not trigger bypass; they are meta-config (log verbosity and which profile to load), not pipeline config.

## Installation

Each Make target does one thing. Nothing builds implicitly except the build targets themselves.

| Command | What it does |
|---------|--------------|
| `make` | Native cargo build → `target/x86_64-unknown-linux-gnu/release/{bones,libbones.so}` |
| `make 32` | 32-bit (i686) build → `target/i686-unknown-linux-gnu/release/libbones.so` |
| `make release` | Same artifacts (64-bit + 32-bit), built in a pinned Debian Bookworm container (glibc 2.36) |
| `make flatpak` | Build `.flatpak` bundles for runtimes 23.08 / 24.08 / 25.08. Requires both `make` and `make 32` to have been run first. |
| `sudo make install` | Install native x64 and x32 binaries that exist in `target/`. **Never builds.** Errors if nothing is built, or if an existing install is detected (run `sudo make uninstall` first or pass `FORCE_INSTALL=1`). |
| `sudo make flatpak-install` | Install built `.flatpak` extensions for the invoking user. Must be run via `sudo` from your user shell (it uses `SUDO_USER`). Does not support `DESTDIR`. |
| `sudo make uninstall` | Remove the launcher, library, manifest, user flatpak extension, and `~/.config/bones` |
| `make clean` | `cargo clean` + remove `flatpak/`, container stamps |
| `make flatpak-clean` | Remove flatpak bundles and workdir only |

### Pre-built binaries (if available)

Download the latest release from the [Releases](../../releases) page, extract, and from inside the extracted directory:

```
sudo make install
```

### Building from source

```
git clone https://github.com/pythonlover02/bones.git
cd bones
make                      # native 64-bit build
make 32                   # optional: 32-bit build for 32-bit games (required for `make flatpak`)
make release              # portable 64-bit + 32-bit build in container (needs podman/docker)
make flatpak              # optional: build flatpak extensions (requires make + make 32 first)
sudo make install         # install native x64 and x32 binaries
sudo make flatpak-install # optional: install flatpak extensions
```

The build requires GNU make 4.3+. The 32-bit build requires `rustup target add i686-unknown-linux-gnu`, `gcc-multilib`, `g++-multilib`, `cmake`, `python3`, and `git`. `make 32` checks for these and prints what is missing. `make release` runs both 64-bit and 32-bit container builds for portable glibc-2.36 artifacts against a pinned `rust:1.82-bookworm` image, so the release stays reproducible across rebuilds.

### Install paths

| File | Default path |
|------|--------------|
| Launcher | `/usr/bin/bones` |
| Library (64-bit) | `/usr/lib/x86_64-linux-gnu/libbones.so` or `/usr/lib64/libbones.so` or `/usr/lib/libbones.so` |
| Library (32-bit) | `/usr/lib/i386-linux-gnu/libbones.so` or `/usr/lib/libbones.so` or `/usr/lib32/libbones.so` |
| Vulkan layer manifest | `/usr/share/vulkan/implicit_layer.d/VkLayer_bones.json` |
| Docs & Licenses | `/usr/share/doc/bones/` |

Because the manifest is installed to the system-wide Vulkan implicit-layer directory and libraries are installed to standard distro-specific library paths, 32-bit games discover the 32-bit layer and 64-bit games discover the 64-bit layer automatically. The Vulkan loader picks up the manifest with no custom `VK_ADD_LAYER_PATH` mapping required, and the launcher gates activation per-process via `BONES_ENABLE=1`.

```
sudo make install DESTDIR=./package    # stage into a directory (packaging)
```

The existing-install check is skipped when `DESTDIR` is set (packaging) or when `FORCE_INSTALL=1` is passed.

> [!WARNING]
> Avoid changing `PREFIX` away from `/usr` or `/usr/local`. The Vulkan loader only scans a fixed set of manifest directories (`/usr/share/vulkan/implicit_layer.d`, `/usr/local/share/vulkan/implicit_layer.d`, `$XDG_DATA_DIRS/vulkan/implicit_layer.d`, and `$VK_LAYER_PATH`). Installing to e.g. `/opt/bones` puts the manifest where nothing reads it, the library where `ldconfig` doesn't see it, and the launcher off `$PATH` the layer will not load even with `BONES_ENABLE=1` set. If you do need a custom prefix, you must also wire up `VK_LAYER_PATH`, `LD_LIBRARY_PATH`, and `PATH` in every shell that runs a Vulkan game.

To install flatpak bundles built with `make flatpak`, run `sudo make flatpak-install`. This installs them per-user for the invoking user. `flatpak-install` does **not** support `DESTDIR`; Flatpak runtime extensions are installed via the `flatpak` CLI, not staged into a filesystem tree. Distro packagers should ship the `.flatpak` files from `flatpak/` directly.

### Uninstalling

```
sudo make uninstall
```

Removes launcher, library (both architectures), manifest, user-scope flatpak extension for the invoking user, and `~/.config/bones`. If run directly as root without `sudo` (no `SUDO_USER`), it skips the user-scope steps and prints the commands to run as your user.

### Cleaning build artifacts

```
make clean          # cargo clean + remove flatpak/, container stamps
make flatpak-clean  # remove only flatpak bundles + workdir
```

## FEX-Emu / Box64

On an aarch64 Linux host, x86_64 Vulkan games are normally run through FEX-Emu or Box64. Bones doesn't ship a native aarch64 build because every shipping Vulkan game on Linux has an x86_64 build, so the translated x86_64 path is what people actually use.

Translation layers typically run the game inside their own root directory a tree containing x86_64 binaries and libraries separate from the aarch64 host `/usr/`. The Bones layer has to be installed into that tree, not the host.

### Recommended: Flatpak (if your host transparently runs x86_64 binaries)

If your aarch64 host is configured to transparently execute x86_64 binaries typically by registering FEX-Emu or Box64 with the kernel `binfmt_misc` so x86_64 ELFs are routed through the translation layer automatically install the Bones x86_64 Flatpak extension into the x86_64 Flatpak runtime. Once the kernel can run x86_64 binaries, an x86_64 Flatpak runtime works like any other Flatpak runtime, and the extension is picked up by games running inside it.

Flatpak itself does **not** translate between architectures. An x86_64 Flatpak runtime contains x86_64 binaries that need an x86_64 CPU to execute; on aarch64, that requires the kernel-level translation setup above. Set that up first, consulting your translation layer documentation, then:

```
make
make 32             # required for the flatpak bundle
make flatpak
sudo make flatpak-install
```

The x86_64 Flatpak runtime must be installed on the host:

```
flatpak install org.freedesktop.Platform//24.08 --arch=x86_64
```

See [Flatpak](#flatpak) for full details.

### Non-Flatpak: install into the translation layer root

Build the x86_64 layer (and optionally the i686 layer) on the host, then point `DESTDIR` at the directory the translation layer uses as its root:

```
make                                                  # build x86_64
make 32                                               # optional, for 32-bit games
sudo make DESTDIR=/path/to/translation-root install
```

The exact path depends on how your translation layer is configured consult its documentation. The destination must be a directory tree where the translated process sees `/usr/` as the host standard layout.

After installing, the layer manifest lives at `<root>/usr/share/vulkan/implicit_layer.d/VkLayer_bones.json` and the libraries at `<root>/usr/lib/x86_64-linux-gnu/libbones.so` (and `.../i386-linux-gnu/libbones.so` if you ran `make 32`). The x86_64 Vulkan loader inside the translated process discovers them via the standard paths no manual env-var routing needed.

To uninstall the layer from the translation layer root:

```
sudo make DESTDIR=/path/to/translation-root uninstall
```

To uninstall the Flatpak extension and the host-side per-user config (`~/.config/bones`):

```
sudo make uninstall
```

These are independent running one doesn't affect the other. If you installed both (extension *and* layer-into-rootfs), run both to fully clean up.

## Flatpak

Flatpak applications run sandboxed and cannot see `VK_ADD_LAYER_PATH` set on the host that variable does not cross the sandbox boundary. Bones supports Flatpak through a **Flatpak extension**: a bundle that mounts the library, the layer manifest, and a small wrapper script (`bones-flatpak`) inside the `org.freedesktop.Platform` runtime, so they are reachable from within the sandbox.

When you run `bones -- flatpak run …`, the launcher rewrites the command to inject `--env=BONES_CONFIG_NAME=<profile>` into the `flatpak run` invocation and redirects execution through `bones-flatpak` inside the sandbox via `--command=`. The wrapper sets `BONES_ENABLE=1`, `VK_ADD_LAYER_PATH`, and `LD_LIBRARY_PATH`, then execs the application entry point (read from `/app/manifest.json`).

Extensions are built for runtime versions `23.08`, `24.08`, and `25.08`.

### Building the extensions

Requires `flatpak`, `ostree`, `python3`, both a 64-bit and a 32-bit build, and a completed `make` (or `make release`) build.

```
make            # 64-bit
make 32         # 32-bit, required by the flatpak bundle
make flatpak
```

This produces one bundle per supported runtime:

```
org.freedesktop.Platform.VulkanLayer.bones-23.08.flatpak
org.freedesktop.Platform.VulkanLayer.bones-24.08.flatpak
org.freedesktop.Platform.VulkanLayer.bones-25.08.flatpak
```

The bundle always includes the 32-bit library, so Flatpak games that run 32-bit Vulkan binaries (uncommon but real for some Wine setups inside Flatpak) get the layer.

### Installing the extensions

Install the bundle matching your `org.freedesktop.Platform` runtime. Run `flatpak list` and look for `org.freedesktop.Platform` if unsure.

```
flatpak install --user org.freedesktop.Platform.VulkanLayer.bones-24.08.flatpak
```

Multiple runtime versions can coexist. If you ran `sudo make flatpak-install` with the extensions already built, theyre installed automatically for the invoking user.

```
flatpak uninstall --user org.freedesktop.Platform.VulkanLayer.bones
```

### Running a Flatpak application through Bones

```
bones -- flatpak run com.example.Game
bones -- flatpak run --branch=stable com.example.Game   # flags pass through
bones myprofile -- flatpak run com.example.Game          # named profile
```

The launcher detects the `flatpak run` invocation and rewrites the command to run `bones-flatpak` inside the sandbox with `BONES_CONFIG_NAME=<profile>` injected as a sandbox env var. The wrapper handles everything else from inside the sandbox.

### Flatpak without the launcher (atomic distros)

On atomic / immutable distros (Fedora Silverblue / Kinoite, Bazzite, SteamOS, …) `/usr/bin` is read-only and `sudo make install` is not viable. Install only the Flatpak extension with `sudo make flatpak-install` and invoke `bones-flatpak` directly via `flatpak run --command=`:

```
flatpak run --command=/usr/lib/extensions/vulkan/bones/bin/bones-flatpak com.example.Game
```

The wrapper exports `BONES_ENABLE=1`, sets the in-sandbox library / layer paths, reads the entry point from `/app/manifest.json`, and execs it. No host side launcher needed. For a named profile or env-mode, prepend the variables on the shell Flatpak passes `BONES_*` env vars through to the sandbox transparently:

```
BONES_CONFIG_NAME=retro flatpak run --command=/usr/lib/extensions/vulkan/bones/bin/bones-flatpak com.example.Game
```

```
BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen" flatpak run --command=/usr/lib/extensions/vulkan/bones/bin/bones-flatpak com.example.Game
```

The config file at `~/.config/bones/` should be visible inside the sandbox through the user home directory, so profiles and hot reload work the same as on the native path. The same applies to env vars Flatpak passes `BONES_*` variables through to the sandbox transparently, so any of them (`BONES_CONFIG`, `BONES_CONFIG_NAME`, `BONES_RESOLUTION_SCALE`, etc.) can be set on the host shell or as Steam launch options and they reach the wrapper unchanged.

### Steam launch option (Flatpak)

Set the game **Launch Options** to the in-sandbox wrapper:

```
/usr/lib/extensions/vulkan/bones/bin/bones-flatpak %command%
```

With env vars (any `BONES_*` variable works, the home directory is mounted so configs and profiles resolve the same way):

```
BONES_CONFIG_NAME=retro /usr/lib/extensions/vulkan/bones/bin/bones-flatpak %command%
```

```
BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen" /usr/lib/extensions/vulkan/bones/bin/bones-flatpak %command%
```

This requires the Bones Flatpak extension matching the game runtime to be installed.

## Usage

The launcher syntax is:

```
bones [profile] -- <command> [args...]
bones -- <command> [args...]              # uses the default profile "bones"
bones --help                              # or -h
```

Pass `--` to separate launcher options from the command. Anything before `--` is treated as an optional profile name; anything after is the command to launch.

### Profiles

A profile is a named config at `~/.config/bones/<name>-config.toml`. The default profile when no name is given is `bones` (`bones-config.toml`). Profile names are validated: a name must be non-empty and consist entirely of printable ASCII, and must not contain path separators or `..`. Anything else (whitespace, control characters, non-ASCII) falls back to the default profile `bones` with a warning. Pass a name to load a different one:

```
bones retro -- ~/games/retro-game   # loads ~/.config/bones/retro-config.toml
```

The profile name also crosses into Flatpak sandboxes via `BONES_CONFIG_NAME`, so `bones retro -- flatpak run com.example.Game` loads the `retro` profile inside the sandbox.

### Configuration file

The default config is generated at `~/.config/bones/bones-config.toml` (or `<profile>-config.toml` if a profile is named) on first run, with every effect listed and documented. Set any effect to `true` under its category section (e.g. `mirror_horizontal = true` under `[geometric]`). Effects are **toggle-only**, all parameters are baked into the shader.

The `[general]` section also controls the architectural settings: `resolution_scale`, `optimize_dynamic_rendering`, `optimize_push_descriptors`, `optimize_sync2`, `optimize_mutable_format`, `optimize_async_compute`, `compute`, `compute_x`, `compute_y`. Every one of these has documentation in the generated config and a corresponding environment variable (see [Environment Variables](#environment-variables)).

In env-mode (see below), no config file is read or written at all the launcher detects env-mode and skips the on-first-run file creation entirely.

### Env-mode (full file bypass)

Setting any of the BONES_* general environment variables takes over from the file entirely no read, no write, no inotify watch. Hot reload is implicitly off in env-mode (nothing to watch). Use this for reproducible launches:

```
BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen;vibrance_boost" \
BONES_RESOLUTION_SCALE=0.75 \
BONES_COMPUTE=true \
bones -- ~/games/game
```

- Setting `BONES_CONFIG`, even to an empty string, takes over from the file. An empty value means "no effects" (a forced clean pass), and thanks to lazy allocation the layer builds nothing.
- Unknown effect names are ignored with a warning.
- Works with Flatpak too: pass any of them on the `bones -- flatpak run …` line and they are forwarded into the sandbox.

### Hot reload (file mode only)

As long as no env-mode `BONES_*` variables are set (`BONES_LOG` and `BONES_CONFIG_NAME` are fine), Bones automatically uses `inotify` to watch the config directory. Save changes and the shader recompiles and reloads without restarting the game. Pipeline settings like `resolution_scale`, `compute`, and temporal toggles are also hot-reloadable; Bones will seamlessly recreate textures and framebuffers on the fly. Device-level optimizations (`optimize_sync2`, etc.) cannot be hot-reloaded and require a game restart. If a reload fails to compile, the previous working shader is kept.

## Environment Variables

### Triggers env-mode (any of these bypasses the file completely)

| Variable | Purpose | Values | Default |
|----------|---------|--------|---------|
| `BONES_CONFIG` | Inline effect list | semicolon-separated effect names (empty = no effects, forced clean pass) | *(unset → use file)* |
| `BONES_RESOLUTION_SCALE` | Postfx render scale | float ≥ 0.05 | `1.0` |
| `BONES_OPTIMIZE_DYNAMIC_RENDERING` | Enable `VK_KHR_dynamic_rendering` | `1`/`true`, `0`/`false` | `true` |
| `BONES_OPTIMIZE_PUSH_DESCRIPTORS` | Enable `VK_KHR_push_descriptor` | `1`/`true`, `0`/`false` | `true` |
| `BONES_OPTIMIZE_SYNC2` | Enable `VK_KHR_synchronization2` | `1`/`true`, `0`/`false` | `true` |
| `BONES_OPTIMIZE_MUTABLE_FORMAT` | Enable `VK_KHR_swapchain_mutable_format` | `1`/`true`, `0`/`false` | `true` |
| `BONES_OPTIMIZE_ASYNC_COMPUTE` | Submit to dedicated async compute queue if available | `1`/`true`, `0`/`false` | `true` |
| `BONES_COMPUTE` | Use compute shader path | `1`/`true`, `0`/`false` | `true` |
| `BONES_COMPUTE_X` | Compute workgroup X | positive integer | `8` |
| `BONES_COMPUTE_Y` | Compute workgroup Y | positive integer | `8` |

Any of these variables triggers env mode when set, even to an empty string. An empty value falls back to that setting default (empty `BONES_CONFIG` = no effects).

### Does not trigger env-mode

| Variable | Purpose | Values | Default |
|----------|---------|--------|---------|
| `BONES_CONFIG_NAME` | Which profile to load. Set automatically by the launcher from the profile argument; can also be set manually. | any profile name (sanitized) | `bones` |
| `BONES_LOG` | Log verbosity (written to stderr) | `off`, `error`, `warn`, `info` | `warn` |

`BONES_ENABLE=1` is set automatically by the launcher on the child process (native path) or by the `bones-flatpak` wrapper inside the sandbox (Flatpak path) to activate the implicit layer. It is not something users set manually. On the Flatpak path, the wrapper additionally sets `VK_ADD_LAYER_PATH` and `LD_LIBRARY_PATH` so the loader can find the manifest and library inside the runtime extension.

## Effect Catalogue

130 effects in 21 categories, applied in the fixed order below. The selection column is the rule of thumb; see the note after the table.

| Order | Category | Section | Count | What it does | Selection |
|------:|----------|---------|------:|--------------|-----------|
| 1 | Geometric | `[geometric]` | 12 | UV / coordinate warps | mostly combinable |
| 2 | Denoise | `[denoise]` | 1 | clean noise before processing | single |
| 3 | Anti-aliasing | `[anti_aliasing]` | 4 | smooth jagged edges | **pick one** |
| 4 | Sharpening | `[sharpening]` | 9 | enhance detail | combinable\* |
| 5 | Local contrast | `[local_contrast]` | 1 | per-pixel "pop" | single |
| 6 | Blur | `[blur]` | 5 | creative / soften | pick one |
| 7 | Image quality | `[image_quality]` | 3 | deband, bloom, lens flare | combinable |
| 8 | Exposure | `[exposure]` | 1 | pre-tonemap exposure | single |
| 9 | Tonemapping | `[tonemapping]` | 8 | HDR → SDR curves | **pick one** |
| 10 | White balance | `[white_balance]` | 3 | colour temperature | **pick one** |
| 11 | Colour grading | `[color_grading]` | 8 | grade / saturation / curves | combinable |
| 12 | Channel curves | `[channel_curves]` | 3 | per-channel S-curves | combinable |
| 13 | Colour balance | `[color_balance]` | 1 | teal / orange tint | single |
| 14 | Selective colour | `[selective_color]` | 3 | per-channel saturation | combinable |
| 15 | Stylization | `[stylization]` | 9 | creative looks | mostly pick one |
| 16 | Toon | `[toon]` | 5 | anime / cartoon rendering | combinable |
| 17 | Accessibility | `[accessibility]` | 6 | CVD simulate / correct | pick one |
| 18 | Inline | `[inline]` | 7 | grain, vignette, letterbox… | stackable |
| 19 | Hardware simulation | `[hardware_simulation]` | 17 | Console GPU sims + CRT / OLED / VHS | one console + one display |
| 20 | Temporal | `[temporal]` | 22 | frame blending / TAA | **one primary** (guards auto) |
| 21 | Overlay (HUD) | `[overlay]` | 2 | FPS, crosshair | combinable |

\* Sharpeners combine (e.g. one sharpener + `midtone_clarity`), but stacking multiple *strong* sharpeners produces halos.

> [!IMPORTANT]
> **Enable at most one effect per category unless noted otherwise.** Dont run two tonemappers or two AA methods. For temporal, pick one primary mode; a fused stabilizer (`convergent_detail_recovery`) activates automatically whenever any temporal mode is enabled. For hardware simulation, pick one console sim and optionally one display sim.

> [!NOTE]
> Combining a temporal mode with a hardware sim is fully supported: temporal runs *after* all grading and hardware sim, so every mode compares like with like history always matches the fully processed frame, and exposure, tonemapping, and display masks can never pulse through the feedback loop. The time-varying slivers (CRT brightness pulse, VHS noise and dropout) apply after temporal smoothing, so they flicker without entering history.

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
| `polynomial_distort` | Brown–Conrady barrel lens distortion (single-term k1) |
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
<summary><b>8. Exposure</b> (1)</summary>

| Effect | Description |
|--------|-------------|
| `linear_exposure` | Pre-tonemap exposure multiplier |
</details>

<details>
<summary><b>9. Tonemapping</b> pick one (8)</summary>

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
<summary><b>10. White balance</b> pick one (3)</summary>

| Effect | Description |
|--------|-------------|
| `neutral_white_balance` | D65 reference correction |
| `warm_temperature` | Warm shift (tungsten) |
| `cool_temperature` | Cool shift (daylight) |
</details>

<details>
<summary><b>11. Colour grading</b> combinable (8)</summary>

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
<summary><b>12. Channel curves</b> combinable (3)</summary>

| Effect | Description |
|--------|-------------|
| `red_channel_curve` | S-curve on red |
| `green_channel_curve` | S-curve on green |
| `blue_channel_curve` | S-curve on blue |
</details>

<details>
<summary><b>13. Colour balance</b> (1)</summary>

| Effect | Description |
|--------|-------------|
| `trizone_color_balance` | Three-zone teal / orange tint |
</details>

<details>
<summary><b>14. Selective colour</b> combinable (3)</summary>

| Effect | Description |
|--------|-------------|
| `red_selective_saturate` | Boost saturation of red-dominant pixels |
| `green_selective_saturate` | Boost saturation of green-dominant pixels |
| `blue_selective_saturate` | Boost saturation of blue-dominant pixels |
</details>

<details>
<summary><b>15. Stylization</b> mostly pick one (9)</summary>

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
<summary><b>16. Toon</b> anime / cartoon (5)</summary>

Anime and cartoon rendering. `kuwahara_paint` is configured here but executes early in the spatial chain (right after denoise), so AA, sharpening, and grading operate on the flattened image. `cel_shade`, `manga_screentone`, and `crosshatch_shade` execute in the deterministic colour stage after stylization. `ink_outline` runs last of the toon effects, just before temporal smoothing, so lines stay on top of cel bands and screentone and outlined edges reject history for free anti-ghosting. All five are deterministic (no time, no random), so temporal history stays aligned.

| Effect | Description |
|--------|-------------|
| `kuwahara_paint` | Kuwahara painterly filter, lowest-variance quadrant mean (flatten regions, keep edges sharp) |
| `cel_shade` | Luminance band quantization, the classic cel-shaded look |
| `manga_screentone` | 45° halftone dots growing in shadows (manga print screentone) |
| `crosshatch_shade` | Diagonal pen hatching, crossed in deep shadow |
| `ink_outline` | Dark ink lines on strong RGB gradients (catches hue edges luma detection misses) |
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

Per-pixel lens and film effects, split across the pipeline. The light-bleed members (`chromatic_aberration`, `red_halation`, `anamorphic_streak`) run before tonemapping, where light scatter physically happens (`chromatic_aberration` at the master fetch). The frame members (`radial_vignette`, `cinematic_letterbox`, `ordered_dither`) run in the deterministic colour stage. `gaussian_grain` applies after temporal smoothing, so the animated grain never enters history.

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

Hardware sims act as the **display device** showing the finished image. AA and sharpen run before the sim so they see the ideal frame; the deterministic UV halves (CRT barrel, VHS ripple, console pixel grids) run in the geometric stage, the colour / mask / quantize halves run in the deterministic colour stage, and the time-varying slivers (CRT brightness pulse, VHS noise and dropout) apply after temporal smoothing so they never enter history.

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
<summary><b>20. Temporal</b> one primary, guards automatic (22)</summary>

22 primary frame-blending modes. A fused stabilizer (`convergent_detail_recovery`) activates automatically whenever any temporal mode is enabled; it gates history influence on large per-pixel changes (acting as a lightweight disocclusion guard) and pulls the current frame toward the converged history to recover detail. Temporal runs after all grading and hardware sim, in final display space, so every mode compares like with like: heavy stylization no longer weakens the flow-based modes, and exposure or tonemap can never pulse through the history. Ordered roughly simplest to most sophisticated.

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
<summary><b>21. Overlay (HUD)</b> combinable (2)</summary>

Overlays render after all processing (colour grading, accessibility, hardware sims, temporal) so they are never affected by other effects.

| Effect | Description |
|--------|-------------|
| `fps_hud` | On-screen FPS counter (7-segment) |
| `crosshair_overlay` | Centred neon crosshair |
</details>

## Recommended Combinations

Copy-paste these as `BONES_CONFIG` values, or set the same keys in your config file. Each respects the one-per-category rule.

```
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

# Anime: painterly flatten + cel bands + ink lines
BONES_CONFIG="kuwahara_paint;cel_shade;ink_outline" bones -- ~/games/game

# Anime on a 90s TV
BONES_CONFIG="cel_shade;ink_outline;crt_simulation" bones -- ~/games/game

# Manga print: grayscale + screentone + crosshatch
BONES_CONFIG="luminance_grayscale;manga_screentone;crosshatch_shade" bones -- ~/games/game

# Colourblind correction (deuteranopia)
BONES_CONFIG="deuteranopia_correct" bones -- ~/games/game

# Half-res render for weak GPUs (combine with any stack above)
BONES_RESOLUTION_SCALE=0.5 BONES_CONFIG="subpixel_aa;contrast_adaptive_sharpen" bones -- ~/games/game
```

The cinematic example deliberately stacks several `[inline]` effects grain, vignette, and letterbox operate independently and are designed to layer.

## Performance & Limitations

**Strengths:**
- **VRAM**: O(1), only two textures (output and history) regardless of effect count, plus a small upscale staging image only when resolution scaling an sRGB swapchain. Swap images are viewed in place, not copied.
- **Dispatch / draw calls**: always one `vkCmdDispatch` on the compute path, one full-screen triangle on fragment.
- **CPU overhead**: minimal command buffers per swap image, and descriptors written inline via `VK_KHR_push_descriptor` when available, or pre-built descriptor sets when it is not.
- **No input copy**: shader samples the swap image directly via per-image views.
- **Lazy build**: zero allocation when no effects are enabled.

**Limitations:**
- Vulkan only. OpenGL and OpenGL ES are out of scope by design.
- No custom shaders or LUTs every effect is built into the ubershader at compile time.
- No multi-pass effects (e.g. advanced bloom with downsample chains). The single-pass discipline rules them out.
- Per-object motion vectors (as in true DLSS) are approximated by optical flow from neighbouring pixels useful but not pixel-perfect on fast motion.
- Fast motion can cause ghosting on some temporal modes; the automatic `convergent_detail_recovery` stabilizer includes a motion gate that softens history influence on large per-pixel changes to reduce it.
- Temporal smoothing runs in final display space, after grading and hardware sims: history always matches the fully processed frame, so deterministic sims stay temporally aligned, but temporal modes never see the pre-grade HDR signal.
- Compute path requires `shaderStorageImageWriteWithoutFormat` and a swap format with `VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT`. Missing either silently falls back to fragment with a log line.
- If postfx submission fails repeatedly on a swapchain (driver in a bad state, etc.), the layer disables itself on that swapchain after a few consecutive failures and logs a single line saying so. Recreating the swapchain (resize, fullscreen toggle) re enables postfx; restarting the game also clears the state. This prevents silent TDR loops at the cost of postfx silently stopping until the swapchain is rebuilt check `BONES_LOG=warn` (or `info`) if effects unexpectedly disappear.

- Bones adds `TRANSFER_SRC` / `TRANSFER_DST` / `SAMPLED` / `COLOR_ATTACHMENT` to the swapchain usage flags so it can read and write the swap images. These are core Vulkan 1.0 usage bits, but a surface is only required to support a subset of them. If the driver rejects the upgraded creation info, the layer retries with the application original parameters and disables postfx for that swapchain, logging a single warning. The game keeps running unmodified.

The effect order is fixed and cannot be changed at runtime.

## Credits & License

Heavily inspired by existing postfx tools, but the ubershader implementation and the majority of the effect code were written from scratch (or ported from GLSL snippets in public-domain and open-source shader repositories).

All 130 effect implementations are original or derived from well-known algorithms (FXAA by Timothy Lottes, CAS / RCAS from AMD GPUOpen, AgX, ACES, and others) under their respective licenses (mostly MIT / BSD). Full attribution and external license texts are in [dist.LICENSE](dist.LICENSE).

The project itself is released under the **GNU General Public License v3.0**, full text in [LICENSE](LICENSE).
