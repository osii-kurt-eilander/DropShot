# DropShot

<p align="center">
  <img src="assets/logo.svg" alt="DropShot logo" width="120" />
</p>

A cross-platform desktop app inspired by the tennis drop shot — quick, precise, and always landing exactly where you want it. Press a hotkey and an app-launcher panel drops down from the top of your screen. Click a shortcut icon to launch your favorite application, then press the hotkey again (or click away) to snap it back up.

Built with [Tauri v2](https://tauri.app/) (Rust backend) and [Preact](https://preactjs.com/) (frontend). Runs on **Windows 10/11**, **Linux (Wayland)**, and **macOS**.

---

## Features

- **Global hotkey** (default: `` ` `` backtick) toggles the panel from anywhere on the desktop
- **Borderless, transparent panel** docked to the top of your primary monitor
- **Always on top** — never buried under other windows
- **Automatic dark / light mode** — follows your OS theme setting instantly
- **Customizable shortcuts** — edit a simple JSON file to add, remove, or reorder apps
- **Smooth slide animation** with a frosted-glass appearance

---

## Prerequisites

### All platforms

#### Rust & Cargo
Tauri requires a Rust toolchain (1.77 or later).

```bash
# Install rustup (manages Rust versions)
# https://rustup.rs
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # Linux / macOS
# On Windows: download and run https://win.rustup.rs

# Verify
rustc --version
cargo --version
```

#### Node.js (LTS)
Required to build the Preact frontend. Download the LTS release from [https://nodejs.org](https://nodejs.org) or use a version manager.

```bash
node --version   # should be 20.x or later
npm --version
```

#### Bun *(optional — faster builds)*
If bun is present on `PATH`, all `make` and build commands use it automatically. If it is not installed, npm is used silently — **do not install bun just to satisfy this project**.

```bash
# Install bun if you want it
curl -fsSL https://bun.sh/install | bash          # Linux / macOS
powershell -c "irm bun.sh/install.ps1 | iex"      # Windows (PowerShell)

bun --version   # verify
```

---

### Windows

| Requirement | Why | How to install |
|---|---|---|
| **MSVC Build Tools** (C++ workload) | Rust on Windows requires the MSVC linker | Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/); select **"Desktop development with C++"** |
| **WebView2 Runtime** | Tauri renders the UI via WebView2 | Pre-installed on Windows 11 and updated Windows 10. If missing: [download here](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |
| **GNU Make** | Required for `make` commands | Included with [MSYS2](https://www.msys2.org/) (`pacman -S make`) or [Git for Windows](https://gitforwindows.org/) (Git Bash includes make) |

Add the Rust Windows target if not already present:
```bash
rustup target add x86_64-pc-windows-msvc
```

#### Installer tool — WiX Toolset v3 *(for `make install-windows`)*
WiX is integrated into Tauri's bundler — no separate installation is required. The custom WiX fragment at `src/installers/windows/dropshot.wxs` is automatically merged during packaging.

---

### Linux (Wayland)

Install system libraries that Tauri and WebKitGTK depend on.

**Debian / Ubuntu:**
```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf \
  build-essential \
  curl wget file \
  libxdo-dev \
  libssl-dev \
  libsoup-3.0-dev \
  javascriptcoregtk-4.1
```

**Arch / Manjaro:**
```bash
sudo pacman -S --needed \
  webkit2gtk-4.1 gtk3 \
  libayatana-appindicator librsvg \
  base-devel curl wget file openssl libxdo
```

Add the Rust Linux target:
```bash
rustup target add x86_64-unknown-linux-gnu
```

#### Installer tool — Flatpak *(for `make install-linux`)*
Flatpak produces a sandboxed, distro-agnostic `.flatpak` bundle — the recommended distribution format for Wayland apps.

```bash
# Install flatpak-builder
sudo apt install flatpak-builder          # Debian/Ubuntu
sudo pacman -S flatpak-builder            # Arch/Manjaro

# Install the required runtime and SDK (one-time)
flatpak install flathub org.freedesktop.Platform//23.08
flatpak install flathub org.freedesktop.Sdk//23.08
```

> **Wayland note:** Transparency and always-on-top depend on the compositor. Both work on KDE/KWin and GNOME/Mutter (GNOME requires the [AppIndicator extension](https://extensions.gnome.org/extension/615/appindicator-support/)). On compositors that deny these features, DropShot degrades gracefully to an opaque panel.

---

### macOS

Install [Xcode Command Line Tools](https://developer.apple.com/xcode/resources/) (provides clang, linker, and system SDKs):

```bash
xcode-select --install
```

Add the Rust macOS target matching your Mac's architecture:

```bash
# Apple Silicon (M1/M2/M3/M4)
rustup target add aarch64-apple-darwin

# Intel Mac
rustup target add x86_64-apple-darwin
```

No additional system libraries are required — macOS provides WebKit natively.

#### Installer tool — create-dmg *(for `make install-mac`)*
`create-dmg` wraps Tauri's `.app` bundle in a polished DMG with a custom background and drag-to-Applications layout.

```bash
brew install create-dmg
```

---

## Installation

```bash
# 1. Clone the repository
git clone https://github.com/your-username/dropshot.git
cd dropshot

# 2. Install JavaScript dependencies
make deps             # uses bun if available, otherwise npm

# 3. Rust dependencies are fetched automatically on first build
```

---

## Running in Development

```bash
make dev
```

This starts the Vite dev server with hot-reload and opens the Tauri window. Frontend changes appear instantly; Rust changes trigger a recompile.

Equivalent direct command: `bun run tauri dev` (or `npm run tauri dev`)

---

## Building and Packaging

DropShot has two distinct steps:

| Step | Command | Output |
|---|---|---|
| **Build** — compile the app | `make` or `make build-<os>` | `bin/<os>/` |
| **Package** — create installer | `make install` or `make install-<os>` | `install/<os>/` |

`make install-<os>` automatically runs `make build-<os>` first if the build output is missing or stale, so you can run just `make install` for a full build + package in one step.

### Auto-detect (recommended)
```bash
make          # compile for current OS → bin/<os>/
make install  # compile + package installer → install/<os>/
```

### Windows
```bash
make build-windows
# → bin/windows/  (.msi, .exe via WiX v3 + NSIS, produced by Tauri)

make install-windows
# → install/windows/  (copies the WiX .msi and NSIS .exe; merges custom
#                       WiX fragment from src/installers/windows/dropshot.wxs)
```

### Linux
```bash
make build-linux
# → bin/linux/  (.AppImage, .deb, raw binary)

make install-linux
# → install/linux/dropshot.flatpak
#   (built with flatpak-builder using src/installers/linux/com.dropshot.app.yml)
#   Requires: flatpak-builder + freedesktop SDK 23.08 (see Prerequisites)
```

### macOS
```bash
make build-mac
# → bin/mac/DropShot.app

make install-mac
# → install/mac/DropShot.dmg
#   (built with create-dmg using src/installers/mac/create-dmg.sh)
#   Requires: brew install create-dmg
```

### All make targets
```
make              — auto-detect OS → build
make deps         — install JS dependencies
make dev          — start dev server with hot-reload
make build-windows
make build-linux
make build-mac
make install      — auto-detect OS → build + package installer
make install-windows  — WiX .msi/.exe  → install/windows/
make install-linux    — Flatpak bundle → install/linux/
make install-mac      — DMG via create-dmg → install/mac/
make clean        — clear bin/*/, install/*/, dist/; cargo clean
make help         — print all targets and detected environment
```

---

## Customizing Your Shortcuts

Edit `data/dropshot.json` in the project root. This file is the single source of configuration — it controls both the layout and the list of shortcuts.

```json
{
  "version": 1,
  "layout": {
    "columns": 8,
    "tileSize": 72,
    "gap": 12,
    "padding": 24
  },
  "shortcuts": [
    { "name": "VS Code",  "path": "...", "icon": "vscode.png" },
    { "name": "Chrome",   "path": "...", "icon": "chrome.png" }
  ]
}
```

Use paths appropriate for your OS:

**Windows** (`path` values):
```
C:\\Users\\you\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe
C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe
C:\\Windows\\System32\\wt.exe
```

**Linux** (`path` values):
```
/usr/bin/code
/usr/bin/firefox
/usr/bin/kitty
```

**macOS** (`path` values):
```
/Applications/Visual Studio Code.app
/Applications/Google Chrome.app
/Applications/iTerm.app
```

- **`name`** — label shown below the icon
- **`path`** — full path to the executable or `.app` bundle
- **`icon`** — filename of a PNG placed in `assets/icons/` (64×64 px minimum, transparent background)

### Where the data file lives

| Mode | Location |
|---|---|
| **Development** | `data/dropshot.json` in this repo — edit directly |
| **Windows** (installed) | `%APPDATA%\dropshot\dropshot.json` |
| **Linux** (installed) | `~/.config/dropshot/dropshot.json` |
| **macOS** (installed) | `~/Library/Application Support/dropshot/dropshot.json` |

On the first launch after installation, the default `data/dropshot.json` is automatically copied to the appropriate OS location. Subsequent launches read and write from that OS location; the repo's `data/` file is not used again.

---

## Changing the Hotkey

The default toggle hotkey is the backtick key (`` ` ``). To change it, open `src-tauri/src/lib.rs` and update the shortcut string passed to `app.global_shortcut().register(...)`. Hotkey strings follow the format `"Ctrl+Shift+Space"`, `"Alt+F1"`, etc.

---

## Project Structure

```
assets/
  logo.svg               # App logo
  icons/                 # Shortcut tile PNG icons (place yours here)

data/                    # Default config — committed to repo
  dropshot.json          # Edit this to configure layout and shortcuts

bin/                     # Compiled app output (do not commit)
  windows/               #   Raw Tauri bundle (.msi, .exe)
  linux/                 #   Raw Tauri bundle (.AppImage, .deb, binary)
  mac/                   #   Raw .app bundle

install/                 # Packaged installers (do not commit)
  windows/               #   WiX .msi and NSIS .exe
  linux/                 #   Flatpak bundle (.flatpak)
  mac/                   #   DMG disk image (.dmg)

docs/
  graphic_design.md      # Design system reference (fonts, colors, animation)

src/                     # Preact frontend
  apps/
    main.tsx             # Mount point; OS theme detection
    App.tsx              # Root component; slide animation & panel state
    app.css              # All styles and design tokens
  components/
    ShortcutGrid.tsx     # Grid of shortcut tiles
    ShortcutTile.tsx     # Single tile: icon + label + launch on click
  installers/            # Installer configuration (source, not output)
    windows/
      dropshot.wxs       #   Custom WiX fragment (branding, shortcuts, registry)
    linux/
      com.dropshot.app.yml  # Flatpak manifest
    mac/
      create-dmg.sh      #   DMG packaging script
      dmg-background.png #   660×400 DMG window background image
  utils/
    icons.ts             # Icon filename → URL resolver

src-tauri/               # Rust / Tauri backend
  src/
    main.rs              # App entry point
    lib.rs               # Tauri commands + hotkey setup + window positioning
    shortcuts.rs         # shortcuts.json loading and deserialization
  tauri.conf.json        # Window config (borderless, transparent, always-on-top)
  Cargo.toml

shortcuts.json           # Your shortcut definitions (edit this)
Makefile                 # Cross-platform build and installer targets
```

---

## License

MIT
