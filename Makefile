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

.PHONY: all build build-32 build-all install remove uninstall clean check-root

all: build

build:
	$(CARGO) build --release

build-32:
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

build-all: build build-32

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
	@echo
	@echo "installed."

remove uninstall: check-root
	rm -f $(DESTDIR)$(BINDIR)/bones
	rm -f $(DESTDIR)$(LIBDIR64)/libbones.so
	rm -f $(DESTDIR)$(LIBDIR32)/libbones.so
	@command -v ldconfig >/dev/null 2>&1 && ldconfig || true

clean:
	$(CARGO) clean
