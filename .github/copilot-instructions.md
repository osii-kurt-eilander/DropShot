# Copilot Instructions — DropShot (Tauri + Preact)

## ⚠️ Single Source of Truth for UI Layout and Features

**`docs/dropshot_config.md` is the authoritative reference for every
`dropshot.json` setting.**  Before implementing or modifying any UI feature,
animation, effect, shortcut behaviour, hotkey, or panel property, read that
document first.  All field names, types, defaults, and platform-specific
behaviours are defined there.  Do **not** invent field names or behaviours
that are not described in `docs/dropshot_config.md`.

## Project Overview
A **cross-platform** desktop application built with **Tauri v2** (Rust backend) and **Preact** (frontend) called **DropShot** — named for both the tennis drop shot and the panel that drops from the top of the screen. It behaves like a Quake-style "window shade": a panel that slides down from the top of the screen when a global hotkey is pressed, displays shortcut icons for launching favorite applications, then slides back up when dismissed.

## Cross-Platform Targets
DropShot must run correctly on all three platforms — do not write platform-only code without a fallback:
- **Windows 10/11** — primary target; use `tauri::PhysicalPosition` for monitor anchoring.
- **Linux (Wayland)** — use `tauri-plugin-global-shortcut`; note that true always-on-top and transparency depend on the compositor. Test on GNOME/Mutter and KDE/KWin. App launching uses `Command::new("xdg-open")` or direct binary paths.
- **macOS** — panel anchors to top of primary screen; use `NSWindowLevel` via Tauri's `set_window_level` for always-on-top. App launching uses `Command::new("open").args(["-a", ...])` or direct binary paths.

Conditional platform logic in Rust should use `#[cfg(target_os = "windows")]`, `#[cfg(target_os = "linux")]`, and `#[cfg(target_os = "macos")]`.

## Architecture

```
.github/
  copilot-instructions.md   # This file

assets/
  logo.svg                  # Master app logo (scalable SVG)

bin/                        # Compiled installer outputs — do not commit
  windows/                  #   Raw Tauri bundle (.msi, .exe)
  linux/                    #   Raw Tauri bundle (.AppImage, .deb) + binary
  mac/                      #   macOS .app bundle

data/                       # Default / seed user data files (committed to repo)
  dropshot.json             #   Default layout and shortcuts config (empty shortcuts list)

install/                    # Final packaged installers — do not commit
  windows/                  #   WiX .msi and NSIS .exe  (output of make install-windows)
  linux/                    #   Flatpak .flatpak bundle  (output of make install-linux)
  mac/                      #   DMG disk image           (output of make install-mac)

docs/
  graphic_design.md         # Fonts, colors, animation specs — must be followed for all UI work

src/                        # All application source code
  apps/                     # Top-level Preact app entry points
    main.tsx                #   Mount point; sets up dark/light mode detection
    App.tsx                 #   Root component; owns slide-open/close animation state
  components/               # Shared Preact UI components
    ShortcutGrid.tsx        #   Grid layout of shortcut tiles
    ShortcutTile.tsx        #   Single tile: icon + label + click-to-launch
  installers/               # Platform-specific installer configuration (source, not output)
    windows/                #   dropshot.wxs — custom WiX v3 fragment (branding, shortcuts, registry, data seeding)
    linux/                  #   com.dropshot.app.yml — Flatpak manifest (flatpak-builder)
    mac/                    #   create-dmg.sh — DMG packaging script (create-dmg)
  test/                     # Frontend and integration tests
  utils/                    # Shared TypeScript utilities (path helpers, icon resolution, etc.)

src-tauri/                  # Rust/Tauri backend (to be created during project init)
  src/
    main.rs                 #   App entry, window setup, global hotkey registration
    lib.rs                  #   Tauri commands exposed to frontend
    shortcuts.rs            #   Config loading: dev reads data/, production reads OS app data dir
  tauri.conf.json           #   Window config: borderless, always-on-top, top-anchored
  Cargo.toml

shortcuts.json              # Legacy flat shortcut list (superseded by data/dropshot.json)
Makefile                    # Cross-platform build; outputs land in bin/<os>/
README.md
```

### Key directory rules
- **`data/`** — committed default config files. **Dev builds read from here directly.** Production builds seed from here on first launch but then read/write the OS app data directory.
- **`src/installers/<os>/`** — installer *configuration* (WiX `.wxs`, Flatpak YAML manifest, `create-dmg.sh`). Edit these to customise the installer for each platform.
- **`bin/<os>/`** — compiled app binary/bundle produced by `make build-<os>`. Stamped with a `.built` sentinel file so `make install-<os>` can declare it as a prerequisite. Never commit.

## Data File Locations

`dropshot.json` is the single source of configuration (layout + shortcuts).

| Mode | Path |
|---|---|
| **Dev** (`debug_assertions`) | `data/dropshot.json` — repo directory, hot-editable |
| **Windows** (production) | `%APPDATA%\dropshot\dropshot.json` |
| **Linux** (production) | `$XDG_CONFIG_HOME/dropshot/dropshot.json` (default: `~/.config/dropshot/`) |
| **macOS** (production) | `~/Library/Application Support/dropshot/dropshot.json` |

**First-run seeding:** If the production path does not exist, `shortcuts.rs::seed_default_config()` copies `data/dropshot.json` from the bundled Tauri resources automatically — no user action required.

**Installer seeding per platform:**
- **Windows (WiX):** The `UserDataComponent` in `dropshot.wxs` installs `dropshot.json` directly to `%APPDATA%\dropshot\` at MSI install time.
- **Linux (Flatpak):** The `dropshot-launch` wrapper script seeds `$XDG_CONFIG_HOME/dropshot/dropshot.json` from `/app/share/dropshot/data/` on the very first launch.
- **macOS (DMG):** `create-dmg.sh` embeds `data/` into `DropShot.app/Contents/Resources/data/`; `seed_default_config()` copies it to `~/Library/Application Support/dropshot/` on first launch.
- **`install/<os>/`** — final packaged installer produced by `make install-<os>`. Contents cleared by `make clean`; subdirectories preserved. Never commit.

## Key Tauri Window Behavior
- Window is **borderless, transparent, always-on-top**, docked to top of primary monitor.
- `tauri.conf.json` sets `"visible": false` on startup; the hotkey toggles visibility.
- Use `tauri-plugin-global-shortcut` for the toggle hotkey (default: `` ` `` backtick).
- Use `tauri-plugin-shell` to launch external applications via `Command::new`.
- Window slide animation is CSS-driven (`transform: translateY`); Tauri only toggles `show()`/`hide()` after animation completes.

## Shortcut Configuration (`shortcuts.json`)
```json
[
  { "name": "VS Code", "path": "C:\\...\\Code.exe", "icon": "vscode.png" },
  { "name": "Browser",  "path": "C:\\...\\chrome.exe", "icon": "chrome.png" }
]
```
- Icons are stored in `assets/icons/` and referenced by filename only.
- Rust deserializes this via `serde_json`; the frontend fetches it via a Tauri `invoke("get_shortcuts")` command.

## Critical Commands
Prefer `bun` over `npm` if it is already installed on the machine. Do **not** install bun for a developer who doesn't have it — fall back to `npm` in that case.

```bash
# Install JS dependencies  (NOTE: 'make deps', not 'make install')
make deps              # or: bun install / npm install

# Dev mode (hot-reload Preact + Tauri window)
bun run tauri dev  # or: npm run tauri dev

# Compile the app for the current OS → bin/<os>/
make

# Package the installer for the current OS → install/<os>/
# (automatically runs the build step first if needed)
make install

# Platform-specific build + install
make build-windows     &&   make install-windows   # → install/windows/
make build-linux       &&   make install-linux     # → install/linux/
make build-mac         &&   make install-mac       # → install/mac/

# Add a Tauri plugin (example)
cargo add tauri-plugin-global-shortcut
bun add @tauri-apps/plugin-global-shortcut  # or: npm install @tauri-apps/plugin-global-shortcut
```

## Tauri Command Pattern
Rust commands in `lib.rs` are exposed with `#[tauri::command]` and registered in `main.rs` via `.invoke_handler(tauri::generate_handler![...])`. Frontend calls them with:
```ts
import { invoke } from '@tauri-apps/api/core';
const shortcuts = await invoke<Shortcut[]>('get_shortcuts');
```

## Preact Conventions
- Use **Preact signals** (`@preact/signals`) for reactive state (e.g., `isOpen`, `shortcuts`).
- No React — import from `preact` and `preact/hooks`, never `react`.
- Components are `.tsx` files using functional style only.

## Design System
**All UI work must follow `docs/graphic_design.md` exactly.** This covers fonts, colors, panel glassmorphism, tile styling, and animation curves. Do not introduce colors, fonts, or spacing values not defined there.

### Dark / Light Mode
- The app **must** respect the OS-level dark/light mode setting automatically — no manual toggle in the UI.
- Detect via the CSS media query `prefers-color-scheme`; set `data-theme` on `<html>` accordingly:
```ts
const mq = window.matchMedia('(prefers-color-scheme: dark)');
document.documentElement.dataset.theme = mq.matches ? 'dark' : 'light';
mq.addEventListener('change', e => {
  document.documentElement.dataset.theme = e.matches ? 'dark' : 'light';
});
```
- All color values in components must use the CSS custom properties defined in `docs/graphic_design.md` (`--color-bg`, `--color-accent`, etc.) — never hardcoded hex values.

## Platform-Specific Window Notes
- **Windows:** `tauri::PhysicalPosition { x: 0, y: 0 }` to pin to top-left; full monitor width.
- **Linux (Wayland):** Transparency and always-on-top depend on compositor support; degrade gracefully (opaque fallback) if compositor denies the request.
- **macOS:** Use `set_window_level` for always-on-top; panel width = full screen width via `NSScreen::mainScreen` bounds.
