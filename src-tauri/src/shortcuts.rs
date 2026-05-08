use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::Manager;

// ── Data types ────────────────────────────────────────────────────────────────
//
// These structs mirror `dropshot.json` exactly.
//
// Design rules:
//   • Every field that *may* be absent in the JSON is `Option<T>`.
//   • Fields that always have a sensible default carry `#[serde(default)]`
//     so a partial or minimal config file still deserialises without error.
//   • `Default` is implemented on every type so `lib.rs::setup` can fall back
//     to safe built-in values when config loading fails at startup.

// ─── menuSize ────────────────────────────────────────────────────────────────

/// Panel dimensions in CSS logical pixels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeConfig {
    #[serde(default = "SizeConfig::default_width")]
    pub width: u32,
    #[serde(default = "SizeConfig::default_height")]
    pub height: u32,
}

impl SizeConfig {
    fn default_width() -> u32 { 1920 }
    fn default_height() -> u32 { 120 }
}

impl Default for SizeConfig {
    fn default() -> Self {
        Self { width: Self::default_width(), height: Self::default_height() }
    }
}

// ─── animations ──────────────────────────────────────────────────────────────

/// Describes how the panel appears or disappears.
///
/// Recognised `animation_type` values (frontend interprets these):
///   `"slideDown"` | `"slideUp"` | `"slide"` — translateY animation  
///   `"fadeIn"`    | `"fadeOut"` | `"fade"`  — opacity animation  
/// Unknown values fall back to `"slideDown"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// CSS animation style — see docs/dropshot_config.md
    #[serde(rename = "type", default = "AnimationConfig::default_type")]
    pub animation_type: String,
    /// Transition duration in milliseconds
    #[serde(default = "AnimationConfig::default_duration")]
    pub duration: u32,
}

impl AnimationConfig {
    fn default_type() -> String { "slideDown".to_string() }
    fn default_duration() -> u32 { 220 }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self { animation_type: Self::default_type(), duration: Self::default_duration() }
    }
}

// ─── border ──────────────────────────────────────────────────────────────────

/// Optional border applied to the panel element.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BorderConfig {
    /// CSS colour string, e.g. `"#ffffff"` or `"rgba(255,255,255,0.5)"`
    pub color: Option<String>,
    /// Border width in pixels (`0` → no visible border)
    pub width: Option<u32>,
    /// Border-radius in pixels
    pub radius: Option<u32>,
}

// ─── application ─────────────────────────────────────────────────────────────

/// Top-level panel appearance and behaviour settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Physical size of the panel in CSS logical pixels
    #[serde(rename = "menuSize", default)]
    pub menu_size: SizeConfig,

    /// Animation played when the panel opens
    #[serde(rename = "openingAnimation", default)]
    pub opening_animation: AnimationConfig,

    /// Animation played when the panel closes
    #[serde(rename = "closingAnimation", default)]
    pub closing_animation: AnimationConfig,

    /// CSS `opacity` of the panel (0.0 = fully transparent, 1.0 = opaque)
    #[serde(default = "ApplicationConfig::default_opacity")]
    pub opacity: f64,

    /// CSS colour for the panel background; overrides the glassmorphism gradient
    #[serde(rename = "backgroundColor", default = "ApplicationConfig::default_bg")]
    pub background_color: String,

    /// Arbitrary CSS declarations injected directly into the panel element's
    /// inline style (applied property-by-property, not via a `<style>` tag,
    /// so declarations cannot escape the panel's own style scope).
    #[serde(rename = "extraCSS")]
    pub extra_css: Option<String>,

    /// Optional border drawn around the panel
    #[serde(default)]
    pub border: BorderConfig,

    /// Height in pixels of the drag-handle strip rendered at the bottom edge of
    /// the panel.  Set to `0` to hide the handle entirely.
    #[serde(rename = "handleThickness", default = "ApplicationConfig::default_handle")]
    pub handle_thickness: u32,

    /// Global keyboard shortcut that toggles the panel visibility.
    /// Uses `tauri-plugin-global-shortcut` Accelerator syntax, e.g.
    /// `"Ctrl+Shift+Space"`, `"Alt+F12"`, `"Backquote"`.
    #[serde(rename = "systemHotkey", default = "ApplicationConfig::default_hotkey")]
    pub system_hotkey: String,

    /// Horizontal anchor of the panel when `menuSize.width` < monitor width.
    /// Values: `"left"` | `"center-left"` | `"center"` | `"center-right"` | `"right"`
    /// Determines the window's X position on screen (calculated in Rust setup).
    #[serde(default = "ApplicationConfig::default_position")]
    pub position: String,
}

impl ApplicationConfig {
    pub fn default_opacity() -> f64 { 0.95 }
    pub fn default_bg() -> String { "#1a1d2e".to_string() }
    pub fn default_handle() -> u32 { 4 }
    pub fn default_hotkey() -> String { "Backquote".to_string() }
    pub fn default_position() -> String { "left".to_string() }
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            menu_size: SizeConfig::default(),
            opening_animation: AnimationConfig::default(),
            closing_animation: AnimationConfig {
                animation_type: "slideUp".to_string(),
                duration: 220,
            },
            opacity: Self::default_opacity(),
            background_color: Self::default_bg(),
            extra_css: None,
            border: BorderConfig::default(),
            handle_thickness: Self::default_handle(),
            system_hotkey: Self::default_hotkey(),
            position: Self::default_position(),
        }
    }
}

// ─── effects ─────────────────────────────────────────────────────────────────

/// Visual effect applied to a tile on a specific pointer event.
/// All fields are optional — any combination may appear on any event type.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EffectConfig {
    /// CSS `transform: scale(N)` factor applied during the effect
    pub scale: Option<f64>,
    /// CSS transition duration in milliseconds
    pub duration: Option<u32>,
    /// Arbitrary CSS declarations applied while the effect is active.
    /// Semicolon-separated, e.g. `"background-color: yellow; color: black;"`.
    #[serde(rename = "extraCSS")]
    pub extra_css: Option<String>,
}

/// Pointer-event effects applied to every shortcut tile.
/// Each field is optional; absent events receive a no-op default in the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EffectsConfig {
    pub mouseover: Option<EffectConfig>,
    pub mouseout: Option<EffectConfig>,
    pub click: Option<EffectConfig>,
}

// ─── shortcut ────────────────────────────────────────────────────────────────

/// A single shortcut tile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    /// Display label shown below the tile icon
    pub name: String,

    /// Executable to launch.
    ///   Windows — absolute path or name resolvable via PATH.
    ///   macOS   — absolute path, `.app` bundle path, or app name for `open -a`.
    ///   Linux   — absolute path, PATH-resolvable binary, or `.desktop` file name.
    pub path: String,

    /// Icon filename (with extension) relative to `assets/icons/`, e.g. `"vscode.svg"`
    pub icon: String,

    /// Left edge of the tile in CSS pixels, relative to the panel origin (x=0 is left edge)
    pub x: i32,

    /// Top edge of the tile in CSS pixels, relative to the panel origin (y=0 is top edge)
    pub y: i32,

    /// Tile width in CSS pixels
    pub width: u32,

    /// Tile height in CSS pixels
    pub height: u32,

    /// Arbitrary CSS declarations applied only to this tile's button element.
    /// Semicolon-separated, e.g. `"border: 2px solid #007ACC; border-radius: 10px;"`.
    #[serde(rename = "extraCss")]
    pub extra_css: Option<String>,

    /// Tooltip text shown on hover.  Defaults to `name` when absent.
    pub tooltip: Option<String>,

    /// CLI arguments forwarded to the launched process, e.g. `["--new-window", "--wait"]`
    pub params: Option<Vec<String>>,

    /// Environment variables injected into the launched process.
    pub environment: Option<HashMap<String, String>>,
}

// ─── root ─────────────────────────────────────────────────────────────────────

/// Root structure of `dropshot.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropshotConfig {
    /// Schema version string, e.g. `"1.1"` — used for future migrations
    pub version: String,
    pub application: ApplicationConfig,
    /// Global pointer-event effects for all tiles.  Absent when no effects are desired.
    #[serde(default)]
    pub effects: EffectsConfig,
    pub shortcuts: Vec<Shortcut>,
}

impl Default for DropshotConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            application: ApplicationConfig::default(),
            effects: EffectsConfig::default(),
            shortcuts: Vec::new(),
        }
    }
}

// ── Path resolution ───────────────────────────────────────────────────────────

/// Return the path to the active `dropshot.json`.
///
/// | Mode        | Path |
/// |-------------|------|
/// | **Dev**     | `<workspace>/data/dropshot.json` |
/// | **Windows** | `%APPDATA%\dropshot\dropshot.json` |
/// | **Linux**   | `$XDG_CONFIG_HOME/dropshot/dropshot.json` |
/// | **macOS**   | `~/Library/Application Support/dropshot/dropshot.json` |
///
/// On the first production launch, the bundled default is copied automatically.
pub fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if cfg!(debug_assertions) {
        // `cargo tauri dev` sets cwd to the workspace root, so this resolves
        // to `<workspace>/data/dropshot.json`.
        return Ok(PathBuf::from("data").join("dropshot.json"));
    }

    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Cannot resolve app config dir: {}", e))?;

    let user_config = config_dir.join("dropshot.json");

    if !user_config.exists() {
        seed_default_config(app, &config_dir, &user_config)?;
    }

    Ok(user_config)
}

/// Copy the bundled `data/dropshot.json` into the OS config directory.
/// Falls back to a hard-coded minimal JSON if the bundled file is not found.
/// Called automatically the first time a production build is launched.
fn seed_default_config(
    app: &tauri::AppHandle,
    config_dir: &PathBuf,
    dest: &PathBuf,
) -> Result<(), String> {
    std::fs::create_dir_all(config_dir)
        .map_err(|e| format!("Cannot create config dir {:?}: {}", config_dir, e))?;

    let bundled = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Cannot resolve resource dir: {}", e))?
        .join("data")
        .join("dropshot.json");

    if bundled.exists() {
        std::fs::copy(&bundled, dest)
            .map_err(|e| format!("Failed to seed config from {:?}: {}", bundled, e))?;
    } else {
        // Hard-coded fallback so the app always starts, even if resource bundling fails.
        let default_json = r#"{
  "version": "1.0",
  "application": {
    "menuSize": { "width": 1920, "height": 120 },
    "openingAnimation": { "type": "slideDown", "duration": 220 },
    "closingAnimation": { "type": "slideUp",   "duration": 220 },
    "opacity": 0.95,
    "backgroundColor": "#1a1d2e",
    "border": {},
    "handleThickness": 4,
    "systemHotkey": "Backquote",
    "position": "left"
  },
  "effects": {
    "mouseover": { "scale": 1.06, "duration": 80 },
    "mouseout":  { "scale": 1.0,  "duration": 80 },
    "click":     { "scale": 0.94, "duration": 60 }
  },
  "shortcuts": []
}"#;
        std::fs::write(dest, default_json)
            .map_err(|e| format!("Failed to write default config: {}", e))?;
    }

    Ok(())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Load and parse the full DropShot config from disk.
pub fn load_config(app: &tauri::AppHandle) -> Result<DropshotConfig, String> {
    let path = config_path(app)?;
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse dropshot.json: {}", e))
}

/// Load only the shortcuts list (convenience wrapper around `load_config`).
pub fn load_shortcuts(app: &tauri::AppHandle) -> Result<Vec<Shortcut>, String> {
    Ok(load_config(app)?.shortcuts)
}

/// Persist a modified config back to the user's data file.
pub fn save_config(app: &tauri::AppHandle, config: &DropshotConfig) -> Result<(), String> {
    let path = config_path(app)?;
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write {:?}: {}", path, e))
}

// ── Icon directory ────────────────────────────────────────────────────────────

/// Return the directory that contains user icon files.
///
/// | Mode        | Path |
/// |-------------|------|
/// | **Dev**     | `<cwd>/data/icons/` — read directly from the repo. |
/// | **Windows** | `%APPDATA%\DropShot\icons\` |
/// | **Linux**   | `$XDG_CONFIG_HOME/DropShot/icons\` |
/// | **macOS**   | `~/Library/Application Support/DropShot/icons\` |
///
/// On the first production launch the bundled icons are seeded automatically.
/// Icons already present in the user directory are **never overwritten**, so
/// custom icon replacements survive app updates.
pub fn icons_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if cfg!(debug_assertions) {
        // Dev: absolute path derived from the current working directory.
        // `cargo tauri dev` sets cwd to the workspace root.
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Cannot determine working directory: {}", e))?;
        return Ok(cwd.join("data").join("icons"));
    }

    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Cannot resolve app config dir: {}", e))?;

    let dir = config_dir.join("icons");

    if !dir.exists() {
        seed_icons(app, &dir)?;
    }

    Ok(dir)
}

/// Copy bundled icons from the Tauri resource directory into the user's icon
/// directory.  Only copies files that do **not** already exist there, so any
/// custom icon replacements the user has made are preserved.
fn seed_icons(app: &tauri::AppHandle, dest_dir: &PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Cannot create icons dir {:?}: {}", dest_dir, e))?;

    let src_dir = match app.path().resource_dir() {
        Ok(d) => d.join("data").join("icons"),
        Err(e) => {
            eprintln!("[DropShot] Cannot resolve resource dir for icon seeding: {}", e);
            return Ok(()); // Non-fatal — app continues without bundled icons.
        }
    };

    if !src_dir.exists() {
        return Ok(()); // No bundled icons to seed.
    }

    let entries = std::fs::read_dir(&src_dir)
        .map_err(|e| format!("Cannot read icon source dir {:?}: {}", src_dir, e))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| format!("Error iterating icon source dir: {}", e))?;
        let dest_file = dest_dir.join(entry.file_name());
        // Skip if the user already has this icon (preserves customisations).
        if !dest_file.exists() {
            std::fs::copy(entry.path(), &dest_file)
                .map_err(|e| format!("Failed to seed icon {:?}: {}", dest_file, e))?;
        }
    }

    Ok(())
}
