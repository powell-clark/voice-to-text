#!/bin/bash
set -e

# Voice-to-Text Local .deb Build Script (2.0.0+)
# Usage: ./scripts/release-local.sh [--install]
#
# Builds a binary .deb locally (no signing, no PPA upload). Use for local
# testing or a side-load install without the PPA round-trip. Pair with
# `sudo apt install ./../voice-to-text_X.Y.Z_amd64.deb` or pass --install
# to install immediately.

cd "$(dirname "$0")/.."

INSTALL=false
for arg in "$@"; do
    [[ "$arg" == "--install" ]] && INSTALL=true
done

VERSION=$(head -1 debian/changelog | grep -oP '\(.*?\)' | tr -d '()')

echo "=== Building voice-to-text $VERSION .deb ==="
echo ""

# Ensure cargo is on PATH for debian/rules (rustup installs under ~/.cargo)
export PATH="$HOME/.cargo/bin:$PATH"

echo "[1/3] Vendoring cargo dependencies..."
if [ ! -d vendor ] || [ ! -f vendor/anyhow/Cargo.toml.orig ]; then
    cargo vendor > /dev/null
fi
echo "  vendor/ size: $(du -sh vendor/ | cut -f1)"
echo ""

echo "[2/3] Running debuild -b -us -uc -d ..."
debuild -b -us -uc -d
echo ""

DEB="../voice-to-text_${VERSION}_amd64.deb"
if [ ! -f "$DEB" ]; then
    echo "ERROR: Build did not produce $DEB"
    exit 1
fi

echo "[3/3] Built: $(realpath "$DEB") ($(du -h "$DEB" | cut -f1))"
echo ""

if [ "$INSTALL" = true ]; then
    echo "=== Installing (you may be prompted for sudo) ==="
    sudo apt install -y "$DEB"
    echo ""
    echo "Installed. Restart VTT with:"
    echo "  pkill -f vtt-linux; /usr/bin/vtt-linux &"
else
    echo "Install with:"
    echo "  sudo apt install \"$DEB\""
    echo ""
    echo "Or pass --install to install now."
fi
