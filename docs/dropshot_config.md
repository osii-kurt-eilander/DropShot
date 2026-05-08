# DropShot — `dropshot.json` Configuration Reference

This document is the **single source of truth** for all UI layout and feature
settings in DropShot.  Every field in `dropshot.json` is described here with
its type, default, and effect on each supported platform.

> **File locations**
>
> | Mode | Path |
> |---|---|
> | Dev (`debug_assertions`) | `<workspace>/data/dropshot.json` |
> | Windows | `%APPDATA%\dropshot\dropshot.json` |
> | Linux | `$XDG_CONFIG_HOME/dropshot/dropshot.json` (default: `~/.config/dropshot/`) |
> | macOS | `~/Library/Application Support/dropshot/dropshot.json` |
>
> In production the file is seeded from the bundled default on first launch.
> In dev mode edit `data/dropshot.json` and press **F5** in the panel to reload.

---

## Top-level structure

```jsonc
{
  "version": "1.1",       // Schema version string — do not change manually
  "application": { … },   // Panel appearance and behaviour
  "effects": { … },       // Pointer-event effects applied to all tiles
  "shortcuts": [ … ]      // Array of shortcut tile definitions
}
```

---

## `application`

Controls the panel's visual appearance and OS-level behaviour.

### `menuSize`

| Key | Type | Default | Description |
|---|---|---|---|
| `width` | `number` | `1920` | Panel width in CSS logical pixels. Clamped to the monitor width by the Rust setup code. |
| `height` | `number` | `120` | Panel height in CSS logical pixels. Sets both the native window height and the CSS panel height. |

> **Platform note:** On HiDPI / Retina displays the Rust backend receives the
> value as a `PhysicalSize`.  The frontend uses `LogicalSize` so the JSON value
> always represents CSS pixels regardless of device-pixel-ratio.

---

### `openingAnimation` / `closingAnimation`

| Key | Type | Default (opening) | Default (closing) | Description |
|---|---|---|---|---|
| `type` | `string` | `"slideDown"` | `"slideUp"` | Animation style — see table below. |
| `duration` | `number` | `220` | `220` | Transition duration in milliseconds. |

**Recognised `type` values:**

| Value | Effect |
|---|---|
| `"slideDown"` | Panel slides in from above (translateY -100% → 0) |
| `"slideUp"` | Panel slides out upward (translateY 0 → -100%) |
| `"slide"` | Alias for `"slideDown"` (opening) / `"slideUp"` (closing) |
| `"fadeIn"` | Panel fades in (opacity 0 → 1); stays at y=0 |
| `"fadeOut"` | Panel fades out (opacity 1 → 0); stays at y=0 |
| `"fade"` | Alias for `"fadeIn"` (opening) / `"fadeOut"` (closing) |

> Unknown values fall back to the slide behaviour.
> The `openingAnimation.type` also determines the CSS `data-anim` attribute
> applied to the panel div, which selects the CSS transition ruleset.

---

### `opacity`

| Type | Default | Range |
|---|---|---|
| `number` | `0.95` | `0.0` – `1.0` |

Applied as CSS `opacity` to the entire panel element.  Affects both the
background and all child tile elements uniformly.

> **Platform note:** Transparency requires compositor support.  On Linux/Wayland
> the compositor may ignore transparency; the panel degrades gracefully to
> fully opaque.

---

### `backgroundColor`

| Type | Default |
|---|---|
| `string` | `"#1a1d2e"` |

A CSS colour value (`"#rrggbb"`, `"rgba(…)"`, `"hsl(…)"`, etc.) applied as
the panel's `background-color`.  This overrides the default glassmorphism
gradient defined in `app.css`.

> To keep the glassmorphism gradient, omit this field or set it to `"transparent"`.

---

### `extraCSS`

| Type | Default |
|---|---|
| `string` (optional) | absent |

Arbitrary CSS declarations applied directly to the panel element's inline
style.  Semicolon-separated property:value pairs, e.g.:

```json
"extraCSS": "box-shadow: 0 0 10px rgba(255,255,255,0.5); backdrop-filter: blur(20px);"
```

> **Implementation detail:** Properties are applied individually via
> `element.style[camelCase] = value`, not via a `<style>` tag, so they
> affect only the panel element and cannot leak to other parts of the page.

---

### `border`

| Key | Type | Default | Description |
|---|---|---|---|
| `color` | `string` (optional) | absent | CSS colour string for the border. |
| `width` | `number` (optional) | `0` | Border width in pixels.  `0` → no border. |
| `radius` | `number` (optional) | `0` | Border-radius in pixels (rounds the panel's corners). |

Example:
```json
"border": { "color": "#ffffff", "width": 2, "radius": 10 }
```

---

### `handleThickness`

| Type | Default |
|---|---|
| `number` | `4` |

Height in pixels of the drag-handle strip rendered at the bottom edge of the
panel.  Set to `0` to hide the handle entirely.

The handle element carries `data-tauri-drag-region`, enabling the user to
reposition the window by dragging it (useful when `menuSize.width` is less
than the monitor width).

---

### `systemHotkey`

| Type | Default |
|---|---|
| `string` | `"Backquote"` |

The global keyboard shortcut that toggles the panel open / closed.  Uses
[`tauri-plugin-global-shortcut`](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/global-shortcut)
Accelerator syntax:

| Format | Example |
|---|---|
| Single key | `"Backquote"`, `"F12"`, `"Space"` |
| With modifiers | `"Ctrl+Shift+Space"`, `"Alt+F12"`, `"CmdOrCtrl+Backquote"` |

> **Platform notes:**
> - Windows / Linux: `Ctrl` = Control key.
> - macOS: use `Cmd` or `CmdOrCtrl` for the Command key.
> - The hotkey is registered at startup.  If it conflicts with an OS-reserved
>   shortcut, a warning is printed to the log and the app continues without
>   a working hotkey.
> - Press **F5** while the panel is visible to reload the config and
>   re-register a changed hotkey without restarting the app.

---

### `position`

| Type | Default |
|---|---|
| `string` | `"left"` |

Horizontal anchor of the panel on screen.  Only has a visible effect when
`menuSize.width` is less than the monitor's physical pixel width.

| Value | Panel position |
|---|---|
| `"left"` | Flush with the left edge of the monitor |
| `"center-left"` | 25 % from the left |
| `"center"` | Horizontally centred |
| `"center-right"` | 75 % from the left |
| `"right"` | Flush with the right edge of the monitor |

The Rust `panel_x()` helper in `lib.rs` calculates the physical window X
coordinate from this value and the monitor width at startup.

---

## `effects`

Pointer-event effects applied globally to every shortcut tile.  All sub-keys
are optional; absent events receive a silent no-op.

```jsonc
"effects": {
  "mouseover": { … },   // Pointer enters a tile
  "mouseout":  { … },   // Pointer leaves a tile
  "click":     { … }    // Tile is clicked
}
```

### Effect object

All fields are optional and may appear on any event type.

| Key | Type | Description |
|---|---|---|
| `scale` | `number` | CSS `transform: scale(N)` factor.  `1.0` = no change, `1.2` = 20 % larger. |
| `duration` | `number` | CSS transition duration in milliseconds for both `transform` and any `extraCSS` properties. |
| `extraCSS` | `string` | Arbitrary CSS declarations applied while the effect is active (semicolon-separated). Properties written by `mouseover.extraCSS` are individually reset on `mouseout` before the `mouseout` effect is applied. |

Example:
```json
"effects": {
  "mouseover": { "scale": 1.2, "duration": 200, "extraCSS": "background-color: yellow;" },
  "mouseout":  { "scale": 1.0, "duration": 200 },
  "click":     { "scale": 0.9, "duration": 100 }
}
```

> **Note:** Because all fields are optional and may appear on any sibling,
> you could, for example, add `"extraCSS"` to `"mouseout"` or `"click"` in a
> future config and the application will apply it correctly.

---

## `shortcuts`

An array of shortcut tile definitions.  Each tile is positioned absolutely
within the panel using its own `x`/`y`/`width`/`height` values.

```jsonc
{
  "name":        "VS Code",
  "path":        "Code.exe",
  "icon":        "vscode.svg",
  "x":           0,
  "y":           0,
  "width":       100,
  "height":      100,
  "extraCss":    "border: 2px solid #007ACC;",   // optional
  "tooltip":     "Open VS Code",                 // optional
  "params":      ["--new-window"],               // optional
  "environment": { "MY_VAR": "value" }           // optional
}
```

### Required fields

| Key | Type | Description |
|---|---|---|
| `name` | `string` | Display label shown below the icon. |
| `path` | `string` | Executable to launch — see platform table below. |
| `icon` | `string` | Icon filename (with extension) relative to `assets/icons/`. Supports `.svg`, `.png`, etc. |
| `x` | `number` | Left edge of the tile in CSS pixels, relative to the panel's left edge. |
| `y` | `number` | Top edge of the tile in CSS pixels, relative to the panel's top edge. |
| `width` | `number` | Tile width in CSS pixels. |
| `height` | `number` | Tile height in CSS pixels. |

### Optional fields

| Key | Type | Description |
|---|---|---|
| `extraCss` | `string` | Arbitrary CSS declarations applied to this tile's `<button>` element only (semicolon-separated). Applied after mount via a DOM ref so positional styles are never overwritten. |
| `tooltip` | `string` | Tooltip text on hover. Defaults to `name` when absent. |
| `params` | `string[]` | CLI arguments forwarded verbatim to the launched process. |
| `environment` | `object` | Key-value pairs injected as environment variables into the launched process. |

### `path` — platform notes

| Platform | Accepted values |
|---|---|
| **Windows** | Absolute path (`C:\…\Code.exe`) or bare name resolvable via `PATH` (`Code.exe`). |
| **macOS** | Absolute path, `.app` bundle path (`/Applications/Visual Studio Code.app`), or bare app name for `open -a` (`"Visual Studio Code"`). |
| **Linux** | Absolute path, `PATH`-resolvable binary name, or a `.desktop` file name (e.g. `code.desktop`) — the last form is passed to `xdg-open`; others are exec'd directly. Only binaries exec'd directly receive `params` and `environment`. |

---

## Full example

```json
{
  "version": "1.1",
  "application": {
    "menuSize": { "width": 1920, "height": 120 },
    "openingAnimation": { "type": "slideDown", "duration": 300 },
    "closingAnimation": { "type": "slideUp",   "duration": 300 },
    "opacity": 0.9,
    "backgroundColor": "#000000",
    "extraCSS": "box-shadow: 0 0 10px rgba(255,255,255,0.5);",
    "border": { "color": "#ffffff", "width": 2, "radius": 10 },
    "handleThickness": 5,
    "systemHotkey": "Ctrl+Shift+Space",
    "position": "center-right"
  },
  "effects": {
    "mouseover": { "scale": 1.2, "duration": 200, "extraCSS": "background-color: yellow;" },
    "mouseout":  { "scale": 1.0, "duration": 200 },
    "click":     { "scale": 0.9, "duration": 100 }
  },
  "shortcuts": [
    {
      "name": "VS Code",
      "path": "Code.exe",
      "icon": "vscode.svg",
      "x": 0, "y": 0, "width": 100, "height": 100,
      "extraCss": "border: 2px solid #007ACC; border-radius: 10px;"
    },
    {
      "name": "Chrome",
      "path": "chrome.exe",
      "icon": "chrome.svg",
      "x": 110, "y": 0, "width": 100, "height": 100,
      "tooltip": "Open Chrome"
    },
    {
      "name": "Terminal",
      "path": "wt.exe",
      "params": ["--fullscreen"],
      "icon": "terminal.svg",
      "x": 220, "y": 0, "width": 100, "height": 100
    },
    {
      "name": "Explorer",
      "path": "explorer.exe",
      "icon": "explorer.svg",
      "x": 330, "y": 0, "width": 100, "height": 100
    }
  ]
}
```
