# ─────────────────────────────────────────────────────────────────────────────
# DropShot — Makefile
#
# Output layout
#   bin/windows/   — compiled Windows binary / raw Tauri bundle
#   bin/linux/     — compiled Linux binary / raw Tauri bundle
#   bin/mac/       — compiled macOS .app bundle
#
#   install/windows/ — final Windows installer  (.msi, .exe)   ← WiX v3
#   install/linux/   — final Linux installer    (.flatpak)      ← flatpak-builder
#   install/mac/     — final macOS disk image   (.dmg)          ← create-dmg
#
#   src/installers/windows/ — WiX fragment source  (dropshot.wxs)
#   src/installers/linux/   — Flatpak manifest     (com.dropshot.app.yml)
#   src/installers/mac/     — create-dmg script    (create-dmg.sh)
#
# Target overview
#   make              — detect OS → run build-<os>   (default)
#   make deps         — install JS dependencies
#   make dev          — Tauri hot-reload dev server
#   make build-<os>   — compile app → bin/<os>/
#   make install      — detect OS → run install-<os>
#   make install-<os> — package installer → install/<os>/  (depends on build-<os>)
#   make clean        — clear bin/*/ and install/*/ contents; cargo clean
# ─────────────────────────────────────────────────────────────────────────────

# ── Output directories (preserved by clean — contents only are deleted) ───────
BIN_WINDOWS  = bin/windows
BIN_LINUX    = bin/linux
BIN_MAC      = bin/mac

INST_WINDOWS = install/windows
INST_LINUX   = install/linux
INST_MAC     = install/mac

# Tauri bundle output roots per target triple
BUNDLE_WINDOWS = src-tauri/target/x86_64-pc-windows-msvc/release/bundle
BUNDLE_LINUX   = src-tauri/target/x86_64-unknown-linux-gnu/release/bundle
BUNDLE_MAC     = src-tauri/target/aarch64-apple-darwin/release/bundle

# Installer source config directories
INSTALLER_WIN = src/installers/windows
INSTALLER_LIN = src/installers/linux
INSTALLER_MAC = src/installers/mac

# Sentinel files — stamped after a successful build so install-<os> targets
# can declare a proper file prerequisite instead of always re-running.
BUILT_WINDOWS = $(BIN_WINDOWS)/.built
BUILT_LINUX   = $(BIN_LINUX)/.built
BUILT_MAC     = $(BIN_MAC)/.built

# ── Package manager detection ─────────────────────────────────────────────────
# Use bun if it is already on PATH; otherwise fall back to npm.
# Do NOT install bun — if it's absent, npm is used silently.
BUN_EXISTS := $(shell bun --version 2>/dev/null)
ifdef BUN_EXISTS
  PKG = bun
  PKG_RUN = bun run
else
  PKG = npm
  PKG_RUN = npm run
endif

# ── OS detection ──────────────────────────────────────────────────────────────
# On Windows, Make (via MSYS2/MinGW) sets OS=Windows_NT in the environment.
# On Linux/macOS, we fall through to uname.
ifeq ($(OS),Windows_NT)
  DETECTED_OS = windows
else
  UNAME := $(shell uname -s)
  ifeq ($(UNAME),Darwin)
    DETECTED_OS = mac
  else ifeq ($(UNAME),Linux)
    DETECTED_OS = linux
  else
    $(error Unsupported OS: $(UNAME))
  endif
endif

# ── Default target ────────────────────────────────────────────────────────────
.PHONY: all
all:
	@echo "Detected OS: $(DETECTED_OS)"
	@$(MAKE) build-$(DETECTED_OS)

# ── JS / Node dependencies ────────────────────────────────────────────────────
# Renamed from 'install' to 'deps' so that 'install' can be the
# platform-installer dispatcher (see below).
.PHONY: deps
deps:
	@echo "Installing JS dependencies with $(PKG)..."
	$(PKG) install

# ── Dev server ────────────────────────────────────────────────────────────────
.PHONY: dev
dev:
	$(PKG_RUN) tauri dev

# ── Platform builds ───────────────────────────────────────────────────────────
# Each build target compiles the app, copies the raw bundle to bin/<os>/,
# then stamps a sentinel file so install-<os> can declare it as a prerequisite.

# Windows: compiles with MSVC; copies .msi and NSIS .exe to bin/windows/
.PHONY: build-windows
build-windows:
	@echo "Building DropShot for Windows..."
	$(PKG_RUN) tauri build --target x86_64-pc-windows-msvc
	@echo "Copying artifacts to $(BIN_WINDOWS)/ ..."
	cp -r $(BUNDLE_WINDOWS)/msi/*.msi   $(BIN_WINDOWS)/ 2>/dev/null || true
	cp -r $(BUNDLE_WINDOWS)/nsis/*.exe  $(BIN_WINDOWS)/ 2>/dev/null || true
	@touch $(BUILT_WINDOWS)
	@echo "Done → $(BIN_WINDOWS)/"

# Linux: compiles for GNU/Linux; copies .AppImage and .deb to bin/linux/
# Also copies the raw binary for use by flatpak-builder in install-linux.
.PHONY: build-linux
build-linux:
	@echo "Building DropShot for Linux..."
	$(PKG_RUN) tauri build --target x86_64-unknown-linux-gnu
	@echo "Copying artifacts to $(BIN_LINUX)/ ..."
	cp -r $(BUNDLE_LINUX)/appimage/*.AppImage $(BIN_LINUX)/ 2>/dev/null || true
	cp -r $(BUNDLE_LINUX)/deb/*.deb           $(BIN_LINUX)/ 2>/dev/null || true
	cp src-tauri/target/x86_64-unknown-linux-gnu/release/dropshot \
	   $(BIN_LINUX)/dropshot 2>/dev/null || true
	@touch $(BUILT_LINUX)
	@echo "Done → $(BIN_LINUX)/"

# macOS: compiles for Apple Silicon; copies .app bundle to bin/mac/
.PHONY: build-mac
build-mac:
	@echo "Building DropShot for macOS..."
	$(PKG_RUN) tauri build --target aarch64-apple-darwin
	@echo "Copying artifacts to $(BIN_MAC)/ ..."
	cp -r $(BUNDLE_MAC)/macos/DropShot.app $(BIN_MAC)/ 2>/dev/null || true
	@touch $(BUILT_MAC)
	@echo "Done → $(BIN_MAC)/"

# ── Installer packaging ───────────────────────────────────────────────────────
# Each install-<os> target depends on the sentinel file stamped by build-<os>.
# Running install-<os> without a prior build-<os> will trigger the build first.

# install   — auto-detect OS and run the appropriate install target
.PHONY: install
install:
	@echo "Detected OS: $(DETECTED_OS)"
	@$(MAKE) install-$(DETECTED_OS)

# install-windows — package WiX .msi installer → install/windows/
# Requires: WiX Toolset v3 installed and on PATH (light, candle).
# The custom WiX fragment from src/installers/windows/dropshot.wxs is copied
# into src-tauri/wix/ so Tauri merges it during the next build.
# For the initial packaging step, Tauri already produced the .msi in bin/windows/.
.PHONY: install-windows
install-windows: $(BUILT_WINDOWS)
	@echo "Packaging Windows installer → $(INST_WINDOWS)/ ..."
	@mkdir -p $(INST_WINDOWS)
	@mkdir -p src-tauri/wix
	cp -n $(INSTALLER_WIN)/dropshot.wxs src-tauri/wix/dropshot.wxs 2>/dev/null || true
	cp $(BIN_WINDOWS)/*.msi $(INST_WINDOWS)/ 2>/dev/null || true
	cp $(BIN_WINDOWS)/*.exe $(INST_WINDOWS)/ 2>/dev/null || true
	@echo "Done → $(INST_WINDOWS)/"

# install-linux — build Flatpak bundle → install/linux/dropshot.flatpak
# Requires: flatpak-builder, and the freedesktop runtimes installed:
#   flatpak install flathub org.freedesktop.Platform//23.08
#   flatpak install flathub org.freedesktop.Sdk//23.08
.PHONY: install-linux
install-linux: $(BUILT_LINUX)
	@echo "Building Flatpak package → $(INST_LINUX)/dropshot.flatpak ..."
	@mkdir -p $(INST_LINUX)
	flatpak-builder --force-clean \
	    --repo=$(INST_LINUX)/repo \
	    $(INST_LINUX)/build \
	    $(INSTALLER_LIN)/com.dropshot.app.yml
	flatpak build-bundle $(INST_LINUX)/repo \
	    $(INST_LINUX)/dropshot.flatpak \
	    com.dropshot.app
	@echo "Done → $(INST_LINUX)/dropshot.flatpak"

# install-mac — build polished DMG → install/mac/DropShot.dmg
# Requires: create-dmg  (brew install create-dmg)
.PHONY: install-mac
install-mac: $(BUILT_MAC)
	@echo "Building macOS DMG → $(INST_MAC)/DropShot.dmg ..."
	@mkdir -p $(INST_MAC)
	bash $(INSTALLER_MAC)/create-dmg.sh
	@echo "Done → $(INST_MAC)/DropShot.dmg"

# ── Cross-compile helpers (run from any host with the target toolchain) ───────
.PHONY: build-windows-cross
build-windows-cross:
	@echo "Cross-compiling for Windows (requires mingw-w64 or cargo-xwin)..."
	$(PKG_RUN) tauri build --target x86_64-pc-windows-msvc
	cp -r $(BUNDLE_WINDOWS)/msi/*.msi  $(BIN_WINDOWS)/ 2>/dev/null || true
	cp -r $(BUNDLE_WINDOWS)/nsis/*.exe $(BIN_WINDOWS)/ 2>/dev/null || true
	@touch $(BUILT_WINDOWS)

.PHONY: build-linux-cross
build-linux-cross:
	@echo "Cross-compiling for Linux (requires linux-musl toolchain)..."
	$(PKG_RUN) tauri build --target x86_64-unknown-linux-gnu
	cp -r $(BUNDLE_LINUX)/appimage/*.AppImage $(BIN_LINUX)/ 2>/dev/null || true
	cp -r $(BUNDLE_LINUX)/deb/*.deb           $(BIN_LINUX)/ 2>/dev/null || true
	cp src-tauri/target/x86_64-unknown-linux-gnu/release/dropshot \
	   $(BIN_LINUX)/dropshot 2>/dev/null || true
	@touch $(BUILT_LINUX)

# ── Clean ─────────────────────────────────────────────────────────────────────
# Deletes the CONTENTS of bin/<os>/ and install/<os>/ subdirectories but
# keeps the directories themselves, preserving the repo folder structure.
.PHONY: clean
clean:
	@echo "Cleaning Rust build artifacts..."
	cargo clean --manifest-path src-tauri/Cargo.toml
	rm -rf dist
	@echo "  Clearing bin/windows/ ..."
	find bin/windows   -mindepth 1 -delete 2>/dev/null || true
	@echo "  Clearing bin/linux/ ..."
	find bin/linux     -mindepth 1 -delete 2>/dev/null || true
	@echo "  Clearing bin/mac/ ..."
	find bin/mac       -mindepth 1 -delete 2>/dev/null || true
	@echo "  Clearing install/windows/ ..."
	find install/windows -mindepth 1 -delete 2>/dev/null || true
	@echo "  Clearing install/linux/ ..."
	find install/linux   -mindepth 1 -delete 2>/dev/null || true
	@echo "  Clearing install/mac/ ..."
	find install/mac     -mindepth 1 -delete 2>/dev/null || true
	@echo "Done."

# ── Help ──────────────────────────────────────────────────────────────────────
.PHONY: help
help:
	@echo ""
	@echo "DropShot make targets:"
	@echo ""
	@echo "  Setup"
	@echo "    make deps              — install JS dependencies (bun or npm)"
	@echo "    make dev               — start hot-reload dev server"
	@echo ""
	@echo "  Build  (compile app → bin/<os>/)"
	@echo "    make                   — auto-detect OS and build"
	@echo "    make build-windows     — compile → bin/windows/"
	@echo "    make build-linux       — compile → bin/linux/"
	@echo "    make build-mac         — compile → bin/mac/"
	@echo ""
	@echo "  Install  (package installer → install/<os>/)"
	@echo "    make install           — auto-detect OS and package installer"
	@echo "    make install-windows   — WiX .msi/.exe  → install/windows/"
	@echo "    make install-linux     — Flatpak .flatpak → install/linux/"
	@echo "    make install-mac       — DMG via create-dmg → install/mac/"
	@echo ""
	@echo "  Maintenance"
	@echo "    make clean             — clear bin/*/, install/*/, dist/; cargo clean"
	@echo "    make help              — show this message"
	@echo ""
	@echo "  Package manager: $(PKG)"
	@echo "  Detected OS:     $(DETECTED_OS)"
	@echo ""
