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

CARGO     ?= cargo
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

CONTAINER_STAMP       := target/release/.container-stamp
INSTALL_STAMP         := $(DESTDIR)$(LIBDIR)/.installed
FLATPAK_INSTALL_STAMP := $(DESTDIR)$(LIBDIR)/.flatpak-installed

RUST_SOURCES := Cargo.toml Cargo.lock \
  $(shell find . -path ./target -prune -o -name '*.rs' -print 2>/dev/null)

INSTALL_FILES := \
  $(DESTDIR)$(BINDIR)/bones \
  $(DESTDIR)$(LIBDIR)/libbones.so \
  $(DESTDIR)$(LIBDIR)/VkLayer_bones.json \
  $(DESTDIR)$(LIBDIR)/LICENSE \
  $(DESTDIR)$(LIBDIR)/dist.LICENSE

INTEGRATED_FILES := \
  $(INTEGRATED_DIR)/libbones.so \
  $(INTEGRATED_DIR)/$(MANIFEST) \
  $(INTEGRATED_DIR)/$(LICENSE) \
  $(INTEGRATED_DIR)/$(DIST_LICENSE)

FLATPAK_BUNDLES := $(foreach rt,$(FLATPAK_RUNTIMES),\
  $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-$(rt).flatpak)

ifeq ($(filter grouped-target,$(.FEATURES)),)
$(error GNU make 4.3+ required)
endif

check_tool = $(if $(shell command -v $(1) 2>/dev/null),,\
  $(error required tool '$(1)' not in PATH))

RELEASE_GOAL := $(filter release,$(MAKECMDGOALS))
INSTALL_GOAL := $(filter install,$(MAKECMDGOALS))

ifneq (,$(RELEASE_GOAL))
$(foreach t,flatpak ostree python3,$(call check_tool,$(t)))
ifeq ($(shell command -v $(CONTAINER) 2>/dev/null),)
$(error required tool 'podman' or 'docker' not in PATH)
endif
endif
ifneq (,$(filter flatpak-bundles install,$(MAKECMDGOALS)))
$(foreach t,flatpak ostree python3,$(call check_tool,$(t)))
endif

.PHONY: all release integrated install remove uninstall \
        clean flatpak-bundles flatpak-clean check-root check-sudo-user

all: $(TARGET) $(BIN)

$(TARGET) $(BIN) &: $(RUST_SOURCES)
	$(CARGO) build --release

integrated: $(INTEGRATED_FILES)

$(INTEGRATED_DIR):
	mkdir -p $@

$(INTEGRATED_DIR)/libbones.so:     $(TARGET)       | $(INTEGRATED_DIR) ; cp $< $@
$(INTEGRATED_DIR)/$(MANIFEST):     $(MANIFEST)     | $(INTEGRATED_DIR) ; cp $< $@
$(INTEGRATED_DIR)/$(LICENSE):      $(LICENSE)      | $(INTEGRATED_DIR) ; cp $< $@
$(INTEGRATED_DIR)/$(DIST_LICENSE): $(DIST_LICENSE) | $(INTEGRATED_DIR) ; cp $< $@

flatpak-bundles: $(FLATPAK_BUNDLES)

$(FLATPAK_OUTDIR):
	mkdir -p $@

$(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-%.flatpak: \
    $(TARGET) $(MANIFEST) $(LICENSE) $(DIST_LICENSE) \
    $(FLATPAK_SRCDIR)/bones-flatpak \
    $(FLATPAK_SRCDIR)/metadata.% \
    $(FLATPAK_SRCDIR)/commit.py \
    | $(FLATPAK_OUTDIR)
	@rt="$*"; work="$(FLATPAK_WORKDIR).$$rt"; \
	echo "building flatpak extension for runtime $$rt..."; \
	rm -rf $$work; \
	mkdir -p $$work/stage/files/lib $$work/stage/files/bin $$work/repo; \
	ostree init --repo=$$work/repo --mode=archive-z2; \
	cp $(TARGET)       $$work/stage/files/lib/libbones.so; \
	cp $(MANIFEST)     $$work/stage/files/lib/VkLayer_bones.json; \
	cp $(LICENSE)      $$work/stage/files/lib/LICENSE; \
	cp $(DIST_LICENSE) $$work/stage/files/lib/dist.LICENSE; \
	cp $(FLATPAK_SRCDIR)/bones-flatpak $$work/stage/files/bin/bones-flatpak; \
	chmod +x $$work/stage/files/bin/bones-flatpak; \
	cp $(FLATPAK_SRCDIR)/metadata.$$rt $$work/metadata; \
	cp $(FLATPAK_SRCDIR)/metadata.$$rt $$work/stage/metadata; \
	python3 $(FLATPAK_SRCDIR)/commit.py "$$rt" "$$work/metadata" "$$work/repo" "$$work/stage"; \
	flatpak build-bundle --arch=$(FLATPAK_ARCH) $$work/repo $@ \
	  $(FLATPAK_EXT_ID) "$$rt" --runtime; \
	rm -rf $$work

release: $(CONTAINER_STAMP) $(FLATPAK_BUNDLES)
	@echo
	@echo "release complete:"
	@printf '  %s\n' $(BIN) $(TARGET) $(FLATPAK_BUNDLES)

$(CONTAINER_STAMP): $(RUST_SOURCES)
	@echo "==> building portable 64-bit (Debian Bookworm / glibc 2.36)..."
	$(CONTAINER) run --rm -v $$(pwd):/src:z -w /src $(CONTAINER_IMAGE) sh -c '\
	  apt-get update -qq && \
	  apt-get install -y -qq cmake python3 git pkg-config 2>/dev/null && \
	  cargo build --release --features shaderc-from-source'
	@mkdir -p $(dir $@)
	@touch $@

ifneq (,$(RELEASE_GOAL))
$(TARGET) $(BIN): $(CONTAINER_STAMP)
endif

install: check-root $(INSTALL_STAMP) $(FLATPAK_INSTALL_STAMP)
	@echo "installed."

$(INSTALL_STAMP): $(INSTALL_FILES)
	@touch $@

$(FLATPAK_INSTALL_STAMP): check-sudo-user $(FLATPAK_BUNDLES)
	@for f in $(FLATPAK_BUNDLES); do \
	  su - "$${SUDO_USER}" -c "flatpak install --user -y --reinstall \"$$(pwd)/$$f\"" \
	    && echo "  installed $$f" \
	    || { echo "  $$f (failed)"; exit 1; }; \
	done
	@mkdir -p $(dir $@)
	@touch $@

$(DESTDIR)$(BINDIR)/bones:              $(BIN)          ; install -Dm755 $< $@
$(DESTDIR)$(LIBDIR)/libbones.so:        $(TARGET)       ; install -Dm755 $< $@
$(DESTDIR)$(LIBDIR)/VkLayer_bones.json: $(MANIFEST)     ; install -Dm644 $< $@
$(DESTDIR)$(LIBDIR)/LICENSE:            $(LICENSE)      ; install -Dm644 $< $@
$(DESTDIR)$(LIBDIR)/dist.LICENSE:       $(DIST_LICENSE) ; install -Dm644 $< $@

remove uninstall: check-root
	rm -f $(DESTDIR)$(BINDIR)/bones
	rm -rf $(DESTDIR)$(LIBDIR)
	@echo "  removed $(BINDIR)/bones and $(LIBDIR)"
	@flatpak uninstall --system -y $(FLATPAK_EXT_ID) 2>/dev/null \
	  && echo "  removed flatpak extension (system)" || true
	@if [ -n "$${SUDO_USER}" ]; then \
	  su - "$${SUDO_USER}" -c \
	    "flatpak uninstall --user -y $(FLATPAK_EXT_ID) 2>/dev/null" \
	    && echo "  removed flatpak extension (user)" || true; \
	  su - "$${SUDO_USER}" -c "rm -rf \"\$$HOME/.config/bones\"" \
	    && echo "  removed ~/.config/bones" || true; \
	else \
	  echo "  note: SUDO_USER not set — skipped --user flatpak and ~/.config/bones"; \
	  echo "        run as your user: flatpak uninstall --user $(FLATPAK_EXT_ID)"; \
	fi
	@echo "removed."

check-root:
	@if [ "$$(id -u)" -ne 0 ]; then \
	  echo "error: needs root — run: sudo make $(MAKECMDGOALS)"; \
	  exit 1; \
	fi

check-sudo-user:
	@if [ -z "$${SUDO_USER}" ]; then \
	  echo "error: SUDO_USER not set — run via 'sudo make ...' from your user shell"; \
	  exit 1; \
	fi

clean:
	$(CARGO) clean
	rm -rf $(FLATPAK_OUTDIR)/*.flatpak $(FLATPAK_WORKDIR)* $(INTEGRATED_DIR)
	rm -f $(CONTAINER_STAMP)

flatpak-clean:
	rm -rf $(FLATPAK_OUTDIR)/*.flatpak $(FLATPAK_WORKDIR)*
