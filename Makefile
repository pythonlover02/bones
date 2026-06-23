PREFIX  ?= /usr/local
DESTDIR ?=
BINDIR  := $(PREFIX)/bin
LIBDIR  := $(PREFIX)/lib/bones

TARGET          := target/release/libbones.so
BIN             := target/release/bones
INTEGRATED_DIR  := target/release/integrated
CONTAINER_STAMP := target/release/.container-stamp

MANIFEST     := VkLayer_bones.json
LICENSE      := LICENSE
DIST_LICENSE := dist.LICENSE

CARGO     ?= cargo
CONTAINER ?= $(shell command -v podman 2>/dev/null || command -v docker 2>/dev/null || echo podman)
CONTAINER_IMAGE := rust:bookworm

FLATPAK_RUNTIMES := 23.08 24.08 25.08
FLATPAK_OUTDIR   := flatpak
FLATPAK_WORKDIR  := .flatpak-work
FLATPAK_SRCDIR   := ftk
FLATPAK_EXT_ID   := org.freedesktop.Platform.VulkanLayer.bones
FLATPAK_ARCH     := x86_64

RUST_SOURCES := Cargo.toml Cargo.lock \
  $(shell find . -path ./target -prune -o -name '*.rs' -print 2>/dev/null)

FLATPAK_BUNDLES := $(foreach rt,$(FLATPAK_RUNTIMES),\
  $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-$(rt).flatpak)

INTEGRATED_FILES := \
  $(INTEGRATED_DIR)/libbones.so \
  $(INTEGRATED_DIR)/$(MANIFEST) \
  $(INTEGRATED_DIR)/$(LICENSE) \
  $(INTEGRATED_DIR)/$(DIST_LICENSE)

ifeq ($(filter grouped-target,$(.FEATURES)),)
$(error GNU make 4.3+ required)
endif

check_tool = $(if $(shell command -v $(1) 2>/dev/null),,\
  $(error required tool '$(1)' not in PATH))

ifneq (,$(filter flatpak,$(MAKECMDGOALS)))
ifeq (,$(wildcard $(TARGET)))
$(error nothing in target/ — run 'make' or 'make release' first)
endif
$(foreach t,flatpak ostree python3,$(call check_tool,$(t)))
endif

ifneq (,$(filter integrated,$(MAKECMDGOALS)))
ifeq (,$(wildcard $(TARGET)))
$(error nothing in target/ — run 'make' or 'make release' first)
endif
endif

ifneq (,$(filter release,$(MAKECMDGOALS)))
ifeq ($(shell command -v $(CONTAINER) 2>/dev/null),)
$(error required tool 'podman' or 'docker' not in PATH)
endif
endif

BUILT_LIB      := $(wildcard $(TARGET))
BUILT_BIN      := $(wildcard $(BIN))
BUILT_FLATPAKS := $(wildcard $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-*.flatpak)

ifneq (,$(BUILT_FLATPAKS))
ifneq (,$(filter install,$(MAKECMDGOALS)))
$(call check_tool,flatpak)
endif
endif

INSTALL_FILES :=
ifneq (,$(BUILT_BIN))
INSTALL_FILES += $(DESTDIR)$(BINDIR)/bones
endif
ifneq (,$(BUILT_LIB))
INSTALL_FILES += \
  $(DESTDIR)$(LIBDIR)/libbones.so \
  $(DESTDIR)$(LIBDIR)/$(MANIFEST) \
  $(DESTDIR)$(LIBDIR)/$(LICENSE) \
  $(DESTDIR)$(LIBDIR)/$(DIST_LICENSE)
endif

INSTALLED_FLATPAK_STAMPS := \
  $(BUILT_FLATPAKS:$(FLATPAK_OUTDIR)/%.flatpak=$(DESTDIR)$(LIBDIR)/.installed-%)

.PHONY: all release integrated flatpak install remove uninstall \
        clean flatpak-clean check-root check-sudo-user

all: $(TARGET) $(BIN)

$(TARGET) $(BIN) &: $(RUST_SOURCES)
	$(CARGO) build --release

release: $(CONTAINER_STAMP)

$(CONTAINER_STAMP): $(RUST_SOURCES)
	@echo "==> building portable 64-bit (Debian Bookworm / glibc 2.36)..."
	$(CONTAINER) run --rm -v $$(pwd):/src:z -w /src $(CONTAINER_IMAGE) sh -c '\
	  apt-get update -qq && \
	  apt-get install -y -qq cmake python3 git pkg-config 2>/dev/null && \
	  cargo build --release --features shaderc-from-source'
	@mkdir -p $(@D)
	@touch $@

integrated: $(INTEGRATED_FILES)

$(INTEGRATED_DIR)/libbones.so:     $(TARGET)        ; @mkdir -p $(@D) && cp $< $@
$(INTEGRATED_DIR)/$(MANIFEST):     $(MANIFEST)      ; @mkdir -p $(@D) && cp $< $@
$(INTEGRATED_DIR)/$(LICENSE):      $(LICENSE)       ; @mkdir -p $(@D) && cp $< $@
$(INTEGRATED_DIR)/$(DIST_LICENSE): $(DIST_LICENSE)  ; @mkdir -p $(@D) && cp $< $@

flatpak: $(FLATPAK_BUNDLES)

$(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-%.flatpak: \
    $(TARGET) $(MANIFEST) $(LICENSE) $(DIST_LICENSE) \
    $(FLATPAK_SRCDIR)/bones-flatpak \
    $(FLATPAK_SRCDIR)/metadata.% \
    $(FLATPAK_SRCDIR)/commit.py
	@mkdir -p $(@D)
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

install: check-root $(INSTALL_FILES) $(INSTALLED_FLATPAK_STAMPS)
ifeq (,$(strip $(INSTALL_FILES)$(INSTALLED_FLATPAK_STAMPS)))
	@echo "nothing to install — build first with 'make' (or 'make release') and/or 'make flatpak'"
else
	@echo "install complete."
endif

$(DESTDIR)$(BINDIR)/bones:           $(BIN)          ; install -Dm755 $< $@
$(DESTDIR)$(LIBDIR)/libbones.so:     $(TARGET)       ; install -Dm755 $< $@
$(DESTDIR)$(LIBDIR)/$(MANIFEST):     $(MANIFEST)     ; install -Dm644 $< $@
$(DESTDIR)$(LIBDIR)/$(LICENSE):      $(LICENSE)      ; install -Dm644 $< $@
$(DESTDIR)$(LIBDIR)/$(DIST_LICENSE): $(DIST_LICENSE) ; install -Dm644 $< $@

$(DESTDIR)$(LIBDIR)/.installed-%: $(FLATPAK_OUTDIR)/%.flatpak | check-sudo-user
	@mkdir -p $(@D)
	su - "$$SUDO_USER" -c "flatpak install --user -y --reinstall '$$(pwd)/$<'"
	@touch $@
	@echo "  installed $<"

remove uninstall: check-root
	rm -f  $(DESTDIR)$(BINDIR)/bones
	rm -rf $(DESTDIR)$(LIBDIR)
	@echo "  removed $(BINDIR)/bones and $(LIBDIR)"
	@flatpak uninstall --system -y $(FLATPAK_EXT_ID) 2>/dev/null \
	  && echo "  removed flatpak extension (system)" || true
	@if [ -n "$$SUDO_USER" ]; then \
	  su - "$$SUDO_USER" -c \
	    "flatpak uninstall --user -y $(FLATPAK_EXT_ID) 2>/dev/null" \
	    && echo "  removed flatpak extension (user)" || true; \
	  su - "$$SUDO_USER" -c "rm -rf \"\$$HOME/.config/bones\"" \
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
	@if [ -z "$$SUDO_USER" ]; then \
	  echo "error: SUDO_USER not set — run via 'sudo make ...' from your user shell"; \
	  exit 1; \
	fi

clean:
	$(CARGO) clean
	rm -rf $(INTEGRATED_DIR) $(FLATPAK_OUTDIR) $(FLATPAK_WORKDIR)*
	rm -f  $(CONTAINER_STAMP)

flatpak-clean:
	rm -rf $(FLATPAK_OUTDIR) $(FLATPAK_WORKDIR)*
