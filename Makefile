PREFIX   ?= /usr/local
DESTDIR  ?=
BINDIR   := $(PREFIX)/bin
LIBDIR   := $(PREFIX)/lib/bones

TARGET   := target/release/libbones.so
BIN      := target/release/bones
MANIFEST := VkLayer_bones.json

CARGO    ?= cargo
CONTAINER ?= $(shell command -v podman 2>/dev/null || command -v docker 2>/dev/null || echo podman)
CONTAINER_IMAGE := rust:bookworm

FLATPAK_RUNTIMES := 23.08 24.08 25.08
FLATPAK_OUTDIR   := flatpak
FLATPAK_WORKDIR  := .flatpak-work
FLATPAK_SRCDIR   := ftk
FLATPAK_EXT_ID   := org.freedesktop.Platform.VulkanLayer.bones
FLATPAK_ARCH     := x86_64
FLATPAK_MOUNT    := /usr/lib/extensions/vulkan/bones
FLATPAK_BIN_ABS  := $(FLATPAK_MOUNT)/bin/bones-inject

.PHONY: all release install integrated remove uninstall clean check-root flatpak flatpak-clean

all:
	$(CARGO) build --release

release:
	@command -v $(CONTAINER) >/dev/null 2>&1 || { echo "error: podman or docker required for releases"; exit 1; }
	@command -v flatpak >/dev/null 2>&1 || { echo "error: flatpak required for releases"; exit 1; }
	@command -v ostree  >/dev/null 2>&1 || { echo "error: ostree required for releases"; exit 1; }
	@command -v python3 >/dev/null 2>&1 || { echo "error: python3 required for releases"; exit 1; }
	@echo "==> building portable 64-bit (Debian Bookworm / glibc 2.36)..."
	$(CONTAINER) run --rm -v $$(pwd):/src:z -w /src $(CONTAINER_IMAGE) sh -c '\
	  apt-get update -qq && \
	  apt-get install -y -qq cmake python3 git pkg-config 2>/dev/null && \
	  cargo build --release --features shaderc-from-source'
	@echo "==> building flatpak extensions..."
	@$(MAKE) flatpak
	@echo
	@echo "release complete:"
	@echo "  $(BIN)"
	@test -f $(TARGET) && echo "  $(TARGET)" || true
	@ls -1 $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-*.flatpak 2>/dev/null || true

check-root:
	@if [ "$$(id -u)" -ne 0 ]; then \
	  echo "error: this target needs root — run with: sudo make $(MAKECMDGOALS)"; \
	  exit 1; \
	fi

install: check-root
	@test -f $(BIN) || { echo "missing $(BIN) — run 'make' first"; exit 1; }
	@test -f $(TARGET) || { echo "missing $(TARGET) — run 'make' first"; exit 1; }
	install -Dm755 $(BIN) $(DESTDIR)$(BINDIR)/bones
	install -Dm755 $(TARGET) $(DESTDIR)$(LIBDIR)/libbones.so
	install -Dm644 $(MANIFEST) $(DESTDIR)$(LIBDIR)/VkLayer_bones.json
	@echo "  $(BINDIR)/bones"
	@echo "  $(LIBDIR)/libbones.so"
	@echo "  $(LIBDIR)/VkLayer_bones.json"
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

integrated:
	@test -f $(TARGET) || { echo "missing $(TARGET) — run 'make' first"; exit 1; }
	install -Dm755 $(TARGET) $(DESTDIR)$(LIBDIR)/libbones.so
	install -Dm644 $(MANIFEST) $(DESTDIR)$(LIBDIR)/VkLayer_bones.json
	@echo "  $(DESTDIR)$(LIBDIR)/libbones.so"
	@echo "  $(DESTDIR)$(LIBDIR)/VkLayer_bones.json"

remove uninstall: check-root
	rm -f $(DESTDIR)$(BINDIR)/bones
	rm -rf $(DESTDIR)$(LIBDIR)
	@if command -v flatpak >/dev/null 2>&1 && [ -n "$${SUDO_USER}" ]; then \
	  su - "$${SUDO_USER}" -c \
	    "flatpak uninstall --user -y $(FLATPAK_EXT_ID) 2>/dev/null" || true; \
	fi

flatpak:
	@test -f $(TARGET) || { \
	  echo "error: missing $(TARGET)"; \
	  echo "  run 'make' for a host build"; \
	  echo "  run 'make release' for a portable build (required for releases)"; \
	  exit 1; \
	}
	@command -v flatpak >/dev/null 2>&1 || { echo "error: flatpak is required"; exit 1; }
	@command -v ostree  >/dev/null 2>&1 || { echo "error: ostree is required (install ostree)"; exit 1; }
	@command -v python3 >/dev/null 2>&1 || { echo "error: python3 is required"; exit 1; }
	@rm -rf $(FLATPAK_OUTDIR)/*.flatpak $(FLATPAK_WORKDIR)
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
	  cp $(TARGET) $(FLATPAK_WORKDIR)/stage/files/lib/libbones.so; \
	  cp $(FLATPAK_SRCDIR)/bones-inject $(FLATPAK_WORKDIR)/stage/files/bin/bones-inject; \
	  chmod +x $(FLATPAK_WORKDIR)/stage/files/bin/bones-inject; \
	  cp $(MANIFEST) $(FLATPAK_WORKDIR)/stage/files/share/vulkan/implicit_layer.d/VkLayer_bones.json; \
	  cp $(MANIFEST) $(FLATPAK_WORKDIR)/stage/files/lib/VkLayer_bones.json; \
	  \
	  cp $(FLATPAK_SRCDIR)/metadata.$$rt $(FLATPAK_WORKDIR)/metadata; \
	  cp $(FLATPAK_SRCDIR)/metadata.$$rt $(FLATPAK_WORKDIR)/stage/metadata; \
	  \
	  python3 $(FLATPAK_SRCDIR)/commit.py \
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
	@echo "  $(FLATPAK_BIN_ABS) %command%"

flatpak-clean:
	rm -rf $(FLATPAK_OUTDIR)/*.flatpak $(FLATPAK_WORKDIR)

clean:
	$(CARGO) clean
	rm -rf $(FLATPAK_OUTDIR)/*.flatpak $(FLATPAK_WORKDIR)
