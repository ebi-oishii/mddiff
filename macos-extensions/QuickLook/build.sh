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

echo "==> Codesigning (identity: $IDENTITY)"
codesign --sign "$IDENTITY" \
  --force --timestamp=none \
  --entitlements "$HERE/src/${BUNDLE_NAME}.entitlements" \
  "$APPEX"

echo ""
echo "==> Built: $APPEX"
du -sh "$APPEX" | awk '{print "    size: " $1}'
