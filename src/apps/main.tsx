import { render } from 'preact';
import { App } from './App';
import './app.css';

// ── Sync data-theme attribute with the OS dark/light preference ──────────────
// This must run before first render to avoid a flash of wrong theme.
function applyTheme(dark: boolean): void {
  document.documentElement.dataset.theme = dark ? 'dark' : 'light';
}

const mq = window.matchMedia('(prefers-color-scheme: dark)');
applyTheme(mq.matches);
mq.addEventListener('change', (e) => applyTheme(e.matches));

// ── Mount ────────────────────────────────────────────────────────────────────
render(<App />, document.getElementById('app')!);
