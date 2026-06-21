PREFIX   ?= /usr/local
DESTDIR  ?=
BINDIR   := $(PREFIX)/bin
LIBDIR   := $(PREFIX)/lib/bones

TARGET          := target/release/libbones.so
BIN             := target/release/bones
INTEGRATED_DIR  := target/release/integrated
MANIFEST        := VkLayer_bones.json
LICENSE         := LICENSE
DIST_LICENSE    := dist.LICENSE

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
FLATPAK_BIN_ABS  := $(FLATPAK_MOUNT)/bin/bones-flatpak

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
	install -Dm644 $(LICENSE) $(DESTDIR)$(LIBDIR)/LICENSE
	install -Dm644 $(DIST_LICENSE) $(DESTDIR)$(LIBDIR)/dist.LICENSE
	@echo "  $(BINDIR)/bones"
	@echo "  $(LIBDIR)/libbones.so"
	@echo "  $(LIBDIR)/VkLayer_bones.json"
	@echo "  $(LIBDIR)/LICENSE"
	@echo "  $(LIBDIR)/dist.LICENSE"
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

integrated: all
	@test -f $(TARGET) || { echo "missing $(TARGET) — build failed"; exit 1; }
	@test -f $(MANIFEST) || { echo "missing $(MANIFEST)"; exit 1; }
	@test -f $(LICENSE) || { echo "missing $(LICENSE)"; exit 1; }
	@test -f $(DIST_LICENSE) || { echo "missing $(DIST_LICENSE)"; exit 1; }
	@rm -rf $(INTEGRATED_DIR)
	@mkdir -p $(INTEGRATED_DIR)
	@cp $(TARGET) $(INTEGRATED_DIR)/libbones.so
	@cp $(MANIFEST) $(INTEGRATED_DIR)/$(MANIFEST)
	@cp $(LICENSE) $(INTEGRATED_DIR)/$(LICENSE)
	@cp $(DIST_LICENSE) $(INTEGRATED_DIR)/$(DIST_LICENSE)
	@echo "integrated build ready in $(INTEGRATED_DIR)/:"
	@echo "  $(INTEGRATED_DIR)/libbones.so"
	@echo "  $(INTEGRATED_DIR)/$(MANIFEST)"
	@echo "  $(INTEGRATED_DIR)/$(LICENSE)"
	@echo "  $(INTEGRATED_DIR)/$(DIST_LICENSE)"
	@echo
	@echo "copy the contents of $(INTEGRATED_DIR)/ wherever your launcher expects them."

remove uninstall: check-root
	rm -f $(DESTDIR)$(BINDIR)/bones
	rm -rf $(DESTDIR)$(LIBDIR)
	@echo "  removed $(BINDIR)/bones"
	@echo "  removed $(LIBDIR)"
	@if command -v flatpak >/dev/null 2>&1; then \
	  flatpak uninstall --system -y $(FLATPAK_EXT_ID) 2>/dev/null \
	    && echo "  removed flatpak extension (system)" || true; \
	  if [ -n "$${SUDO_USER}" ]; then \
	    su - "$${SUDO_USER}" -c \
	      "flatpak uninstall --user -y $(FLATPAK_EXT_ID) 2>/dev/null" \
	      && echo "  removed flatpak extension (user)" || true; \
	    su - "$${SUDO_USER}" -c \
	      "rm -rf \"\$$HOME/.config/bones\"" \
	      && echo "  removed user config/runtime (~/.config/bones)" || true; \
	  else \
	    echo "  note: SUDO_USER not set — could not remove --user flatpak extension"; \
	    echo "        run as your user: flatpak uninstall --user $(FLATPAK_EXT_ID)"; \
	    echo "        and: rm -rf ~/.config/bones"; \
	  fi; \
	fi
	@echo "removed."

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
	    $(FLATPAK_WORKDIR)/repo; \
	  \
	  ostree init --repo=$(FLATPAK_WORKDIR)/repo --mode=archive-z2; \
	  \
	  cp $(TARGET) $(FLATPAK_WORKDIR)/stage/files/lib/libbones.so; \
	  cp $(FLATPAK_SRCDIR)/bones-flatpak $(FLATPAK_WORKDIR)/stage/files/bin/bones-flatpak; \
	  chmod +x $(FLATPAK_WORKDIR)/stage/files/bin/bones-flatpak; \
	  cp $(MANIFEST) $(FLATPAK_WORKDIR)/stage/files/lib/VkLayer_bones.json; \
	  cp $(LICENSE) $(FLATPAK_WORKDIR)/stage/files/lib/LICENSE; \
	  cp $(DIST_LICENSE) $(FLATPAK_WORKDIR)/stage/files/lib/dist.LICENSE; \
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
	@echo "steam (flatpak) launch option — default profile:"
	@echo "  $(FLATPAK_BIN_ABS) %command%"
	@echo "steam (flatpak) launch option — named profile:"
	@echo "  BONES_CONFIG_NAME=myprofile $(FLATPAK_BIN_ABS) %command%"
	@echo
	@echo "any flatpak app via bones:"
	@echo "  bones -- flatpak run APP_ID"

flatpak-clean:
	rm -rf $(FLATPAK_OUTDIR)/*.flatpak $(FLATPAK_WORKDIR)

clean:
	$(CARGO) clean
	rm -rf $(FLATPAK_OUTDIR)/*.flatpak $(FLATPAK_WORKDIR) $(INTEGRATED_DIR)
