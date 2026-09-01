#!/usr/bin/env bash
# Installs the cosmic-applet-opencode-quota applet user-locally into ~/.local.
# Usage: ./install.sh   (or override $PREFIX/BINDIR/DATADIR)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ID="net.d11n.CosmicOpencodeQuota"
DESKTOP_SRC="$HERE/net.d11n.CosmicOpencodeQuota.desktop"
ICON_SRC="$HERE/o.svg"
BIN_SRC="$HERE/cosmic-applet-opencode-quota"

BINDIR="${BINDIR:-$HOME/.local/bin}"
DATADIR="${DATADIR:-$HOME/.local/share}"

for f in "$BIN_SRC" "$DESKTOP_SRC" "$ICON_SRC"; do
    [ -f "$f" ] || { echo "Error: $f is missing (run from the extracted tar.gz)" >&2; exit 1; }
done

install -Dm0755 "$BIN_SRC" "$BINDIR/cosmic-applet-opencode-quota"
install -Dm0644 "$ICON_SRC" "$DATADIR/icons/hicolor/scalable/apps/$APP_ID.svg"

DESKTOP_DST="$DATADIR/applications/$APP_ID.desktop"
sed "s|@BIN@|$BINDIR/cosmic-applet-opencode-quota|" "$DESKTOP_SRC" > "$DESKTOP_DST"

echo "Installed:"
echo "  $BINDIR/cosmic-applet-opencode-quota"
echo "  $DESKTOP_DST"
echo "  $DATADIR/icons/hicolor/scalable/apps/$APP_ID.svg"
echo
echo "Add the 'OpenCode Quota' applet in Settings → Panel → Applets (right wing)."
echo "API key: opencode-go in ~/.local/share/opencode/auth.json or env OPENCODE_API_KEY."
