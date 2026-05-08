import { invoke } from '@tauri-apps/api/core';

/**
 * Icon resolution utility for DropShot.
 *
 * Primary path: calls the Rust `get_icon_data_url` command, which reads the
 * icon from the OS app-config directory (production) or `data/icons/` (dev)
 * and returns a `data:image/svg+xml;base64,...` URL.
 *
 * Fallback: if IPC fails (e.g. running in a plain browser without Tauri), the
 * icon is requested from `/icons/<filename>`, which Vite serves from `data/`
 * (see `publicDir: "data"` in vite.config.ts).
 *
 * Results are cached in a module-level Map to avoid repeated IPC round-trips.
 */

const cache = new Map<string, string>();

/**
 * Resolve a shortcut icon filename to a URL suitable for an `<img src>`.
 * Returns a base64 data URL (via Tauri IPC) or a static path fallback.
 */
export async function resolveIconUrl(filename: string): Promise<string> {
  if (cache.has(filename)) return cache.get(filename)!;
  try {
    const dataUrl = await invoke<string>('get_icon_data_url', { filename });
    cache.set(filename, dataUrl);
    return dataUrl;
  } catch {
    // Dev browser fallback: Vite serves data/ as the public root
    const fallback = `/icons/${filename}`;
    cache.set(filename, fallback);
    return fallback;
  }
}

/**
 * Pre-fetch a list of icons so that subsequent `resolveIconUrl` calls are
 * synchronous (value already in cache). Call this right after loading config.
 */
export async function preloadIcons(filenames: string[]): Promise<void> {
  await Promise.allSettled(filenames.map(resolveIconUrl));
}

/**
 * Synchronous cache lookup — returns the cached URL or undefined if not yet
 * loaded. Useful for render paths that cannot await.
 */
export function getCachedIconUrl(filename: string): string | undefined {
  return cache.get(filename);
}
