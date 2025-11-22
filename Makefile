# Project variables
PROJECT_NAME := simon
CARGO := cargo
CROSS := cross
RELEASE_DIR := bin-releases
CLEANUP ?= false

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
	$(CARGO) clean && rm -rf web/build && rm -rf $(RELEASE_DIR)

# Install the application
.PHONY: install
install: release
	cp target/release/$(PROJECT_NAME) /usr/local/bin/

# Cross-compilation targets
#
# Linux targets
.PHONY: linux-x86_64
linux-x86_64:
	@echo "Building for linux-x86_64..."
	$(CROSS) build --release --target x86_64-unknown-linux-gnu
	@mkdir -p $(RELEASE_DIR)
	@cp target/x86_64-unknown-linux-gnu/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-x86_64-linux
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up linux-x86_64 build artifacts..."; \
		rm -rf target/x86_64-unknown-linux-gnu; \
		docker system prune -af; \
	fi

.PHONY: linux-aarch64
linux-aarch64:
	@echo "Building for linux-aarch64..."
	$(CROSS) build --release --target aarch64-unknown-linux-gnu
	@mkdir -p $(RELEASE_DIR)
	@cp target/aarch64-unknown-linux-gnu/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-aarch64-linux
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up linux-aarch64 build artifacts..."; \
		rm -rf target/aarch64-unknown-linux-gnu; \
		docker system prune -af; \
	fi

.PHONY: linux-armv7
linux-armv7:
	@echo "Building for linux-armv7..."
	$(CROSS) build --release --target armv7-unknown-linux-gnueabihf
	@mkdir -p $(RELEASE_DIR)
	@cp target/armv7-unknown-linux-gnueabihf/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-armv7-linux
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up linux-armv7 build artifacts..."; \
		rm -rf target/armv7-unknown-linux-gnueabihf; \
		docker system prune -af; \
	fi

.PHONY: linux-i686
linux-i686:
	@echo "Building for linux-i686..."
	$(CROSS) build --release --target i686-unknown-linux-gnu
	@mkdir -p $(RELEASE_DIR)
	@cp target/i686-unknown-linux-gnu/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-i686-linux
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up linux-i686 build artifacts..."; \
		rm -rf target/i686-unknown-linux-gnu; \
		docker system prune -af; \
	fi

.PHONY: linux-riscv64
linux-riscv64:
	@echo "Building for linux-riscv64..."
	$(CROSS) build --release --target riscv64gc-unknown-linux-gnu
	@mkdir -p $(RELEASE_DIR)
	@cp target/riscv64gc-unknown-linux-gnu/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-riscv64-linux
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up linux-riscv64 build artifacts..."; \
		rm -rf target/riscv64gc-unknown-linux-gnu; \
		docker system prune -af; \
	fi

.PHONY: linux-aarch64-musl
linux-aarch64-musl:
	@echo "Building for linux-aarch64-musl..."
	$(CROSS) build --release --target aarch64-unknown-linux-musl
	@mkdir -p $(RELEASE_DIR)
	@cp target/aarch64-unknown-linux-musl/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-aarch64-linux-musl
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up linux-aarch64-musl build artifacts..."; \
		rm -rf target/aarch64-unknown-linux-musl; \
		docker system prune -af; \
	fi

.PHONY: linux-armv7-musl
linux-armv7-musl:
	@echo "Building for linux-armv7-musl..."
	$(CROSS) build --release --target armv7-unknown-linux-musleabihf
	@mkdir -p $(RELEASE_DIR)
	@cp target/armv7-unknown-linux-musleabihf/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-armv7-linux-musl
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up linux-armv7-musl build artifacts..."; \
		rm -rf target/armv7-unknown-linux-musleabihf; \
		docker system prune -af; \
	fi

.PHONY: linux-x86_64-musl
linux-x86_64-musl:
	@echo "Building for linux-x86_64-musl..."
	$(CROSS) build --release --target x86_64-unknown-linux-musl
	@mkdir -p $(RELEASE_DIR)
	@cp target/x86_64-unknown-linux-musl/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-x86_64-linux-musl
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up linux-x86_64-musl build artifacts..."; \
		rm -rf target/x86_64-unknown-linux-musl; \
		docker system prune -af; \
	fi

.PHONY: linux-i686-musl
linux-i686-musl:
	@echo "Building for linux-i686-musl..."
	$(CROSS) build --release --target i686-unknown-linux-musl
	@mkdir -p $(RELEASE_DIR)
	@cp target/i686-unknown-linux-musl/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-i686-linux-musl
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up linux-i686-musl build artifacts..."; \
		rm -rf target/i686-unknown-linux-musl; \
		docker system prune -af; \
	fi

.PHONY: linux-riscv64-musl
linux-riscv64-musl:
	@echo "Building for linux-riscv64-musl..."
	$(CROSS) build --release --target riscv64gc-unknown-linux-musl
	@mkdir -p $(RELEASE_DIR)
	@cp target/riscv64gc-unknown-linux-musl/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-riscv64-linux-musl
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up linux-riscv64-musl build artifacts..."; \
		rm -rf target/riscv64gc-unknown-linux-musl; \
		docker system prune -af; \
	fi

.PHONY: android-aarch64
android-aarch64:
	@echo "Building for android-aarch64..."
	$(CROSS) build --release --target aarch64-linux-android
	@mkdir -p $(RELEASE_DIR)
	@cp target/aarch64-linux-android/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-aarch64-android
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up android-aarch64 build artifacts..."; \
		rm -rf target/aarch64-linux-android; \
		docker system prune -af; \
	fi

.PHONY: android-armv7
android-armv7:
	@echo "Building for android-armv7..."
	$(CROSS) build --release --target armv7-linux-androideabi
	@mkdir -p $(RELEASE_DIR)
	@cp target/armv7-linux-androideabi/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-armv7-android
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up android-armv7 build artifacts..."; \
		rm -rf target/armv7-linux-androideabi; \
		docker system prune -af; \
	fi

.PHONY: android-x86_64
android-x86_64:
	@echo "Building for android-x86_64..."
	$(CROSS) build --release --target x86_64-linux-android
	@mkdir -p $(RELEASE_DIR)
	@cp target/x86_64-linux-android/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-x86_64-android
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up android-x86_64 build artifacts..."; \
		rm -rf target/x86_64-linux-android; \
		docker system prune -af; \
	fi

.PHONY: windows-x86_64
windows-x86_64:
	@echo "Building for windows-x86_64..."
	$(CROSS) build --release --target x86_64-pc-windows-gnu
	@mkdir -p $(RELEASE_DIR)
	@cp target/x86_64-pc-windows-gnu/release/$(PROJECT_NAME).exe $(RELEASE_DIR)/$(PROJECT_NAME)-x86_64-windows.exe
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up windows-x86_64 build artifacts..."; \
		rm -rf target/x86_64-pc-windows-gnu; \
		docker system prune -af; \
	fi

.PHONY: freebsd-x86_64
freebsd-x86_64:
	@echo "Building for freebsd-x86_64..."
	$(CROSS) build --release --target x86_64-unknown-freebsd
	@mkdir -p $(RELEASE_DIR)
	@cp target/x86_64-unknown-freebsd/release/$(PROJECT_NAME) $(RELEASE_DIR)/$(PROJECT_NAME)-x86_64-freebsd
	@if [ "$(CLEANUP)" = "true" ]; then \
		echo "Cleaning up freebsd-x86_64 build artifacts..."; \
		rm -rf target/x86_64-unknown-freebsd; \
		docker system prune -af; \
	fi


# Build all supported targets
.PHONY: all-targets
all-targets: linux-x86_64 linux-aarch64 linux-armv7 linux-i686 linux-riscv64 linux-aarch64-musl linux-armv7-musl linux-x86_64-musl linux-i686-musl linux-riscv64-musl android-aarch64 android-armv7 android-x86_64 windows-x86_64 freebsd-x86_64


# CI: Build GitHub release artifacts sequentially. For each target this copies the release binary into $(RELEASE_DIR),
# then removes the target build directory and prunes Docker to avoid running out of disk space on CI runners.
.PHONY: ci-release
ci-release:
	@echo "Building all targets for GitHub release..."
	@mkdir -p $(RELEASE_DIR)
	@$(MAKE) linux-x86_64 CLEANUP=true
	@echo "Completed linux-x86_64 - 1/15"
	@$(MAKE) linux-aarch64 CLEANUP=true
	@echo "Completed linux-aarch64 - 2/15"
	@$(MAKE) linux-armv7 CLEANUP=true
	@echo "Completed linux-armv7 - 3/15"
	@$(MAKE) linux-i686 CLEANUP=true
	@echo "Completed linux-i686 - 4/15"
	@$(MAKE) linux-riscv64 CLEANUP=true
	@echo "Completed linux-riscv64 - 5/15"
	@$(MAKE) linux-aarch64-musl CLEANUP=true
	@echo "Completed linux-aarch64-musl - 6/15"
	@$(MAKE) linux-armv7-musl CLEANUP=true
	@echo "Completed linux-armv7-musl - 7/15"
	@$(MAKE) linux-x86_64-musl CLEANUP=true
	@echo "Completed linux-x86_64-musl - 8/15"
	@$(MAKE) linux-i686-musl CLEANUP=true
	@echo "Completed linux-i686-musl - 9/15"
	@$(MAKE) linux-riscv64-musl CLEANUP=true
	@echo "Completed linux-riscv64-musl - 10/15"
	@$(MAKE) android-aarch64 CLEANUP=true
	@echo "Completed android-aarch64 - 11/15"
	@$(MAKE) android-armv7 CLEANUP=true
	@echo "Completed android-armv7 - 12/15"
	@$(MAKE) android-x86_64 CLEANUP=true
	@echo "Completed android-x86_64 - 13/15"
	@$(MAKE) windows-x86_64 CLEANUP=true
	@echo "Completed windows-x86_64 - 14/15"
	@$(MAKE) freebsd-x86_64 CLEANUP=true
	@echo "Completed freebsd-x86_64 - 15/15"
	@echo "Running UPX compression on Linux binaries..."
	@echo "Finished creating GitHub release artifacts in $(RELEASE_DIR)"

# Install cross-compilation toolchains
.PHONY: install-cross
install-cross:
	cargo install cross --git https://github.com/cross-rs/cross

# Install UPX (Ultimate Packer for eXecutables)
.PHONY: install-upx
install-upx:
	@echo "Installing UPX..."
	@if command -v upx >/dev/null 2>&1; then \
		echo "UPX is already installed: $$(upx --version | head -n1)"; \
	else \
		echo "UPX not found. Installing..."; \
		if command -v apt-get >/dev/null 2>&1; then \
			sudo apt-get update && sudo apt-get install -y upx; \
		elif command -v dnf >/dev/null 2>&1; then \
			sudo dnf install -y upx; \
		else \
			echo "ERROR: Package manager not supported. Please install UPX manually."; \
			exit 1; \
		fi; \
	fi

# Compress Linux binaries with UPX
.PHONY: upx-compress
upx-compress:
	@echo "Compressing Linux binaries with UPX..."
	@if ! command -v upx >/dev/null 2>&1; then \
		echo "ERROR: UPX is not installed. Run 'make install-upx' first."; \
		exit 1; \
	fi
	@for binary in $(RELEASE_DIR)/$(PROJECT_NAME)-*-linux $(RELEASE_DIR)/$(PROJECT_NAME)-*-linux-musl; do \
		if [ -f "$$binary" ]; then \
			echo "Compressing $$binary..."; \
			upx --best --lzma "$$binary" || echo "Warning: Failed to compress $$binary"; \
		fi; \
	done
	@echo "UPX compression completed!"

.PHONY: docker
docker:
	docker build -t $(PROJECT_NAME) .

.PHONY: docker-all
docker-all:
	docker buildx build -t $(PROJECT_NAME) --platform linux/amd64,linux/arm64,linux/386,linux/arm/v7,linux/riscv64 .

.PHONY: web
web:
	cd web && bun run build && mkdir -p build/static && cp build/index.html build/auth.html build/favicon.png build/Inter-Regular.woff build/Inter-Regular.woff2 build/RobotoMono-Regular.woff build/RobotoMono-Regular.woff2 build/static

.PHONY: web-setup
web-setup:
	cd web && bun install

# Help
.PHONY: help
help:
	@echo "╔════════════════════════════════════════════════════════════════════════════╗"
	@echo "║                           $(PROJECT_NAME) - Makefile Help                            ║"
	@echo "╚════════════════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "STANDARD TARGETS"
	@echo "  all                    Default target, builds in debug mode"
	@echo "  build                  Build in debug mode"
	@echo "  release                Build with optimizations"
	@echo "  test                   Run all tests"
	@echo "  clean                  Remove all build artifacts and web builds"
	@echo "  install                Install release binary to /usr/local/bin (requires sudo)"
	@echo ""
	@echo "LINUX CROSS-COMPILATION TARGETS"
	@echo "  linux-x86_64           64-bit Linux (glibc)"
	@echo "  linux-aarch64          64-bit ARM Linux (glibc)"
	@echo "  linux-armv7            32-bit ARMv7 Linux (glibc)"
	@echo "  linux-i686             32-bit x86 Linux (glibc)"
	@echo "  linux-riscv64          64-bit RISC-V Linux (glibc)"
	@echo ""
	@echo "LINUX MUSL TARGETS (Static linking)"
	@echo "  linux-x86_64-musl      64-bit Linux (musl, static)"
	@echo "  linux-aarch64-musl     64-bit ARM Linux (musl, static)"
	@echo "  linux-armv7-musl       32-bit ARMv7 Linux (musl, static)"
	@echo "  linux-i686-musl        32-bit x86 Linux (musl, static)"
	@echo "  linux-riscv64-musl     64-bit RISC-V Linux (musl, static)"
	@echo ""
	@echo "ANDROID TARGETS"
	@echo "  android-aarch64        64-bit ARM Android (API 21+)"
	@echo "  android-armv7          32-bit ARMv7 Android (API 16+)"
	@echo "  android-x86_64         64-bit x86_64 Android (API 21+)"
	@echo ""
	@echo "WINDOWS TARGETS"
	@echo "  windows-x86_64         64-bit Windows (MinGW)"
	@echo ""
	@echo "BSD TARGETS"
	@echo "  freebsd-x86_64         64-bit FreeBSD"
	@echo ""
	@echo "BATCH BUILD TARGETS"
	@echo "  all-targets            Build all supported targets (15 platforms)"
	@echo "  build-gh-release       Build all targets for GitHub release (optimized for CI)"
	@echo ""
	@echo "DOCKER TARGETS"
	@echo "  docker                 Build Docker image for current platform"
	@echo "  docker-all             Build multi-arch Docker image (amd64, arm64, 386, armv7)"
	@echo ""
	@echo "WEB FRONTEND TARGETS"
	@echo "  web-setup              Install web dependencies (requires bun)"
	@echo "  web                    Build the web frontend (SvelteKit)"
	@echo ""
	@echo "TOOLCHAIN SETUP"
	@echo "  install-cross          Install cross-compilation toolchain (cargo-cross)"
	@echo "  install-upx            Install UPX (Ultimate Packer for eXecutables)"
	@echo ""
	@echo "COMPRESSION"
	@echo "  upx-compress           Compress Linux binaries in $(RELEASE_DIR) with UPX"
	@echo ""
	@echo "HELP"
	@echo "  help                   Show this help message"
	@echo ""
	@echo "USAGE EXAMPLES:"
	@echo "  make                   # Build in debug mode"
	@echo "  make release           # Build optimized binary"
	@echo "  make linux-aarch64     # Cross-compile for ARM64 Linux"
	@echo "  make all-targets       # Build for all platforms"
	@echo "  make web && make build # Build web frontend and backend"
	@echo ""
	@echo "NOTES:"
	@echo "  • Cross-compilation requires 'cross' (run: make install-cross)"
	@echo "  • Web builds require 'bun' package manager"
	@echo "  • Release binaries are placed in: target/<arch>/release/"
	@echo "  • Batch builds output to: $(RELEASE_DIR)/"
	@echo ""
