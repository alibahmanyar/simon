
# Project variables
PROJECT_NAME := simon
VERSION := 0.1.0
CARGO := cargo
CROSS := cross

# Default target is the native build
.PHONY: all
all: build

# Standard development build
.PHONY: build
build:
	$(CARGO) build

# Release build (optimized)
.PHONY: release
release:
	$(CARGO) build --release

# Run tests
.PHONY: test
test:
	$(CARGO) test

# Clean build artifacts
.PHONY: clean
clean:
	$(CARGO) clean

# Install the application
.PHONY: install
install: release
	cp target/release/$(PROJECT_NAME) /usr/local/bin/

# Cross-compilation targets
#
# Linux targets
.PHONY: linux-x86_64
linux-x86_64:
	$(CROSS) build --release --target x86_64-unknown-linux-gnu

.PHONY: linux-aarch64
linux-aarch64:
	$(CROSS) build --release --target aarch64-unknown-linux-gnu

.PHONY: linux-armv7
linux-armv7:
	$(CROSS) build --release --target armv7-unknown-linux-gnueabihf

.PHONY: linux-i686
linux-i686:
	$(CROSS) build --release --target i686-unknown-linux-gnu

.PHONY: linux-aarch64-musl
linux-aarch64-musl:
	$(CROSS) build --release --target aarch64-unknown-linux-musl 

.PHONY: linux-armv7-musl
linux-armv7-musl:
	$(CROSS) build --release --target armv7-unknown-linux-musleabihf

.PHONY: linux-x86_64-musl
linux-x86_64-musl:
	$(CROSS) build --release --target x86_64-unknown-linux-musl

.PHONY: linux-i686-musl
linux-i686-musl:
	$(CROSS) build --release --target i686-unknown-linux-musl


# Build all supported targets
.PHONY: all-targets
all-targets: linux-x86_64 linux-aarch64 linux-armv7 linux-i686 linux-aarch64-musl linux-armv7-musl linux-x86_64-musl linux-i686-musl

# Install cross-compilation toolchains
.PHONY: install-cross
install-cross:
	cargo install cross --git https://github.com/cross-rs/cross

# Help
.PHONY: help
help:
	@echo "$(PROJECT_NAME) v$(VERSION) Makefile help:"
	@echo ""
	@echo "Standard targets:"
	@echo "  all          Default target, builds in debug mode"
	@echo "  build        Build in debug mode"
	@echo "  release      Build with optimizations"
	@echo "  run          Build and run the project"
	@echo "  test         Run tests"
	@echo "  clean        Remove build artifacts"
	@echo "  install      Install release binary to /usr/local/bin"
	@echo ""
	@echo "Cross-compilation targets:"
	@echo "  linux-x86_64       64-bit Linux (x86_64)"
	@echo "  linux-aarch64      64-bit ARM Linux"
	@echo "  linux-armv7        32-bit ARM Linux"
	@echo "  linux-i686         32-bit Linux (x86)"
	@echo ""
	@echo "Special targets:"
	@echo "  all-targets        Build all supported targets"
	@echo "  install-cross      Install all cross-compilation toolchains"
