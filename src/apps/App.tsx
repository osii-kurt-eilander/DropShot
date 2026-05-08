import { useEffect, useRef } from 'preact/hooks';
import { signal, effect } from '@preact/signals';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { LogicalSize } from '@tauri-apps/api/dpi';
import { invoke } from '@tauri-apps/api/core';
import { ShortcutGrid } from '../components/ShortcutGrid';
import { preloadIcons } from '../utils/icons';
import type { DropshotConfig } from '../types/config';

// ── Module-level signals ──────────────────────────────────────────────────────
// Kept at module scope so Preact effects that depend on them can be created
// outside the component (e.g. window-resize side-effect).

const isOpen = signal(false);
const config = signal<DropshotConfig | null>(null);

// ── Config loader ─────────────────────────────────────────────────────────────

/**
 * Fetch the full config from Rust and store it in the module-level signal.
 * Simultaneously resizes the native Tauri window to match `application.menuSize`
 * so the OS window and the CSS panel always agree on dimensions.
 */
async function loadConfig(): Promise<void> {
  try {
    const cfg = await invoke<DropshotConfig>('get_config');
    config.value = cfg;

    // Pre-fetch all shortcut icons so ShortcutTile renders can hit the cache
    // synchronously (resolveIconUrl returns from cache after preload).
    await preloadIcons(cfg.shortcuts.map((s: { icon: string }) => s.icon));

    // Resize the native window to match the declared panel size.
    // Uses LogicalSize so the value from the JSON (CSS px) is scaled
    // correctly on HiDPI / Retina displays.
    const { width, height } = cfg.application.menuSize;
    await getCurrentWindow().setSize(new LogicalSize(width, height));
  } catch (err) {
    console.error('[DropShot] Failed to load config:', err);
  }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/**
 * Determine the `data-anim` attribute value for the panel from the opening
 * animation type string.  The CSS uses this attribute to choose between the
 * slide (translateY) and fade (opacity) transition rules.
 *
 * Recognised families:
 *   "slideDown" | "slideUp" | "slide" → "slide"
 *   "fadeIn"    | "fadeOut" | "fade"  → "fade"
 *   anything else                     → "slide"  (safe default)
 */
function animFamily(type: string | undefined): 'slide' | 'fade' {
  if (!type) return 'slide';
  const t = type.toLowerCase();
  if (t.includes('fade')) return 'fade';
  return 'slide';
}

/**
 * Parse a semicolon-separated CSS declarations string and apply each property
 * directly to an element's inline style.  Properties are applied individually
 * so the element's existing positional styles (set via the `style` attribute)
 * are never overwritten wholesale.
 */
function applyInlineCSS(el: HTMLElement, css: string): void {
  css.split(';').forEach((rule) => {
    const idx = rule.indexOf(':');
    if (idx === -1) return;
    const prop = rule.slice(0, idx).trim();
    const val  = rule.slice(idx + 1).trim();
    if (!prop || !val) return;
    // Convert kebab-case to camelCase for HTMLElement.style assignment.
    const camel = prop.replace(/-([a-z])/g, (_, c: string) => c.toUpperCase());
    (el.style as unknown as Record<string, string>)[camel] = val;
  });
}

// ── Component ─────────────────────────────────────────────────────────────────

export function App() {
  const panelRef = useRef<HTMLDivElement>(null);

  // ── Initialise: load config and wire up listeners ──────────────────────────
  useEffect(() => {
    loadConfig();

    // F5 reloads the config from disk without restarting the app.
    function onKeyDown(e: KeyboardEvent): void {
      if (e.key === 'F5') {
        e.preventDefault();
        loadConfig();
      }
    }
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  // ── Toggle panel on Rust global-hotkey event ───────────────────────────────
  useEffect(() => {
    const unlisten = listen<void>('toggle-panel', () => {
      isOpen.value = !isOpen.value;
    });
    return () => { unlisten.then((fn: () => void) => fn()); };
  }, []);

  // ── Show native window before the CSS open-animation plays ────────────────
  useEffect(() => {
    const dispose = effect(() => {
      if (isOpen.value) getCurrentWindow().show();
    });
    return dispose;
  }, []);

  // ── Apply extraCSS from config to the panel element ───────────────────────
  // Runs whenever the loaded config changes.  We apply each property
  // individually via applyInlineCSS so the declarations cannot escape
  // the panel element's own inline style scope.
  useEffect(() => {
    const dispose = effect(() => {
      const css = config.value?.application.extraCSS;
      const el  = panelRef.current;
      if (el && css) applyInlineCSS(el, css);
    });
    return dispose;
  }, []);

  // ── Native window hides after the close-animation finishes ────────────────
  function handleTransitionEnd(e: TransitionEvent): void {
    // Guard against sibling / child transition events bubbling up.
    if (e.target !== panelRef.current) return;
    if (!isOpen.value) getCurrentWindow().hide();
  }

  // ── Derive panel style from current config ─────────────────────────────────
  const cfg = config.value;
  const app = cfg?.application;

  // Animation duration for the current open/close direction.
  const animDuration = isOpen.value
    ? (app?.openingAnimation?.duration  ?? 220)
    : (app?.closingAnimation?.duration ?? 220);

  // The animation family (slide / fade) is driven by the OPENING animation
  // type only — closing is the reverse of the same motion.
  const anim = animFamily(app?.openingAnimation?.type);

  // Inline styles that *cannot* be expressed as static CSS because they come
  // from user config (opacity, backgroundColor, border, transition duration).
  const panelStyle: Record<string, string> = {
    transition: `transform ${animDuration}ms cubic-bezier(0.4,0,0.2,1), `
              + `opacity ${animDuration}ms ease`,
  };
  if (app) {
    if (app.opacity    != null)   panelStyle['opacity']         = String(app.opacity);
    if (app.backgroundColor)      panelStyle['backgroundColor'] = app.backgroundColor;
    if (app.menuSize) {
      panelStyle['width']  = `${app.menuSize.width}px`;
      panelStyle['height'] = `${app.menuSize.height}px`;
    }
    if (app.border) {
      const b = app.border;
      const bw = b.width  ?? 0;
      const bc = b.color  ?? 'transparent';
      const br = b.radius ?? 0;
      if (bw > 0) panelStyle['border']       = `${bw}px solid ${bc}`;
      if (br > 0) panelStyle['borderRadius'] = `${br}px`;
    }
  }

  return (
    <div
      ref={panelRef}
      class={`panel${isOpen.value ? ' panel--open' : ''}`}
      data-anim={anim}
      style={panelStyle}
      onTransitionEnd={handleTransitionEnd}
    >
      {cfg && <ShortcutGrid config={cfg} />}

      {/* Drag handle — a thin strip at the bottom edge of the panel.
          `data-tauri-drag-region` lets the user reposition the window
          by dragging this strip.  Height is controlled by handleThickness. */}
      {app && (app.handleThickness ?? 0) > 0 && (
        <div
          class="panel__handle"
          data-tauri-drag-region
          style={{ height: `${app.handleThickness}px` }}
          aria-hidden="true"
        />
      )}
    </div>
  );
}
