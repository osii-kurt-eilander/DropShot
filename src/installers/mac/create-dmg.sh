#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# DropShot — macOS DMG packaging script
# Uses: create-dmg  (https://github.com/create-dmg/create-dmg)
#
# Prerequisites:
#   brew install create-dmg
#
# Called by: make install-mac
# Input:     bin/mac/DropShot.app   (produced by make build-mac)
# Output:    install/mac/DropShot.dmg
#
# Data file seeding:
#   Default data files from data/ are embedded into the .app bundle at:
#     DropShot.app/Contents/Resources/data/dropshot.json
#   On first launch, the Rust binary (via shortcuts.rs / seed_default_config)
#   detects that ~/Library/Application Support/dropshot/dropshot.json does
#   not yet exist and copies the bundled defaults there automatically.
#   No post-install script is required.
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

APP_PATH="$REPO_ROOT/bin/mac/DropShot.app"
OUT_DIR="$REPO_ROOT/install/mac"
DMG_PATH="$OUT_DIR/DropShot.dmg"
APP_VERSION="${DROPSHOT_VERSION:-0.1.0}"

# ── Sanity checks ─────────────────────────────────────────────────────────────
if [ ! -d "$APP_PATH" ]; then
  echo "ERROR: $APP_PATH not found. Run 'make build-mac' first." >&2
  exit 1
fi

if ! command -v create-dmg &>/dev/null; then
  echo "ERROR: create-dmg not found. Install with: brew install create-dmg" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
rm -f "$DMG_PATH"

# ── Embed default data files into the .app bundle ────────────────────────────
# Tauri's resource_dir() resolves to Contents/Resources/ at runtime, so
# the Rust seed_default_config() will find them at:
#   DropShot.app/Contents/Resources/data/dropshot.json
echo "Embedding data files into app bundle..."
DATA_DEST="$APP_PATH/Contents/Resources/data"
mkdir -p "$DATA_DEST"
cp -r "$REPO_ROOT/data/." "$DATA_DEST/"
echo "  Embedded: $(ls "$DATA_DEST")"

# ── Build the DMG ─────────────────────────────────────────────────────────────
echo "Packaging DropShot $APP_VERSION → $DMG_PATH ..."

create-dmg \
  --volname "DropShot $APP_VERSION" \
  --volicon "$REPO_ROOT/assets/logo.svg" \
  --window-pos 200 120 \
  --window-size 660 400 \
  --icon-size 128 \
  --icon "DropShot.app" 160 185 \
  --hide-extension "DropShot.app" \
  --app-drop-link 500 185 \
  --background "$SCRIPT_DIR/dmg-background.png" \
  --no-internet-enable \
  "$DMG_PATH" \
  "$APP_PATH"

echo "Done → $DMG_PATH"
