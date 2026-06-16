PREFIX   ?= /usr/local
DESTDIR  ?=
BINDIR   := $(PREFIX)/bin
LIBDIR64 := $(PREFIX)/lib
LIBDIR32 := $(PREFIX)/lib32

TARGET64 := target/release/libbones.so
TARGET32 := target/i686-unknown-linux-gnu/release/libbones.so
BIN      := target/release/bones

CARGO    ?= cargo
RUSTUP   ?= rustup
CONTAINER ?= $(shell command -v podman 2>/dev/null || command -v docker 2>/dev/null || echo podman)
CONTAINER_IMAGE := rust:bookworm

FLATPAK_RUNTIMES := 23.08 24.08 25.08
FLATPAK_OUTDIR   := flatpak
FLATPAK_WORKDIR  := .flatpak-work
FLATPAK_EXT_ID   := org.freedesktop.Platform.VulkanLayer.bones
FLATPAK_ARCH     := x86_64
FLATPAK_MOUNT    := /usr/lib/extensions/vulkan/bones
FLATPAK_BIN_ABS  := $(FLATPAK_MOUNT)/bin/bones-inject

.PHONY: all 32 release install remove uninstall clean check-root flatpak flatpak-clean

all:
	$(CARGO) build --release

32:
	@command -v $(RUSTUP) >/dev/null 2>&1 || { \
	  echo "error: rustup is required for 32-bit builds"; exit 1; \
	}
	@$(RUSTUP) target list --installed | grep -q '^i686-unknown-linux-gnu$$' \
	  || $(RUSTUP) target add i686-unknown-linux-gnu
	@command -v cmake  >/dev/null 2>&1 || { echo "error: cmake required (shaderc)"; exit 1; }
	@command -v python >/dev/null 2>&1 || command -v python3 >/dev/null 2>&1 \
	  || { echo "error: python required"; exit 1; }
	@command -v git    >/dev/null 2>&1 || { echo "error: git required"; exit 1; }
	CFLAGS="-m32" \
	CXXFLAGS="-m32 -include cstdint" \
	CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=gcc \
	CMAKE_POLICY_VERSION_MINIMUM=3.5 \
	$(CARGO) build --release \
	    --target i686-unknown-linux-gnu \
	    --features shaderc-from-source

release:
	@command -v $(CONTAINER) >/dev/null 2>&1 || { \
	  echo "error: podman or docker required for releases"; exit 1; \
	}
	@command -v flatpak >/dev/null 2>&1 || { echo "error: flatpak required for releases"; exit 1; }
	@command -v ostree  >/dev/null 2>&1 || { echo "error: ostree required for releases"; exit 1; }
	@command -v python3 >/dev/null 2>&1 || { echo "error: python3 required for releases"; exit 1; }
	@echo "==> building portable 64-bit (Debian Bookworm / glibc 2.36)..."
	$(CONTAINER) run --rm -v $$(pwd):/src:z -w /src $(CONTAINER_IMAGE) sh -c '\
	  apt-get update -qq && \
	  apt-get install -y -qq cmake python3 git pkg-config 2>/dev/null && \
	  cargo build --release --features shaderc-from-source'
	@echo "==> building 32-bit..."
	@$(MAKE) 32 || echo "warning: 32-bit build failed, skipping"
	@echo "==> building flatpak extensions..."
	@$(MAKE) flatpak
	@echo
	@echo "release complete:"
	@echo "  $(BIN)"
	@test -f $(TARGET64) && echo "  $(TARGET64)" || true
	@test -f $(TARGET32) && echo "  $(TARGET32)" || true
	@ls -1 $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-*.flatpak 2>/dev/null || true

check-root:
	@if [ "$$(id -u)" -ne 0 ]; then \
	  echo "error: this target needs root — run with: sudo make $(MAKECMDGOALS)"; \
	  exit 1; \
	fi

install: check-root
	@test -f $(BIN) || { echo "missing $(BIN) — run 'make' first"; exit 1; }
	install -Dm755 $(BIN) $(DESTDIR)$(BINDIR)/bones
	@echo "  $(BINDIR)/bones"
	@if [ -f $(TARGET64) ]; then \
	  install -Dm755 $(TARGET64) $(DESTDIR)$(LIBDIR64)/libbones.so; \
	  echo "  $(LIBDIR64)/libbones.so"; \
	fi
	@if [ -f $(TARGET32) ]; then \
	  install -Dm755 $(TARGET32) $(DESTDIR)$(LIBDIR32)/libbones.so; \
	  echo "  $(LIBDIR32)/libbones.so"; \
	fi
	@command -v ldconfig >/dev/null 2>&1 && ldconfig || true
	@if command -v flatpak >/dev/null 2>&1 && \
	   ls $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-*.flatpak >/dev/null 2>&1; then \
	  echo "installing flatpak extensions..."; \
	  if [ -z "$${SUDO_USER}" ]; then \
	    echo "  warning: SUDO_USER not set, skipping flatpak install"; \
	    echo "  run manually: flatpak install --user RUNTIME.flatpak"; \
	  else \
	    for f in $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-*.flatpak; do \
	      su - "$${SUDO_USER}" -c "flatpak install --user -y \"$$(pwd)/$$f\"" \
	        && echo "  $$f" \
	        || echo "  $$f (failed — run: flatpak install --user $$(pwd)/$$f)"; \
	    done; \
	  fi; \
	else \
	  echo "  no flatpak extensions found in $(FLATPAK_OUTDIR)/, skipping"; \
	  echo "  run 'make release' or 'make flatpak' to build them first"; \
	fi
	@echo
	@echo "installed."

remove uninstall: check-root
	rm -f $(DESTDIR)$(BINDIR)/bones
	rm -f $(DESTDIR)$(LIBDIR64)/libbones.so
	rm -f $(DESTDIR)$(LIBDIR32)/libbones.so
	@command -v ldconfig >/dev/null 2>&1 && ldconfig || true
	@if command -v flatpak >/dev/null 2>&1 && [ -n "$${SUDO_USER}" ]; then \
	  su - "$${SUDO_USER}" -c \
	    "flatpak uninstall --user -y $(FLATPAK_EXT_ID) 2>/dev/null" || true; \
	fi

flatpak:
	@test -f $(TARGET64) || { \
	  echo "error: missing $(TARGET64)"; \
	  echo "  run 'make' for a host build"; \
	  echo "  run 'make release' for a portable build (required for releases)"; \
	  exit 1; \
	}
	@command -v flatpak >/dev/null 2>&1 || { echo "error: flatpak is required"; exit 1; }
	@command -v ostree  >/dev/null 2>&1 || { echo "error: ostree is required (install ostree)"; exit 1; }
	@command -v python3 >/dev/null 2>&1 || { echo "error: python3 is required"; exit 1; }
	@rm -rf $(FLATPAK_OUTDIR) $(FLATPAK_WORKDIR)
	@mkdir -p $(FLATPAK_OUTDIR)
	@for rt in $(FLATPAK_RUNTIMES); do \
	  echo "building extension for runtime $$rt..."; \
	  rm -rf $(FLATPAK_WORKDIR); \
	  mkdir -p \
	    $(FLATPAK_WORKDIR)/stage/files/lib \
	    $(FLATPAK_WORKDIR)/stage/files/bin \
	    $(FLATPAK_WORKDIR)/stage/files/share/vulkan/implicit_layer.d \
	    $(FLATPAK_WORKDIR)/repo; \
	  \
	  ostree init --repo=$(FLATPAK_WORKDIR)/repo --mode=archive-z2; \
	  \
	  cp $(TARGET64) $(FLATPAK_WORKDIR)/stage/files/lib/libbones.so; \
	  if [ -f $(TARGET32) ]; then \
	    cp $(TARGET32) $(FLATPAK_WORKDIR)/stage/files/lib/libbones32.so; \
	    echo "  including 32-bit library"; \
	  fi; \
	  \
	  printf '#!/bin/sh\nexport LD_PRELOAD="$(FLATPAK_MOUNT)/lib/libbones.so$${LD_PRELOAD:+:$$LD_PRELOAD}"\nexport BONES_ACTIVE=1\nexec "$$@"\n' \
	    > $(FLATPAK_WORKDIR)/stage/files/bin/bones-inject; \
	  chmod +x $(FLATPAK_WORKDIR)/stage/files/bin/bones-inject; \
	  \
	  printf '{\n    "file_format_version": "1.2.0",\n    "layer": {\n        "name": "VK_LAYER_BONES_overlay",\n        "type": "GLOBAL",\n        "library_path": "$(FLATPAK_MOUNT)/lib/libbones.so",\n        "api_version": "1.3.0",\n        "implementation_version": "3",\n        "description": "Bones -> Linux GL/Vulkan shader post-FX",\n        "enable_environment": { "BONES_ACTIVE": "1" },\n        "functions": { "vkNegotiateLoaderLayerInterfaceVersion": "vkNegotiateLoaderLayerInterfaceVersion" }\n    }\n}\n' \
	    > $(FLATPAK_WORKDIR)/stage/files/share/vulkan/implicit_layer.d/VkLayer_bones.json; \
	  \
	  printf '[Runtime]\nname=%s\nruntime=org.freedesktop.Platform/%s/%s\nsdk=org.freedesktop.Sdk/%s/%s\n\n[ExtensionOf]\nref=runtime/org.freedesktop.Platform/%s/%s\n' \
	    "$(FLATPAK_EXT_ID)" "$(FLATPAK_ARCH)" "$$rt" "$(FLATPAK_ARCH)" "$$rt" "$(FLATPAK_ARCH)" "$$rt" \
	    > $(FLATPAK_WORKDIR)/metadata; \
	  cp $(FLATPAK_WORKDIR)/metadata $(FLATPAK_WORKDIR)/stage/metadata; \
	  \
	  printf '%s\n' \
	    'import subprocess, sys' \
	    'rt, meta_file, repo, stage = sys.argv[1:]' \
	    'meta = open(meta_file).read()' \
	    'branch = "runtime/org.freedesktop.Platform.VulkanLayer.bones/x86_64/" + rt' \
	    'r = subprocess.run(["ostree", "commit", "--repo=" + repo, "--branch=" + branch, "--subject=bones extension " + rt, "--add-metadata-string=xa.metadata=" + meta, stage])' \
	    'sys.exit(r.returncode)' \
	    > $(FLATPAK_WORKDIR)/commit.py; \
	  python3 $(FLATPAK_WORKDIR)/commit.py \
	    "$$rt" \
	    "$(FLATPAK_WORKDIR)/metadata" \
	    "$(FLATPAK_WORKDIR)/repo" \
	    "$(FLATPAK_WORKDIR)/stage"; \
	  \
	  flatpak build-bundle \
	    --arch=$(FLATPAK_ARCH) \
	    $(FLATPAK_WORKDIR)/repo \
	    $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-$$rt.flatpak \
	    $(FLATPAK_EXT_ID) "$$rt" --runtime; \
	  echo "  -> $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-$$rt.flatpak"; \
	done
	@rm -rf $(FLATPAK_WORKDIR)
	@echo
	@echo "flatpak extensions ready:"
	@ls -1 $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-*.flatpak
	@echo
	@echo "install:"
	@echo "  flatpak install --user $(FLATPAK_EXT_ID)-RUNTIME.flatpak"
	@echo
	@echo "steam (flatpak) launch option:"
	@echo "  $(FLATPAK_BIN_ABS) %command%"
	@echo
	@echo "any flatpak app:"
	@echo "  bones -- flatpak run APP_ID"

flatpak-clean:
	rm -rf $(FLATPAK_OUTDIR) $(FLATPAK_WORKDIR)

clean:
	$(CARGO) clean
	rm -rf $(FLATPAK_OUTDIR) $(FLATPAK_WORKDIR)
