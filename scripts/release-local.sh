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

# debian/rules deliberately does not compile — it installs
# packaging/linux/vtt.prebuilt verbatim, because Launchpad builds on Noble
# whose cargo 1.75 cannot parse this crate's edition-2024 manifest. The
# prebuilt is gitignored and built here, from the checked-out source, on every
# run, so the .deb can never ship a binary older than the source.
PREBUILT="packaging/linux/vtt.prebuilt"
echo "[2/4] Building the release binary (offline, locked, build paths remapped)..."
bash scripts/build-release.sh --offline --locked
cp target/release/vtt "$PREBUILT"
# scripts/build-release.sh refuses a binary that still holds this machine's
# home path; check the staged copy too, since the .deb publishes it.
if grep -q -a -F -- "$HOME" "$PREBUILT"; then
    echo "ERROR: $PREBUILT holds this machine's home directory path; do not ship it."
    exit 1
fi
echo "  OK — staged $PREBUILT ($(du -h "$PREBUILT" | cut -f1)); it is what the .deb ships."
echo ""

echo "[3/4] Running debuild -b -us -uc -d ..."
debuild -b -us -uc -d
echo ""

DEB="../voice-to-text_${VERSION}_amd64.deb"
if [ ! -f "$DEB" ]; then
    echo "ERROR: Build did not produce $DEB"
    exit 1
fi

echo "[4/4] Built: $(basename "$DEB") ($(du -h "$DEB" | cut -f1))"

# Stage in /tmp (world-readable) before apt install: apt's sandboxed _apt
# user can't read files under a home directory locked down to 700, which
# otherwise prints a harmless but recurring "unsandboxed as root" warning.
STAGED_DEB="/tmp/$(basename "$DEB")"
cp "$DEB" "$STAGED_DEB"
chmod 644 "$STAGED_DEB"

# debuild writes the .deb, .build, .buildinfo and .changes beside the repo,
# in the parent folder. Move this run's into build-archives/, as
# release-ppa.sh does, so the parent folder is left as it was; the staged
# copy above is the one to install.
mkdir -p build-archives
archived=0
for f in ../voice-to-text_"${VERSION}"_*; do
    if [ -f "$f" ]; then
        mv -f "$f" build-archives/
        archived=$((archived + 1))
    fi
done
echo "  Staged $STAGED_DEB; archived $archived build output(s) to build-archives/"
echo ""

if [ "$INSTALL" = true ]; then
    echo "=== Installing (you may be prompted for sudo) ==="
    # dpkg -i, NOT apt install. A local build carries the same version as what
    # is already installed — that is the normal case for this flag — and apt
    # answers "already the newest version", exits 0, and installs nothing. On
    # 2026-09-03 that printed "Installed." over a no-op twice (TASK-VTT137).
    PACKAGED_SHA=$(dpkg-deb --fsys-tarfile "$STAGED_DEB" 2>/dev/null \
        | tar -xO ./usr/bin/vtt 2>/dev/null | sha256sum | cut -d' ' -f1)
    sudo dpkg -i "$STAGED_DEB"
    # dpkg does not resolve dependencies; this is the documented repair and a
    # no-op when nothing is broken.
    sudo apt-get -f install -y

    # Verify rather than trust the exit code. Compare content, never mtime:
    # dpkg preserves the packaged timestamp, which is exactly what made a
    # two-week-old binary look current this morning.
    INSTALLED_SHA=$(sha256sum /usr/bin/vtt 2>/dev/null | cut -d' ' -f1)
    if [ -n "$PACKAGED_SHA" ] && [ "$PACKAGED_SHA" != "$INSTALLED_SHA" ]; then
        echo ""
        echo "ERROR: /usr/bin/vtt does not match the package just installed."
        echo "  packaged:  $PACKAGED_SHA"
        echo "  installed: $INSTALLED_SHA"
        echo "The install did not take. Do not trust a later 'it works'."
        exit 1
    fi
    echo "  Verified: /usr/bin/vtt matches the package."
    echo ""

    # A running process keeps executing the inode it started from, so the
    # install has NOT reached the app until it restarts. pkill is the wrong
    # tool: vtt.service carries Restart=always, so it respawns the old binary
    # within five seconds and the singleton lock then blocks a replacement.
    STALE_PIDS=""
    for exe in /proc/[0-9]*/exe; do
        target=$(readlink "$exe" 2>/dev/null) || continue
        case "$target" in
            */vtt|*vtt-linux*) STALE_PIDS="$STALE_PIDS ${exe#/proc/}" ;;
        esac
    done
    STALE_PIDS=$(echo "$STALE_PIDS" | sed 's|/exe||g' | tr -s ' ')
    if [ -n "$(echo "$STALE_PIDS" | tr -d ' ')" ]; then
        echo "NOTE: vtt is running (pid$STALE_PIDS) and still holds the OLD binary."
        echo "      It keeps the replaced inode until it restarts."
    fi

    echo "Restart it so the new binary actually runs:"
    echo "  systemctl --user restart vtt.service"
    echo ""
    echo "Then confirm:"
    echo "  vtt --doctor            # 'Everything checks out' means it took"
else
    echo "Install with:"
    echo "  sudo dpkg -i \"$STAGED_DEB\"   # dpkg, not apt: same version installs over"
    echo ""
    echo "Or pass --install to install now."
fi
