#!/usr/bin/env bash
#
# Build the mddiff Quick Look extension .appex bundle.
#
# We avoid Xcode by compiling the Swift source directly with `swiftc` and
# assembling the .appex layout by hand. Only Xcode Command Line Tools are
# required (for `swiftc`, `codesign`, and the macOS SDK).
#
# markdown-it.min.js is pulled from the repo's node_modules at build time
# rather than vendored, so `npm install` must have been run in the repo
# root before invoking this script.
#
# Env overrides:
#   ARCH        target architecture (arm64|x86_64|universal). Default arm64.
#   MIN_MACOS   minimum macOS deployment target. Default 11.0.
#   IDENTITY    codesign identity. Default "-" (ad-hoc); pass a Developer ID
#               (e.g. "Developer ID Application: ...") for distribution.
#
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"

BUNDLE_NAME="MddiffQuickLook"
BUILD_DIR="$HERE/build"
APPEX="$BUILD_DIR/${BUNDLE_NAME}.appex"

ARCH="${ARCH:-arm64}"
MIN_MACOS="${MIN_MACOS:-11.0}"
IDENTITY="${IDENTITY:--}"

SDK_PATH="$(xcrun --show-sdk-path --sdk macosx)"

echo "==> Cleaning build dir"
rm -rf "$BUILD_DIR"
mkdir -p "$APPEX/Contents/MacOS"
mkdir -p "$APPEX/Contents/Resources"

echo "==> Compiling Swift (arch: $ARCH, macOS: $MIN_MACOS)"
if [ "$ARCH" = "universal" ]; then
  # Build both arch slices and lipo them together. Slower but produces a
  # single binary that runs on Intel and Apple Silicon.
  TMP_ARM="$BUILD_DIR/${BUNDLE_NAME}.arm64"
  TMP_X86="$BUILD_DIR/${BUNDLE_NAME}.x86_64"
  swiftc -emit-executable -sdk "$SDK_PATH" \
    -target "arm64-apple-macos${MIN_MACOS}" \
    -module-name "$BUNDLE_NAME" \
    -o "$TMP_ARM" \
    "$HERE/src/PreviewProvider.swift"
  swiftc -emit-executable -sdk "$SDK_PATH" \
    -target "x86_64-apple-macos${MIN_MACOS}" \
    -module-name "$BUNDLE_NAME" \
    -o "$TMP_X86" \
    "$HERE/src/PreviewProvider.swift"
  lipo -create -output "$APPEX/Contents/MacOS/$BUNDLE_NAME" "$TMP_ARM" "$TMP_X86"
  rm -f "$TMP_ARM" "$TMP_X86"
else
  swiftc -emit-executable -sdk "$SDK_PATH" \
    -target "${ARCH}-apple-macos${MIN_MACOS}" \
    -module-name "$BUNDLE_NAME" \
    -o "$APPEX/Contents/MacOS/$BUNDLE_NAME" \
    "$HERE/src/PreviewProvider.swift"
fi

echo "==> Copying Info.plist and resources"
cp "$HERE/src/Info.plist" "$APPEX/Contents/Info.plist"
cp "$HERE/resources/preview.html" "$APPEX/Contents/Resources/"
cp "$HERE/resources/preview.css" "$APPEX/Contents/Resources/"

MDIT_SRC="$REPO/node_modules/markdown-it/dist/markdown-it.min.js"
if [ ! -f "$MDIT_SRC" ]; then
  echo "!! markdown-it.min.js not found at: $MDIT_SRC" >&2
  echo "   Run 'npm install' in the repo root first." >&2
  exit 1
fi
cp "$MDIT_SRC" "$APPEX/Contents/Resources/markdown-it.min.js"

# --------------------------------------------------------------------
# highlight.js common bundle + light/dark theme CSS.
#
# The .appex is offline (no CDN), so bundle everything in. We use
# esbuild (installed as a Vite transitive dep) with a temp entry file
# that pulls the "common" subset and stashes it on `window.hljs` for
# the WKWebView global scope. Rebuild-safe: entry file is written
# fresh each run.
# --------------------------------------------------------------------
HLJS_STYLES_DIR="$REPO/node_modules/highlight.js/styles"
if [ ! -d "$HLJS_STYLES_DIR" ]; then
  echo "!! highlight.js not found in node_modules." >&2
  echo "   Run 'npm install' in the repo root first." >&2
  exit 1
fi

HLJS_ENTRY="$BUILD_DIR/.hljs-entry.mjs"
cat > "$HLJS_ENTRY" <<'EOF'
import hljs from "highlight.js/lib/common";
window.hljs = hljs;
EOF

echo "==> Bundling highlight.js common set (esbuild)"
(cd "$REPO" && npx --no-install esbuild "$HLJS_ENTRY" \
  --bundle --minify --format=iife \
  --outfile="$APPEX/Contents/Resources/highlight.min.js" >/dev/null)
rm -f "$HLJS_ENTRY"

# --------------------------------------------------------------------
# KaTeX + @vscode/markdown-it-katex bundle + WOFF2 fonts.
# --------------------------------------------------------------------
KATEX_DIR="$REPO/node_modules/katex/dist"
if [ ! -d "$KATEX_DIR" ]; then
  echo "!! katex not found in node_modules." >&2
  echo "   Run 'npm install' in the repo root first." >&2
  exit 1
fi

KATEX_ENTRY="$BUILD_DIR/.katex-entry.mjs"
cat > "$KATEX_ENTRY" <<'EOF'
import katex from "katex";
import katexPlugin from "@vscode/markdown-it-katex";
window.katex = katex;
// Handle both `export default` and `module.exports = fn` shapes;
// @vscode/markdown-it-katex ships a CJS default that esbuild may
// wrap under `.default`.
window.markdownItKatex = katexPlugin.default ?? katexPlugin;
EOF

echo "==> Bundling KaTeX + markdown-it-katex (esbuild)"
(cd "$REPO" && npx --no-install esbuild "$KATEX_ENTRY" \
  --bundle --minify --format=iife \
  --outfile="$APPEX/Contents/Resources/katex-bundle.js" >/dev/null)
rm -f "$KATEX_ENTRY"

echo "==> Copying KaTeX WOFF2 fonts"
mkdir -p "$APPEX/Contents/Resources/fonts"
# WOFF2 only — WKWebView supports it and dropping WOFF/TTF fallbacks
# saves ~875KB inside the extension. The KaTeX CSS lists WOFF2 first
# in each src: rule so the browser picks it up before failing over
# to the (missing) legacy fonts, and the failed fetches are silent
# in the QuickLook popover.
cp "$KATEX_DIR/fonts"/*.woff2 "$APPEX/Contents/Resources/fonts/"

echo "==> Composing preview.css (base + light/dark hljs themes + KaTeX)"
# Preview base + hljs themes wrapped in prefers-color-scheme media
# queries so the QL popover follows the system light/dark preference
# (QL extensions are isolated processes and can't read the app's own
# theme preference, so following the OS is the sane default here).
# KaTeX CSS is appended verbatim; it references fonts via `url(fonts/...)`
# relative paths which resolve against Resources/fonts/.
PREVIEW_CSS="$APPEX/Contents/Resources/preview.css"
{
  cat "$HERE/resources/preview.css"
  printf '\n\n/* --- highlight.js themes (auto light/dark via prefers-color-scheme) --- */\n'
  printf '\n@media (prefers-color-scheme: light) {\n'
  cat "$HLJS_STYLES_DIR/github.css"
  printf '\n}\n'
  printf '\n@media (prefers-color-scheme: dark) {\n'
  cat "$HLJS_STYLES_DIR/github-dark.css"
  printf '\n}\n'
  printf '\n\n/* --- KaTeX --- */\n'
  cat "$KATEX_DIR/katex.min.css"
} > "$PREVIEW_CSS"

echo "==> Codesigning (identity: $IDENTITY)"
codesign --sign "$IDENTITY" \
  --force --timestamp=none \
  --entitlements "$HERE/src/${BUNDLE_NAME}.entitlements" \
  "$APPEX"

echo ""
echo "==> Built: $APPEX"
du -sh "$APPEX" | awk '{print "    size: " $1}'
