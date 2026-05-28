#!/usr/bin/env bash
# Regenerate the application icon set from the master SVG.
#
# Source of truth: app-icon/aui.svg
# Output:          src-tauri/icons/  (used for app, window, and tray)
#
# Preferred (needs the dev deps installed):
#   npx tauri icon app-icon/aui.svg
#
# Fallback below uses only system tools (rsvg-convert + ImageMagick), handy
# when node_modules isn't installed.
set -euo pipefail

cd "$(dirname "$0")/.."
SVG="app-icon/aui.svg"
OUT="src-tauri/icons"
TMP="$(mktemp -d)/icon-1024.png"

rsvg-convert -w 1024 -h 1024 "$SVG" -o "$TMP"

convert "$TMP" -resize 32x32   "$OUT/32x32.png"
convert "$TMP" -resize 128x128 "$OUT/128x128.png"
convert "$TMP" -resize 256x256 "$OUT/128x128@2x.png"
convert "$TMP" -resize 512x512 "$OUT/icon.png"
convert "$TMP" -define icon:auto-resize=256,128,64,48,32,16 "$OUT/icon.ico"
convert "$TMP" "$OUT/icon.icns"

echo "Icon set regenerated in $OUT from $SVG"
