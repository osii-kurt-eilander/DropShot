// ─────────────────────────────────────────────────────────────────────────────
// DropShot — TypeScript interfaces mirroring the dropshot.json schema.
//
// Rules:
//   • Every field that may be absent in the JSON is marked `?` (optional).
//   • Fields that appear on one sibling object of the same type (e.g. extraCSS
//     on one EffectConfig) may appear on ANY sibling — never assume absence.
//   • See docs/dropshot_config.md for the full description of every field.
// ─────────────────────────────────────────────────────────────────────────────

/** Panel dimensions in CSS logical pixels. */
export interface SizeConfig {
  width: number;
  height: number;
}

/**
 * Animation configuration for opening or closing the panel.
 *
 * `type` values and their meaning:
 *   "slideDown" | "slideUp" | "slide" → translateY animation (default)
 *   "fadeIn"    | "fadeOut" | "fade"  → opacity animation
 * Unknown values fall back to `"slideDown"`.
 */
export interface AnimationConfig {
  /** Animation style identifier — see docs/dropshot_config.md */
  type?: string;
  /** Transition duration in milliseconds */
  duration?: number;
}

/** Optional border drawn around the panel element. */
export interface BorderConfig {
  /** CSS colour string, e.g. `"#ffffff"` */
  color?: string;
  /** Border width in pixels; `0` → no visible border */
  width?: number;
  /** Border-radius in pixels */
  radius?: number;
}

/** Top-level panel appearance and behaviour settings. */
export interface ApplicationConfig {
  /** Dimensions of the panel in CSS logical pixels */
  menuSize: SizeConfig;
  /** Animation played when the panel opens */
  openingAnimation?: AnimationConfig;
  /** Animation played when the panel closes */
  closingAnimation?: AnimationConfig;
  /** Panel CSS `opacity` — `0.0` (transparent) to `1.0` (opaque) */
  opacity?: number;
  /** CSS colour for the panel background; overrides the glassmorphism default */
  backgroundColor?: string;
  /**
   * Arbitrary CSS declarations injected into the panel element's inline style.
   * Applied property-by-property (not via a `<style>` tag) so they cannot
   * leak out of the panel element's own style scope.
   * Semicolon-separated, e.g. `"box-shadow: 0 0 10px rgba(0,0,0,0.5);"`.
   */
  extraCSS?: string;
  /** Optional border drawn around the panel */
  border?: BorderConfig;
  /**
   * Height in pixels of the drag-handle strip at the bottom of the panel.
   * Set to `0` to hide it entirely.
   */
  handleThickness?: number;
  /**
   * Global keyboard shortcut that toggles the panel.
   * Uses `tauri-plugin-global-shortcut` Accelerator syntax:
   * e.g. `"Ctrl+Shift+Space"`, `"Alt+F12"`, `"Backquote"`.
   */
  systemHotkey?: string;
  /**
   * Horizontal anchor of the panel when `menuSize.width` < monitor width.
   * Values: `"left"` | `"center-left"` | `"center"` | `"center-right"` | `"right"`
   */
  position?: string;
}

/**
 * Visual effect applied to a shortcut tile on a specific pointer event.
 * All fields are optional — any combination may appear on any event type.
 */
export interface EffectConfig {
  /** CSS `transform: scale(N)` factor; `1.0` = no change */
  scale?: number;
  /** CSS transition duration in milliseconds */
  duration?: number;
  /**
   * Arbitrary CSS declarations applied while this effect is active.
   * Semicolon-separated, e.g. `"background-color: yellow; color: black;"`.
   * Properties are individually cleared when the effect ends.
   */
  extraCSS?: string;
}

/**
 * Per-event-type effects applied to all shortcut tiles globally.
 * Each event is optional; absent events receive a silent no-op in the frontend.
 */
export interface EffectsConfig {
  /** Effect applied when the pointer enters a tile */
  mouseover?: EffectConfig;
  /** Effect applied when the pointer leaves a tile */
  mouseout?: EffectConfig;
  /** Effect applied when a tile is clicked */
  click?: EffectConfig;
}

/** A single shortcut tile entry. */
export interface Shortcut {
  /** Display label shown below the tile icon */
  name: string;
  /**
   * Executable to launch.
   *   Windows — absolute path or name resolvable via `PATH`
   *   macOS   — absolute path, `.app` bundle, or app name for `open -a`
   *   Linux   — absolute path, `PATH` binary, or `.desktop` file name
   */
  path: string;
  /** Icon filename (with extension) relative to `assets/icons/` */
  icon: string;
  /** Left edge of the tile in CSS px, relative to the panel's left edge */
  x: number;
  /** Top edge of the tile in CSS px, relative to the panel's top edge */
  y: number;
  /** Tile width in CSS pixels */
  width: number;
  /** Tile height in CSS pixels */
  height: number;
  /**
   * Arbitrary CSS declarations applied to this tile's `<button>` element.
   * Applied via DOM ref after mount, not via the JSX `style` prop.
   */
  extraCss?: string;
  /** Tooltip text shown on hover; defaults to `name` when absent */
  tooltip?: string;
  /** CLI arguments forwarded to the launched process */
  params?: string[];
  /** Environment variables injected into the launched process */
  environment?: Record<string, string>;
}

/** Root structure of `dropshot.json`. */
export interface DropshotConfig {
  /** Schema version string, e.g. `"1.1"` — used for future migrations */
  version: string;
  application: ApplicationConfig;
  /** Global pointer-event effects for all tiles; absent = no effects */
  effects?: EffectsConfig;
  shortcuts: Shortcut[];
}
