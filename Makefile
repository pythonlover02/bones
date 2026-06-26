PREFIX  ?= /usr/local
DESTDIR ?=
BINDIR  := $(PREFIX)/bin
LIBDIR  := $(PREFIX)/lib/bones

TARGET_64 := target/release/libbones.so
TARGET_32 := target/i686-unknown-linux-gnu/release/libbones.so
BIN       := target/release/bones

INTEGRATED_DIR := target/release/integrated
INTEGRATED_64  := $(INTEGRATED_DIR)/x86_64
INTEGRATED_32  := $(INTEGRATED_DIR)/i686

MANIFEST     := VkLayer_bones.json
LICENSE      := LICENSE
DIST_LICENSE := dist.LICENSE

CARGO     ?= cargo
RUSTUP    ?= rustup
CONTAINER ?= $(shell command -v podman 2>/dev/null || command -v docker 2>/dev/null || echo podman)
CONTAINER_IMAGE := rust:bookworm

FLATPAK_RUNTIMES := 23.08 24.08 25.08
FLATPAK_OUTDIR   := flatpak
FLATPAK_WORKDIR  := .flatpak-work
FLATPAK_SRCDIR   := ftk
FLATPAK_EXT_ID   := org.freedesktop.Platform.VulkanLayer.bones
FLATPAK_ARCH     := x86_64

CONTAINER_STAMP_64 := target/release/.container-stamp-64
CONTAINER_STAMP_32 := target/release/.container-stamp-32

RUST_SOURCES := Cargo.toml Cargo.lock \
  $(shell find . -path ./target -prune -o -name '*.rs' -print 2>/dev/null)

FLATPAK_BUNDLES := $(foreach rt,$(FLATPAK_RUNTIMES),\
  $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-$(rt).flatpak)

FLATPAK_LIB_DEPS_32 := $(wildcard $(TARGET_32))

BUILT_LIB_64   := $(wildcard $(TARGET_64))
BUILT_LIB_32   := $(wildcard $(TARGET_32))
BUILT_BIN      := $(wildcard $(BIN))
BUILT_FLATPAKS := $(wildcard $(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-*.flatpak)

INSTALL_FILES :=
ifneq (,$(BUILT_BIN))
INSTALL_FILES += $(DESTDIR)$(BINDIR)/bones
endif
ifneq (,$(BUILT_LIB_64))
INSTALL_FILES += \
  $(DESTDIR)$(LIBDIR)/x86_64/libbones.so \
  $(DESTDIR)$(LIBDIR)/x86_64/$(MANIFEST)
endif
ifneq (,$(BUILT_LIB_32))
INSTALL_FILES += \
  $(DESTDIR)$(LIBDIR)/i686/libbones.so \
  $(DESTDIR)$(LIBDIR)/i686/$(MANIFEST)
endif
ifneq (,$(BUILT_LIB_64)$(BUILT_LIB_32))
INSTALL_FILES += \
  $(DESTDIR)$(LIBDIR)/$(LICENSE) \
  $(DESTDIR)$(LIBDIR)/$(DIST_LICENSE)
endif

INSTALLED_FLATPAK_STAMPS := \
  $(BUILT_FLATPAKS:$(FLATPAK_OUTDIR)/%.flatpak=$(DESTDIR)$(LIBDIR)/.installed-%)

ifeq ($(filter grouped-target,$(.FEATURES)),)
$(error GNU make 4.3+ required)
endif

.PHONY: all 32 release integrated flatpak install remove uninstall \
        clean flatpak-clean check-root check-sudo-user

all: $(TARGET_64) $(BIN)

$(TARGET_64) $(BIN) &: $(RUST_SOURCES)
	$(CARGO) build --release

32: $(TARGET_32)

$(TARGET_32): $(RUST_SOURCES)
	@command -v $(RUSTUP) >/dev/null 2>&1 || { \
	  echo "error: rustup required for 32-bit builds"; exit 1; }
	@$(RUSTUP) target list --installed | grep -q '^i686-unknown-linux-gnu$$' \
	  || $(RUSTUP) target add i686-unknown-linux-gnu
	@command -v cmake >/dev/null 2>&1 || { echo "error: cmake required"; exit 1; }
	@command -v python3 >/dev/null 2>&1 || { echo "error: python3 required"; exit 1; }
	@command -v git >/dev/null 2>&1 || { echo "error: git required"; exit 1; }
	CFLAGS="-m32" CXXFLAGS="-m32 -include cstdint" \
	CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=gcc \
	CMAKE_POLICY_VERSION_MINIMUM=3.5 \
	$(CARGO) build --release \
	    --target i686-unknown-linux-gnu \
	    --features shaderc-from-source

release: $(CONTAINER_STAMP_64) $(CONTAINER_STAMP_32)

$(CONTAINER_STAMP_64): $(RUST_SOURCES)
	@command -v $(CONTAINER) >/dev/null 2>&1 || { \
	  echo "error: podman or docker required"; exit 1; }
	@echo "==> building portable 64-bit (Debian Bookworm / glibc 2.36)..."
	$(CONTAINER) run --rm -v $$(pwd):/src:z -w /src $(CONTAINER_IMAGE) sh -c '\
	  apt-get update -qq && \
	  apt-get install -y -qq cmake python3 git pkg-config 2>/dev/null && \
	  cargo build --release --features shaderc-from-source'
	@mkdir -p $(@D)
	@touch $@

$(CONTAINER_STAMP_32): $(RUST_SOURCES)
	@command -v $(CONTAINER) >/dev/null 2>&1 || { \
	  echo "error: podman or docker required"; exit 1; }
	@echo "==> building portable 32-bit (Debian Bookworm / glibc 2.36)..."
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

integrated: \
  $(INTEGRATED_64)/libbones.so $(INTEGRATED_64)/$(MANIFEST) \
  $(INTEGRATED_DIR)/$(LICENSE) $(INTEGRATED_DIR)/$(DIST_LICENSE)

$(INTEGRATED_64)/libbones.so: $(TARGET_64)
	@mkdir -p $(@D)
	cp $< $@

$(INTEGRATED_64)/$(MANIFEST): $(MANIFEST)
	@mkdir -p $(@D)
	cp $< $@

$(INTEGRATED_32)/libbones.so: $(TARGET_32)
	@mkdir -p $(@D)
	cp $< $@

$(INTEGRATED_32)/$(MANIFEST): $(MANIFEST)
	@mkdir -p $(@D)
	cp $< $@

$(INTEGRATED_DIR)/$(LICENSE): $(LICENSE)
	@mkdir -p $(@D)
	cp $< $@

$(INTEGRATED_DIR)/$(DIST_LICENSE): $(DIST_LICENSE)
	@mkdir -p $(@D)
	cp $< $@

flatpak: $(FLATPAK_BUNDLES)

$(FLATPAK_OUTDIR)/$(FLATPAK_EXT_ID)-%.flatpak: \
    $(TARGET_64) $(FLATPAK_LIB_DEPS_32) $(MANIFEST) $(LICENSE) $(DIST_LICENSE) \
    $(FLATPAK_SRCDIR)/bones-flatpak \
    $(FLATPAK_SRCDIR)/metadata.% \
    $(FLATPAK_SRCDIR)/commit.py
	@command -v flatpak >/dev/null 2>&1 || { echo "error: flatpak required"; exit 1; }
	@command -v ostree  >/dev/null 2>&1 || { echo "error: ostree required";  exit 1; }
	@command -v python3 >/dev/null 2>&1 || { echo "error: python3 required"; exit 1; }
	@mkdir -p $(@D)
	@rt="$*"; work="$(FLATPAK_WORKDIR).$$rt"; \
	echo "building flatpak extension for runtime $$rt..."; \
	rm -rf $$work; \
	mkdir -p $$work/stage/files/lib/x86_64 $$work/stage/files/bin $$work/repo; \
	ostree init --repo=$$work/repo --mode=archive-z2; \
	cp $(TARGET_64) $$work/stage/files/lib/x86_64/libbones.so; \
	cp $(MANIFEST)  $$work/stage/files/lib/x86_64/VkLayer_bones.json; \
	if [ -n "$(FLATPAK_LIB_DEPS_32)" ]; then \
	  mkdir -p $$work/stage/files/lib/i686; \
	  cp $(TARGET_32) $$work/stage/files/lib/i686/libbones.so; \
	  cp $(MANIFEST)  $$work/stage/files/lib/i686/VkLayer_bones.json; \
	fi; \
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
	@echo "nothing to install — build first with 'make', 'make 32', or 'make release', and/or 'make flatpak'"
else
	@echo "install complete."
endif

$(DESTDIR)$(BINDIR)/bones:                   $(BIN)          ; install -Dm755 $< $@
$(DESTDIR)$(LIBDIR)/x86_64/libbones.so:      $(TARGET_64)    ; install -Dm755 $< $@
$(DESTDIR)$(LIBDIR)/x86_64/$(MANIFEST):      $(MANIFEST)     ; install -Dm644 $< $@
$(DESTDIR)$(LIBDIR)/i686/libbones.so:        $(TARGET_32)    ; install -Dm755 $< $@
$(DESTDIR)$(LIBDIR)/i686/$(MANIFEST):        $(MANIFEST)     ; install -Dm644 $< $@
$(DESTDIR)$(LIBDIR)/$(LICENSE):              $(LICENSE)      ; install -Dm644 $< $@
$(DESTDIR)$(LIBDIR)/$(DIST_LICENSE):         $(DIST_LICENSE) ; install -Dm644 $< $@

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
	rm -f  $(CONTAINER_STAMP_64) $(CONTAINER_STAMP_32)

flatpak-clean:
	rm -rf $(FLATPAK_OUTDIR) $(FLATPAK_WORKDIR)*
