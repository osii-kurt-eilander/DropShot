# DropShot — Graphic Design Reference

## Logo

<p align="center">
  <img src="../assets/logo.svg" alt="DropShot logo" width="140"/>
</p>

The logo combines two visual metaphors that mirror the app's dual name:
- A **monitor with a panel dropping from the top** — representing the sliding app-launcher shade.
- A **tennis racket with a ball in mid-drop** — representing the drop shot stroke.

### Logo Usage
| Context | Guidance |
|---|---|
| App window titlebar | 24×24 px, no text |
| Installer / about screen | 120×120 px with wordmark |
| README / docs | 120 px wide, centered |
| Favicon / taskbar icon | Export to `.ico` at 16, 32, 48 px sizes |

Do not recolor, stretch, or crop the logo. Maintain at least 16 px clear space on all sides.

---

## Typography

All fonts are open-source and available on [Google Fonts](https://fonts.google.com).

### Primary — [Inter](https://fonts.google.com/specimen/Inter)
Used for UI labels, shortcut tile names, and body text.

```css
font-family: 'Inter', sans-serif;
```

| Role | Weight | Size |
|---|---|---|
| Shortcut tile label | 500 Medium | 12 px |
| Panel heading / section label | 600 SemiBold | 14 px |
| Settings / dialog body | 400 Regular | 13 px |

### Accent / Wordmark — [Outfit](https://fonts.google.com/specimen/Outfit)
Used for the app name "DropShot" in splash screens, the about dialog, and documentation headings. Its geometric rounded terminals echo the racket-head oval in the logo.

```css
font-family: 'Outfit', sans-serif;
font-weight: 700; /* Bold */
```

### Monospace — [JetBrains Mono](https://fonts.google.com/specimen/JetBrains+Mono)
Used for any file paths displayed in the UI (e.g., a shortcut's `.exe` path in a tooltip or settings panel).

```css
font-family: 'JetBrains Mono', monospace;
font-weight: 400;
font-size: 11px;
```

---

## Color Palette

### Dark Mode (default)

| Role | Name | Hex | Swatch |
|---|---|---|---|
| App background / screen body | **Void** | `#12141f` | █ |
| Panel / surface | **Deep Navy** | `#1a1d2e` | █ |
| Raised surface (cards, tiles) | **Slate** | `#23273d` | █ |
| Border / divider | **Rim** | `#2e3450` | █ |
| Primary accent (panel gradient top) | **Sky Blue** | `#4f8ef7` | █ |
| Primary accent (panel gradient bottom) | **Royal Blue** | `#2355c9` | █ |
| Accent highlight / glow | **Ice** | `#7ab0ff` | █ |
| Racket / warm accent (gradient top) | **Gold** | `#f7c948` | █ |
| Racket / warm accent (gradient bottom) | **Amber** | `#e07b20` | █ |
| Tennis ball / success / active | **Volt** | `#c8f74f` | █ |
| Primary text | **Snow** | `#f0f2ff` | █ |
| Secondary text | **Mist** | `#8b90b0` | █ |
| Disabled text | **Fog** | `#555a78` | █ |

#### CSS Custom Properties — Dark
```css
:root[data-theme="dark"] {
  --color-bg:           #12141f;
  --color-surface:      #1a1d2e;
  --color-surface-raised: #23273d;
  --color-border:       #2e3450;
  --color-accent:       #4f8ef7;
  --color-accent-deep:  #2355c9;
  --color-accent-glow:  #7ab0ff;
  --color-warm:         #f7c948;
  --color-warm-deep:    #e07b20;
  --color-volt:         #c8f74f;
  --color-text:         #f0f2ff;
  --color-text-muted:   #8b90b0;
  --color-text-disabled:#555a78;
}
```

---

### Light Mode

| Role | Name | Hex |
|---|---|---|
| App background | **Paper** | `#f4f5fb` |
| Panel / surface | **White** | `#ffffff` |
| Raised surface (cards, tiles) | **Cloud** | `#eaecf7` |
| Border / divider | **Silver** | `#cdd0e3` |
| Primary accent (top) | **Sky Blue** | `#4f8ef7` *(same)* |
| Primary accent (bottom) | **Royal Blue** | `#2355c9` *(same)* |
| Accent highlight | **Cornflower** | `#3a6fd8` |
| Warm accent | **Gold** | `#e09b10` |
| Tennis ball / success / active | **Grass** | `#7ab814` |
| Primary text | **Ink** | `#0e1120` |
| Secondary text | **Slate** | `#4a5070` |
| Disabled text | **Pebble** | `#9397b2` |

#### CSS Custom Properties — Light
```css
:root[data-theme="light"] {
  --color-bg:           #f4f5fb;
  --color-surface:      #ffffff;
  --color-surface-raised: #eaecf7;
  --color-border:       #cdd0e3;
  --color-accent:       #4f8ef7;
  --color-accent-deep:  #2355c9;
  --color-accent-glow:  #3a6fd8;
  --color-warm:         #e09b10;
  --color-warm-deep:    #c47010;
  --color-volt:         #7ab814;
  --color-text:         #0e1120;
  --color-text-muted:   #4a5070;
  --color-text-disabled:#9397b2;
}
```

---

## Panel Styling

The drop-down panel uses a **glassmorphism** treatment in both modes.

```css
/* Dark mode panel */
.panel {
  background: linear-gradient(
    180deg,
    rgba(79, 142, 247, 0.18) 0%,
    rgba(26, 29, 46, 0.92) 100%
  );
  backdrop-filter: blur(16px) saturate(1.4);
  border-bottom: 1px solid rgba(122, 176, 255, 0.25);
}

/* Light mode panel */
[data-theme="light"] .panel {
  background: linear-gradient(
    180deg,
    rgba(255, 255, 255, 0.82) 0%,
    rgba(234, 236, 247, 0.95) 100%
  );
  backdrop-filter: blur(16px) saturate(1.2);
  border-bottom: 1px solid rgba(79, 142, 247, 0.30);
}
```

---

## Shortcut Tile

```css
.tile {
  width: 72px;
  height: 72px;
  border-radius: 14px;                   /* Rounded square, like macOS icons */
  background: var(--color-surface-raised);
  border: 1px solid var(--color-border);
  transition: transform 80ms ease, background 120ms ease;
}

.tile:hover {
  transform: translateY(-3px) scale(1.06);
  background: var(--color-accent);
  border-color: var(--color-accent-glow);
}

.tile__label {
  font-family: 'Inter', sans-serif;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-muted);
}
```

---

## Slide Animation

```css
.panel {
  transform: translateY(-100%);
  transition: transform 220ms cubic-bezier(0.4, 0, 0.2, 1);
}

.panel.is-open {
  transform: translateY(0);
}
```

The easing (`cubic-bezier(0.4, 0, 0.2, 1)`) is Material Design's **standard curve** — fast out, gentle deceleration — which suits a panel snapping down and easing to a stop.

---

## Assets Directory

```
assets/
  logo.svg               # Master logo (scalable)
src/assets/icons/        # Shortcut tile PNG icons (64×64 px minimum)
```

Recommended icon style for shortcut tiles: **flat with subtle shadow**, consistent padding (~10% of canvas), transparent background.
