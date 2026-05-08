import { useEffect, useRef, useState } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { resolveIconUrl, getCachedIconUrl } from '../utils/icons';
import type { Shortcut, EffectsConfig, EffectConfig } from '../types/config';

interface Props {
  shortcut: Shortcut;
  effects?: EffectsConfig;
}

// ── Effect helpers ────────────────────────────────────────────────────────────

/**
 * Parse a semicolon-delimited CSS declarations string and apply each property
 * to the given element's inline style.
 * Returns the list of camelCase property names that were written so the caller
 * can clean them up later.
 */
function applyCssString(el: HTMLElement, css: string): string[] {
  const written: string[] = [];
  css.split(';').forEach((rule) => {
    const idx = rule.indexOf(':');
    if (idx === -1) return;
    const prop = rule.slice(0, idx).trim();
    const val  = rule.slice(idx + 1).trim();
    if (!prop || !val) return;
    const camel = prop.replace(/-([a-z])/g, (_, c: string) => c.toUpperCase());
    (el.style as unknown as Record<string, string>)[camel] = val;
    written.push(camel);
  });
  return written;
}

/**
 * Apply an `EffectConfig` to a DOM element:
 *   • sets `transform: scale(N)` and a matching `transition`
 *   • applies any `extraCSS` declarations
 *
 * Returns the camelCase property names written by `extraCSS` so the caller
 * can erase them on the next state change.
 */
function applyEffect(el: HTMLElement, cfg: EffectConfig): string[] {
  const scale    = cfg.scale    ?? 1;
  const duration = cfg.duration ?? 80;
  el.style.transition = `transform ${duration}ms ease`;
  el.style.transform  = `scale(${scale})`;
  if (cfg.extraCSS) return applyCssString(el, cfg.extraCSS);
  return [];
}

// ── Component ─────────────────────────────────────────────────────────────────

export function ShortcutTile({ shortcut, effects }: Props) {
  const btnRef = useRef<HTMLButtonElement>(null);

  // ── Icon URL (async, with cache) ──────────────────────────────────────────
  // Start with the cached value (populated by preloadIcons in App.tsx) so the
  // first render is synchronous. If the cache is cold, kick off an async
  // resolve and update state when it settles.
  const [iconUrl, setIconUrl] = useState<string>(
    () => getCachedIconUrl(shortcut.icon) ?? `/icons/${shortcut.icon}`
  );
  useEffect(() => {
    let cancelled = false;
    resolveIconUrl(shortcut.icon).then((url: string) => {
      if (!cancelled) setIconUrl(url);
    });
    return () => { cancelled = true; };
  }, [shortcut.icon]);

  /**
   * CSS property names written by the last mouseover extraCSS.
   * We track them so we can erase exactly those properties on mouseleave,
   * rather than blindly wiping the whole inline style string.
   */
  const hoverExtraProps = useRef<string[]>([]);

  // ── Apply per-tile extraCss after mount / when it changes ─────────────────
  // We do this in a useEffect rather than via the JSX `style` prop because
  // mixing an object-form style prop with `cssText` doesn't work — the browser
  // treats `cssText` as an unknown CSS property and silently ignores it.
  useEffect(() => {
    const el = btnRef.current;
    if (el && shortcut.extraCss) applyCssString(el, shortcut.extraCss);
  }, [shortcut.extraCss]);

  // ── Inline style: position + size only ────────────────────────────────────
  // extraCss is handled separately by the useEffect above.
  const tileStyle: Record<string, string> = {
    position: 'absolute',
    left:     `${shortcut.x}px`,
    top:      `${shortcut.y}px`,
    width:    `${shortcut.width}px`,
    height:   `${shortcut.height}px`,
  };

  // ── Event handlers ────────────────────────────────────────────────────────

  function handleMouseEnter(): void {
    const el = btnRef.current;
    if (!el || !effects?.mouseover) return;
    hoverExtraProps.current = applyEffect(el, effects.mouseover);
  }

  function handleMouseLeave(): void {
    const el = btnRef.current;
    if (!el) return;

    // Erase CSS properties that were injected by the mouseover extraCSS.
    // We must do this BEFORE applying the mouseout effect so the mouseout's
    // own extraCSS (if any) takes precedence.
    hoverExtraProps.current.forEach((prop) => {
      (el.style as unknown as Record<string, string>)[prop] = '';
    });
    hoverExtraProps.current = [];

    if (effects?.mouseout) {
      applyEffect(el, effects.mouseout);
    } else {
      // Smooth return to scale(1) even when no mouseout config is defined.
      el.style.transition = 'transform 80ms ease';
      el.style.transform  = 'scale(1)';
    }
  }

  async function handleClick(): Promise<void> {
    const el = btnRef.current;
    if (el && effects?.click) applyEffect(el, effects.click);

    try {
      await invoke('launch_app', {
        path:        shortcut.path,
        params:      shortcut.params      ?? [],
        environment: shortcut.environment ?? {},
      });
      // Hide the panel after a successful launch.
      await getCurrentWindow().hide();
    } catch (err) {
      console.error(`[DropShot] Failed to launch "${shortcut.name}":`, err);
    }
  }

  return (
    <button
      ref={btnRef}
      class="tile"
      style={tileStyle}
      onClick={handleClick}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      title={shortcut.tooltip ?? shortcut.name}
      aria-label={`Launch ${shortcut.name}`}
    >
      <img
        class="tile__icon"
        src={iconUrl}
        alt=""
        draggable={false}
        onError={(e: Event) => {
          // Dim the broken-image placeholder instead of showing a torn icon.
          (e.currentTarget as HTMLImageElement).style.opacity = '0.3';
        }}
      />
      <span class="tile__label">{shortcut.name}</span>
    </button>
  );
}


