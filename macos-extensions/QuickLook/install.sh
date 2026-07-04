#!/usr/bin/env bash
#
# Copy the built .appex into an mddiff.app's PlugIns directory so macOS
# discovers it on the next Launch Services scan.
#
# Usage:
#   ./install.sh /path/to/mddiff.app
#
# The script also nudges the Quick Look server so the freshly installed
# extension is picked up without a logout/login cycle.
#
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
APPEX="$HERE/build/MddiffQuickLook.appex"

if [ ! -d "$APPEX" ]; then
  echo "!! No build found at $APPEX" >&2
  echo "   Run ./build.sh first." >&2
  exit 1
fi

TARGET_APP="${1:-}"
if [ -z "$TARGET_APP" ]; then
  echo "Usage: $0 /path/to/mddiff.app" >&2
  exit 1
fi

if [ ! -d "$TARGET_APP/Contents" ]; then
  echo "!! $TARGET_APP does not look like a valid .app bundle" >&2
  exit 1
fi

PLUGINS_DIR="$TARGET_APP/Contents/PlugIns"
mkdir -p "$PLUGINS_DIR"

APPEX_NAME="$(basename "$APPEX")"
# Nuke any stale copy so partial overwrites don't leave old resources
# tangled with the new binary (a common source of "the change didn't
# take" confusion).
rm -rf "$PLUGINS_DIR/$APPEX_NAME"
cp -R "$APPEX" "$PLUGINS_DIR/"

echo "==> Copied to $PLUGINS_DIR/$APPEX_NAME"

# Re-register the app with Launch Services so the extension appears in
# `pluginkit -m -p com.apple.quicklook.preview`. Without -f, LS may keep
# using cached metadata and skip the newly-added extension.
LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
if [ -x "$LSREGISTER" ]; then
  "$LSREGISTER" -f "$TARGET_APP" >/dev/null 2>&1 || true
fi

# Bounce the Quick Look server so the next Space press loads the new
# extension. Harmless no-op if it wasn't running.
pkill -f "com.apple.quicklook" 2>/dev/null || true

cat <<EOF

==> Installed. Test with:
    pluginkit -m -p com.apple.quicklook.preview | grep -i mddiff
Then Space-preview any .md file in Finder.
EOF
