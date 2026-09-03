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

echo "[1/4] Vendoring cargo dependencies..."
if [ ! -d vendor ] || [ ! -f vendor/anyhow/Cargo.toml.orig ]; then
    cargo vendor > /dev/null
fi
echo "  vendor/ size: $(du -sh vendor/ | cut -f1)"

# Force Cargo.lock v3 format so Ubuntu Noble cargo 1.75 can parse it.
# rustup >=1.78 writes v4 by default, which breaks Launchpad builds.
if grep -q '^version = 4$' Cargo.lock; then
    sed -i 's/^version = 4$/version = 3/' Cargo.lock
fi
echo ""

# Staleness gate. debian/rules deliberately does not compile — it installs the
# committed packaging/linux/vtt-linux.prebuilt, because Launchpad builds on
# Noble whose cargo 1.75 cannot parse this crate's edition-2024 manifest. The
# cost of that is a .deb which can silently ship a binary older than the source:
# on 2026-09-03 this script ran four green stages and installed a two-week-old
# build, after a sudo prompt, with no warning (TASK-VTT152). The pre-flight
# below proves the SOURCE compiles and proves nothing about what gets PACKAGED.
PREBUILT="packaging/linux/vtt-linux.prebuilt"
if [ -f "$PREBUILT" ]; then
    PREBUILT_EPOCH=$(stat -c %Y "$PREBUILT")
    SRC_EPOCH=$(git log -1 --format=%ct -- src/ Cargo.toml Cargo.lock 2>/dev/null || echo 0)
    if [ "$SRC_EPOCH" -gt "$PREBUILT_EPOCH" ]; then
        echo "ERROR: $PREBUILT is older than the source it is supposed to build from."
        echo "  prebuilt:     $(date -d "@$PREBUILT_EPOCH" '+%Y-%m-%d %H:%M:%S')"
        echo "  newest src/:  $(date -d "@$SRC_EPOCH" '+%Y-%m-%d %H:%M:%S')  ($(git log -1 --format=%h -- src/ Cargo.toml Cargo.lock))"
        echo ""
        echo "The .deb installs this file verbatim, so building now would ship stale code."
        echo "Refresh it:"
        echo "  cargo build --release && cp target/release/vtt-linux $PREBUILT"
        exit 1
    fi
    echo "[0/4] Prebuilt is current (built $(date -d "@$PREBUILT_EPOCH" '+%Y-%m-%d %H:%M')); it is what the .deb ships."
    echo ""
fi

echo "[2/4] Pre-flight: cargo build --offline --locked (mirrors Launchpad)..."
if ! cargo build --release --offline --locked 2>&1 | tail -5 | grep -q "Finished\|up to date"; then
    echo "  WARN — offline locked build may have issues. Continuing with debuild anyway."
else
    echo "  OK — offline locked build succeeds. Launchpad would build too."
fi
echo ""

echo "[3/4] Running debuild -b -us -uc -d ..."
debuild -b -us -uc -d
echo ""

DEB="../voice-to-text_${VERSION}_amd64.deb"
if [ ! -f "$DEB" ]; then
    echo "ERROR: Build did not produce $DEB"
    exit 1
fi

echo "[4/4] Built: $(realpath "$DEB") ($(du -h "$DEB" | cut -f1))"
echo ""

# Stage in /tmp (world-readable) before apt install: apt's sandboxed _apt
# user can't read files under a home directory locked down to 700, which
# otherwise prints a harmless but recurring "unsandboxed as root" warning.
STAGED_DEB="/tmp/$(basename "$DEB")"
cp "$DEB" "$STAGED_DEB"
chmod 644 "$STAGED_DEB"

if [ "$INSTALL" = true ]; then
    echo "=== Installing (you may be prompted for sudo) ==="
    sudo apt install -y "$STAGED_DEB"
    echo ""
    echo "Installed. Restart VTT with:"
    echo "  pkill -f vtt-linux; /usr/bin/vtt-linux &"
else
    echo "Install with:"
    echo "  sudo apt install \"$STAGED_DEB\""
    echo ""
    echo "Or pass --install to install now."
fi
