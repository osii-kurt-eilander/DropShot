use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

mod shortcuts;

// ── Tauri Commands ────────────────────────────────────────────────────────────
//
// Each function annotated with `#[tauri::command]` is callable from the
// frontend via `invoke('<name>', { ...args })`.
// All commands are registered in `run()` via `invoke_handler`.

/// Return all shortcuts from `dropshot.json`.
#[tauri::command]
fn get_shortcuts(app: tauri::AppHandle) -> Result<Vec<shortcuts::Shortcut>, String> {
    shortcuts::load_shortcuts(&app)
}

/// Return the full config (application settings + effects + shortcuts).
#[tauri::command]
fn get_config(app: tauri::AppHandle) -> Result<shortcuts::DropshotConfig, String> {
    shortcuts::load_config(&app)
}

/// Persist a modified config back to `dropshot.json`.
#[tauri::command]
fn save_config(
    app: tauri::AppHandle,
    config: shortcuts::DropshotConfig,
) -> Result<(), String> {
    shortcuts::save_config(&app, &config)
}

/// Return the filesystem path of the active `dropshot.json` (useful for
/// debugging / user-facing "open config folder" features).
#[tauri::command]
fn config_path(app: tauri::AppHandle) -> Result<String, String> {
    shortcuts::config_path(&app).map(|p| p.display().to_string())
}

/// Launch an external application.
///
/// # Arguments
/// * `path`        — Executable path or name (see `Shortcut.path` docs).
/// * `params`      — Optional CLI arguments forwarded verbatim.
/// * `environment` — Optional extra environment variables for the child process.
///
/// # Platform behaviour
/// * **Windows** — `CreateProcess` via `std::process::Command`.
/// * **macOS**   — Uses `open -a <path> [--args …]` for `.app` bundles;
///                 falls back to direct exec for plain binaries.
/// * **Linux**   — Uses `xdg-open` only for `.desktop` file names;
///                 otherwise execs the binary directly with params / env.
#[tauri::command]
async fn launch_app(
    path: String,
    params: Option<Vec<String>>,
    environment: Option<std::collections::HashMap<String, String>>,
) -> Result<(), String> {
    let args = params.unwrap_or_default();
    let env  = environment.unwrap_or_default();

    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new(&path);
        cmd.args(&args);
        for (k, v) in &env { cmd.env(k, v); }
        cmd.spawn()
            .map_err(|e| format!("Failed to launch \"{}\": {}", path, e))?;
    }

    #[cfg(target_os = "macos")]
    {
        // `.app` bundles must be opened via `open -a` so macOS applies the
        // correct sandbox / activation policy.
        if path.ends_with(".app") {
            let mut cmd = std::process::Command::new("open");
            cmd.arg("-a").arg(&path);
            if !args.is_empty() { cmd.arg("--args").args(&args); }
            for (k, v) in &env { cmd.env(k, v); }
            cmd.spawn()
                .map_err(|e| format!("Failed to open \"{}\": {}", path, e))?;
        } else {
            let mut cmd = std::process::Command::new(&path);
            cmd.args(&args);
            for (k, v) in &env { cmd.env(k, v); }
            cmd.spawn()
                .map_err(|e| format!("Failed to launch \"{}\": {}", path, e))?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        // `xdg-open` handles `.desktop` file associations; for any real binary
        // (absolute path or PATH-resolvable name) exec it directly so that
        // `params` and `environment` are honoured.
        if path.ends_with(".desktop") {
            std::process::Command::new("xdg-open")
                .arg(&path)
                .spawn()
                .map_err(|e| format!("Failed to open \"{}\": {}", path, e))?;
        } else {
            let mut cmd = std::process::Command::new(&path);
            cmd.args(&args);
            for (k, v) in &env { cmd.env(k, v); }
            cmd.spawn()
                .map_err(|e| format!("Failed to launch \"{}\": {}", path, e))?;
        }
    }

    Ok(())
}

/// Read an icon file from the resolved icons directory and return it as a
/// base64-encoded data URL (`data:<mime>;base64,<data>`).
///
/// This works identically in dev and production:
/// * **dev**        → reads from `<workspace>/data/icons/<filename>`
/// * **production** → reads from `<app_config_dir>/icons/<filename>`;
///                    falls back to the bundled resource copy if the file has
///                    been deleted from the user directory.
///
/// # Security
/// `filename` must be a plain filename (no `/`, `\`, or `..`).  Any attempt
/// to traverse outside the icons directory returns an error immediately.
#[tauri::command]
fn get_icon_data_url(app: tauri::AppHandle, filename: String) -> Result<String, String> {
    // ── Sanitise input ────────────────────────────────────────────────────────
    let name = filename.trim();
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(format!("Invalid icon filename: '{}'", filename));
    }

    // ── Locate the file ───────────────────────────────────────────────────────
    let dir  = shortcuts::icons_dir(&app)?;
    let path = dir.join(name);

    let bytes = if path.exists() {
        std::fs::read(&path)
            .map_err(|e| format!("Cannot read icon '{}': {}", name, e))?
    } else if !cfg!(debug_assertions) {
        // Production fallback: user may have deleted their copy of the icon.
        // Try the bundled resource copy before giving up.
        let bundled = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Cannot resolve resource dir: {}", e))?
            .join("data")
            .join("icons")
            .join(name);
        std::fs::read(&bundled)
            .map_err(|_| format!("Icon '{}' not found in user dir or bundled resources", name))?
    } else {
        return Err(format!("Icon '{}' not found in data/icons/", name));
    };

    // ── Determine MIME type ───────────────────────────────────────────────────
    let mime = match std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
    {
        Some("svg")          => "image/svg+xml",
        Some("png")          => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp")         => "image/webp",
        Some("gif")          => "image/gif",
        _                    => "image/svg+xml",
    };

    // ── Encode and return ─────────────────────────────────────────────────────
    use base64::Engine as _;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

/// Calculate the window's left edge (physical pixels) for a given `position`
/// string when the panel is narrower than the monitor.
///
/// | Value            | Window X            |
/// |------------------|---------------------|
/// | `"left"`         | 0 (flush left)      |
/// | `"center-left"`  | 25 % from left      |
/// | `"center"`       | centred             |
/// | `"center-right"` | 75 % from left      |
/// | `"right"`        | flush right         |
/// | any other        | 0                   |
fn panel_x(position: &str, monitor_w: u32, panel_w: u32) -> i32 {
    if panel_w >= monitor_w { return 0; }
    let gap = (monitor_w - panel_w) as i32;
    match position {
        "center-left"  => gap / 4,
        "center"       => gap / 2,
        "center-right" => gap * 3 / 4,
        "right"        => gap,
        _              => 0,
    }
}

// ── App entry ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    // Only react on key-down; ignore key-up / repeat events.
                    if event.state() == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("toggle-panel", ());
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            // Load config early so we can apply hotkey + window geometry.
            // If the config is unreadable we fall back to built-in defaults
            // and log a message — the app still starts.
            let cfg = shortcuts::load_config(&app.handle()).unwrap_or_else(|err| {
                eprintln!(
                    "[DropShot] Config load failed during setup: {}. Using built-in defaults.",
                    err
                );
                shortcuts::DropshotConfig::default()
            });

            // ── Register the global toggle hotkey ──────────────────────────
            let hotkey = cfg.application.system_hotkey.as_str();
            if let Err(e) = app.global_shortcut().register(hotkey) {
                eprintln!(
                    "[DropShot] Failed to register hotkey \"{}\": {}. \
                     The panel will not respond to keyboard shortcuts.",
                    hotkey, e
                );
            }

            // ── Size and position the native window ────────────────────────
            // Window height and horizontal position come from the config.
            // The width is clamped to the monitor width.
            let window = app
                .get_webview_window("main")
                .expect("'main' window must exist — check tauri.conf.json");

            let panel_h   = cfg.application.menu_size.height;
            let panel_cfg_w = cfg.application.menu_size.width;
            let pos_str   = cfg.application.position.as_str();

            #[cfg(target_os = "windows")]
            {
                use tauri::{PhysicalPosition, PhysicalSize};
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let mw = monitor.size().width;
                    let pw = panel_cfg_w.min(mw);
                    let _ = window.set_size(PhysicalSize { width: pw, height: panel_h });
                    let x = panel_x(pos_str, mw, pw);
                    let _ = window.set_position(PhysicalPosition { x, y: 0i32 });
                } else {
                    let _ = window.set_position(PhysicalPosition { x: 0i32, y: 0i32 });
                }
            }

            #[cfg(target_os = "macos")]
            {
                use tauri::{PhysicalPosition, PhysicalSize};
                // Set always-on-top before making the window visible.
                let _ = window.set_always_on_top(true);
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let mw = monitor.size().width;
                    let pw = panel_cfg_w.min(mw);
                    let _ = window.set_size(PhysicalSize { width: pw, height: panel_h });
                    let x = panel_x(pos_str, mw, pw);
                    let _ = window.set_position(PhysicalPosition { x, y: 0i32 });
                } else {
                    let _ = window.set_position(PhysicalPosition { x: 0i32, y: 0i32 });
                }
            }

            #[cfg(target_os = "linux")]
            {
                use tauri::{PhysicalPosition, PhysicalSize};
                // always_on_top under Wayland is compositor-dependent; degrade gracefully.
                let _ = window.set_always_on_top(true);
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let mw = monitor.size().width;
                    let pw = panel_cfg_w.min(mw);
                    let _ = window.set_size(PhysicalSize { width: pw, height: panel_h });
                    let x = panel_x(pos_str, mw, pw);
                    let _ = window.set_position(PhysicalPosition { x, y: 0i32 });
                } else {
                    let _ = window.set_position(PhysicalPosition { x: 0i32, y: 0i32 });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_shortcuts,
            get_config,
            save_config,
            config_path,
            launch_app,
            get_icon_data_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DropShot");
}
