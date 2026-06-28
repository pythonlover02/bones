PREFIX  ?= /usr
DESTDIR ?=
bindir  ?= $(PREFIX)/bin
libdir  ?= $(PREFIX)/lib
datadir ?= $(PREFIX)/share

ifneq ($(wildcard /usr/lib/x86_64-linux-gnu/.),)
LIBDIR_64_REL := lib/x86_64-linux-gnu
LIBDIR_32_REL := lib/i386-linux-gnu
else ifneq ($(wildcard /usr/lib32/.),)
LIBDIR_64_REL := lib
LIBDIR_32_REL := lib32
else ifneq ($(wildcard /usr/lib64/.),)
LIBDIR_64_REL := lib64
LIBDIR_32_REL := lib
else
LIBDIR_64_REL := lib
LIBDIR_32_REL := lib32
endif

LIBDIR_64    ?= $(PREFIX)/$(LIBDIR_64_REL)
LIBDIR_32    ?= $(PREFIX)/$(LIBDIR_32_REL)
VK_LAYER_DIR  := $(datadir)/vulkan/implicit_layer.d
DOC_DIR       := $(datadir)/doc/bones
localstatedir ?= /var
STATE_DIR     := $(localstatedir)/lib/bones

TARGET_64 := target/x86_64-unknown-linux-gnu/release/libbones.so
TARGET_32 := target/i686-unknown-linux-gnu/release/libbones.so
BIN       := target/x86_64-unknown-linux-gnu/release/bones

MANIFEST     := VkLayer_bones.json
LICENSE      := LICENSE
DIST_LICENSE := dist.LICENSE

CARGO   ?= cargo
RUSTUP  ?= rustup
CMAKE   ?= cmake
GIT     ?= git
PYTHON3 ?= python3
OSTREE  ?= ostree
FLATPAK ?= flatpak

FORCE_INSTALL ?= 0

RUST_SOURCES := $(wildcard Cargo.toml Cargo.lock) \
  $(shell find . -path ./target -prune -o -name '*.rs' -print 2>/dev/null)

ifeq ($(filter grouped-target,$(.FEATURES)),)
$(error GNU make 4.3+ required)
endif

.PHONY: all 32 release flatpak install flatpak-install uninstall clean flatpak-clean \
        check-root check-sudo-user


all: $(TARGET_64) $(BIN)

$(TARGET_64) $(BIN) &: $(RUST_SOURCES)
	$(CARGO) build --release --target x86_64-unknown-linux-gnu

32: $(TARGET_32)

$(TARGET_32): $(RUST_SOURCES)
	@command -v $(RUSTUP) >/dev/null 2>&1 || { \
	  echo "error: rustup required for 32-bit builds"; exit 1; }
	@$(RUSTUP) target list --installed | grep -q '^i686-unknown-linux-gnu$$' \
	  || $(RUSTUP) target add i686-unknown-linux-gnu
	@command -v $(CMAKE) >/dev/null 2>&1 || { echo "error: $(CMAKE) required"; exit 1; }
	@command -v $(PYTHON3) >/dev/null 2>&1 || { echo "error: $(PYTHON3) required"; exit 1; }
	@command -v $(GIT) >/dev/null 2>&1 || { echo "error: $(GIT) required"; exit 1; }
	# shaderc-from-source: i686 build fails without explicit <cstdint>
	# CMAKE_POLICY_VERSION_MINIMUM: allow CMake 4.x to build deps that declare cmake_minimum_required(3.x<3.5)
	CFLAGS="-m32" CXXFLAGS="-m32 -include cstdint" \
	CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=gcc \
	CMAKE_POLICY_VERSION_MINIMUM=3.5 \
	$(CARGO) build --release \
	    --target i686-unknown-linux-gnu \
	    --features shaderc-from-source


CONTAINER          ?= $(shell command -v podman 2>/dev/null || command -v docker 2>/dev/null || echo podman)
CONTAINER_IMAGE    := rust:1.82-bookworm
CONTAINER_STAMP_64 := target/.container-stamp-64
CONTAINER_STAMP_32 := target/.container-stamp-32

release: $(CONTAINER_STAMP_64) $(CONTAINER_STAMP_32)

$(CONTAINER_STAMP_64): $(RUST_SOURCES)
	@command -v $(CONTAINER) >/dev/null 2>&1 || { \
	  echo "error: podman or docker required"; exit 1; }
	$(CONTAINER) run --rm -v $$(pwd):/src:z -w /src $(CONTAINER_IMAGE) sh -c '\
	  apt-get update -qq && \
	  apt-get install -y -qq cmake python3 git pkg-config 2>/dev/null && \
	  cargo build --release --target x86_64-unknown-linux-gnu --features shaderc-from-source'
	@mkdir -p $(@D)
	@touch $@

$(CONTAINER_STAMP_32): $(RUST_SOURCES)
	@command -v $(CONTAINER) >/dev/null 2>&1 || { \
	  echo "error: podman or docker required"; exit 1; }
	$(CONTAINER) run --rm -v $$(pwd):/src:z -w /src $(CONTAINER_IMAGE) sh -c '\
	  dpkg --add-architecture i386 && apt-get update -qq && \
	  apt-get install -y -qq cmake python3 git pkg-config gcc-multilib g++-multilib 2>/dev/null && \
	  rustup target add i686-unknown-linux-gnu && \
	  CFLAGS="-m32" CXXFLAGS="-m32 -include cstdint" \
	  CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=gcc \
	  CMAKE_POLICY_VERSION_MINIMUM=3.5 \
	  cargo build --release --target i686-unknown-linux-gnu --features shaderc-from-source'
	@mkdir -p $(@D)
	@touch $@


FLATPAK_RUNTIMES := 23.08 24.08 25.08
FLATPAK_OUTDIR   := flatpak
FLATPAK_WORKDIR  := .flatpak-work
FLATPAK_SRCDIR   := ftk
FLATPAK_EXT_ID   := org.freedesktop.Platform.VulkanLayer.bones
FLATPAK_ARCH     := x86_64

FLATPAK_BUNDLES := $(foreach rt,$(FLATPAK_RUNTIMES),\
  $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-$(rt).flatpak)

ifeq ($(and $(wildcard $(TARGET_64)),$(wildcard $(TARGET_32))),)
flatpak:
	@echo "nothing to package build first with 'make' and 'make 32' (or 'make release')"
	@exit 1
else
flatpak: $(FLATPAK_BUNDLES)
endif

$(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-%.flatpak: \
    $(TARGET_64) $(TARGET_32) $(MANIFEST) $(LICENSE) $(DIST_LICENSE) \
    $(FLATPAK_SRCDIR)/bones-flatpak \
    $(FLATPAK_SRCDIR)/commit.py
	@command -v $(FLATPAK) >/dev/null 2>&1 || { echo "error: $(FLATPAK) required"; exit 1; }
	@command -v $(OSTREE)  >/dev/null 2>&1 || { echo "error: $(OSTREE) required";  exit 1; }
	@command -v $(PYTHON3) >/dev/null 2>&1 || { echo "error: $(PYTHON3) required"; exit 1; }
	@mkdir -p $(@D)
	@set -e; \
	rt="$*"; work="$(FLATPAK_WORKDIR).$$rt"; \
	rm -rf $$work; \
	mkdir -p $$work/stage/files/lib/x86_64-linux-gnu \
	         $$work/stage/files/lib/i386-linux-gnu \
	         $$work/stage/files/share/vulkan/implicit_layer.d \
	         $$work/stage/files/share/doc/bones \
	         $$work/stage/files/bin \
	         $$work/repo; \
	$(OSTREE) init --repo=$$work/repo --mode=archive-z2; \
	cp $(TARGET_64) $$work/stage/files/lib/x86_64-linux-gnu/libbones.so; \
	cp $(TARGET_32) $$work/stage/files/lib/i386-linux-gnu/libbones.so; \
	cp $(MANIFEST)  $$work/stage/files/share/vulkan/implicit_layer.d/$(MANIFEST); \
	cp $(LICENSE)      $$work/stage/files/share/doc/bones/$(LICENSE); \
	cp $(DIST_LICENSE) $$work/stage/files/share/doc/bones/$(DIST_LICENSE); \
	cp $(FLATPAK_SRCDIR)/bones-flatpak $$work/stage/files/bin/bones-flatpak; \
	chmod +x $$work/stage/files/bin/bones-flatpak; \
	$(PYTHON3) $(FLATPAK_SRCDIR)/commit.py "$$rt" "$$work/repo" "$$work/stage"; \
	$(FLATPAK) build-bundle --arch=$(FLATPAK_ARCH) $$work/repo $@ \
	  $(FLATPAK_EXT_ID) "$$rt" --runtime; \
	rm -rf $$work


INSTALL_FILES :=
ifneq (,$(wildcard $(BIN)))
INSTALL_FILES += $(DESTDIR)$(bindir)/bones
endif
ifneq (,$(wildcard $(TARGET_64)))
INSTALL_FILES += $(DESTDIR)$(LIBDIR_64)/libbones.so
endif
ifneq (,$(wildcard $(TARGET_32)))
INSTALL_FILES += $(DESTDIR)$(LIBDIR_32)/libbones.so
endif
ifneq (,$(wildcard $(TARGET_64))$(wildcard $(TARGET_32)))
INSTALL_FILES += $(DESTDIR)$(VK_LAYER_DIR)/$(MANIFEST) \
                 $(DESTDIR)$(DOC_DIR)/$(LICENSE) \
                 $(DESTDIR)$(DOC_DIR)/$(DIST_LICENSE)
endif

BUILT_FLATPAKS := $(wildcard $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-*.flatpak)
INSTALLED_FLATPAK_STAMPS := \
  $(BUILT_FLATPAKS:$(FLATPAK_OUTDIR)/%.flatpak=$(STATE_DIR)/.installed-%)

EXISTING_INSTALL := $(strip \
  $(wildcard $(DESTDIR)$(bindir)/bones) \
  $(wildcard $(DESTDIR)$(LIBDIR_64)/libbones.so) \
  $(wildcard $(DESTDIR)$(LIBDIR_32)/libbones.so))

ifneq ($(DESTDIR),)
EXISTING_INSTALL :=
endif
ifneq ($(FORCE_INSTALL),0)
EXISTING_INSTALL :=
endif

ifneq (,$(EXISTING_INSTALL))
install: check-root
	@echo "error: bones or a program with the same name is installed:"
	@for f in $(EXISTING_INSTALL); do echo "  $$f"; done
	@echo "if it's a prior version of bones, run 'make uninstall' first or use FORCE_INSTALL=1"
	@exit 1
else ifeq (,$(strip $(INSTALL_FILES)))
install: check-root
	@echo "nothing to install build first with 'make', 'make 32', or 'make release'"
	@exit 1
else
install: check-root $(INSTALL_FILES)
	@test -n "$(DESTDIR)" || ldconfig 2>/dev/null || true
	@echo "install complete."
	@echo "  bin:       $(bindir)"
	@echo "  lib (64):  $(LIBDIR_64)"
	@echo "  lib (32):  $(LIBDIR_32)"
	@echo "  manifest:  $(VK_LAYER_DIR)"
endif

ifneq ($(DESTDIR),)
flatpak-install: check-root
	@echo "error: flatpak-install does not support DESTDIR"
	@echo "  flatpak runtime extensions are installed per-user via the flatpak CLI,"
	@echo "  not staged into a filesystem tree. package the .flatpak files from"
	@echo "  $(FLATPAK_OUTDIR)/ directly if you're building a distro package."
	@exit 1
else ifeq (,$(strip $(INSTALLED_FLATPAK_STAMPS)))
flatpak-install: check-root
	@echo "nothing to install build first with 'make flatpak'"
	@exit 1
else
flatpak-install: check-root $(INSTALLED_FLATPAK_STAMPS)
	@echo "flatpak extensions installed."
endif

$(DESTDIR)$(bindir)/bones:             $(BIN)          ; install -Dm755 $< $@
$(DESTDIR)$(LIBDIR_64)/libbones.so:    $(TARGET_64)    ; install -Dm755 $< $@
$(DESTDIR)$(LIBDIR_32)/libbones.so:    $(TARGET_32)    ; install -Dm755 $< $@
$(DESTDIR)$(VK_LAYER_DIR)/$(MANIFEST): $(MANIFEST)     ; install -Dm644 $< $@
$(DESTDIR)$(DOC_DIR)/$(LICENSE):       $(LICENSE)      ; install -Dm644 $< $@
$(DESTDIR)$(DOC_DIR)/$(DIST_LICENSE):  $(DIST_LICENSE) ; install -Dm644 $< $@

$(STATE_DIR)/.installed-%: $(FLATPAK_OUTDIR)/%.flatpak | check-sudo-user
	@mkdir -p $(@D)
	@su - "$$SUDO_USER" -c "$(FLATPAK) install --user -y --reinstall '$(abspath $<)'"
	@touch $@

uninstall: check-root
	rm -f  $(DESTDIR)$(bindir)/bones
	rm -f  $(DESTDIR)$(LIBDIR_64)/libbones.so
	rm -f  $(DESTDIR)$(LIBDIR_32)/libbones.so
	rm -f  $(DESTDIR)$(VK_LAYER_DIR)/$(MANIFEST)
	rm -rf $(DESTDIR)$(DOC_DIR)
	@if [ -z "$(DESTDIR)" ]; then \
	  rm -rf $(STATE_DIR); \
	  ldconfig 2>/dev/null || true; \
	  if [ -n "$$SUDO_USER" ]; then \
	    su - "$$SUDO_USER" -c \
	      "flatpak uninstall --user -y $(FLATPAK_EXT_ID) 2>/dev/null" || true; \
	    su - "$$SUDO_USER" -c "rm -rf \"\$$HOME/.config/bones\"" || true; \
	  else \
	    echo "note: SUDO_USER not set run as your user: flatpak uninstall --user $(FLATPAK_EXT_ID)"; \
	  fi; \
	fi

check-root:
	@if [ "$$(id -u)" -ne 0 ]; then \
	  echo "error: needs root run: sudo make $(MAKECMDGOALS)"; \
	  exit 1; \
	fi

check-sudo-user:
	@if [ -z "$$SUDO_USER" ]; then \
	  echo "error: SUDO_USER not set run via 'sudo make ...' from your user shell"; \
	  exit 1; \
	fi

clean:
	$(CARGO) clean
	rm -rf $(FLATPAK_OUTDIR) $(FLATPAK_WORKDIR)*
	rm -f  $(CONTAINER_STAMP_64) $(CONTAINER_STAMP_32)

flatpak-clean:
	rm -rf $(FLATPAK_OUTDIR) $(FLATPAK_WORKDIR)*
