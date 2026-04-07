#!/bin/bash
set -e

# Voice-to-Text PPA Release Script
# Usage: ./scripts/release-ppa.sh [--force]
#
# Pre-flight checks, builds, signs, uploads to PPA for Noble + Jammy,
# tags the release, and verifies upload.

cd "$(dirname "$0")/.."

FORCE=false
[[ "$1" == "--force" ]] && FORCE=true

# ═══════════════════════════════════════════════════════════════
# PRE-FLIGHT CHECKS
# ═══════════════════════════════════════════════════════════════

echo "=== Pre-flight checks ==="

# Check for uncommitted changes (source files only, ignore CONSCIOUSNESS/)
DIRTY=$(git diff --name-only HEAD -- src/ debian/ Makefile.linux vtt.service scripts/ | head -20)
if [ -n "$DIRTY" ] && [ "$FORCE" = false ]; then
    echo "ERROR: Uncommitted source changes:"
    echo "$DIRTY"
    echo ""
    echo "Commit first, or use --force to override."
    exit 1
fi

# Check we're on main
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ] && [ "$FORCE" = false ]; then
    echo "ERROR: Not on main branch (on $BRANCH). Use --force to override."
    exit 1
fi

# Check remote is up to date
git fetch origin main --quiet
LOCAL=$(git rev-parse HEAD)
REMOTE=$(git rev-parse origin/main)
if [ "$LOCAL" != "$REMOTE" ] && [ "$FORCE" = false ]; then
    echo "ERROR: Local and remote are out of sync. Push or pull first."
    exit 1
fi

echo "  Branch: $BRANCH"
echo "  Commit: $(git rev-parse --short HEAD)"
echo "  Tree: clean"
echo ""

# ═══════════════════════════════════════════════════════════════
# VERSION
# ═══════════════════════════════════════════════════════════════

VERSION=$(head -1 debian/changelog | grep -oP '\(.*?\)' | tr -d '()')
DISTRO=$(head -1 debian/changelog | awk '{print $3}' | tr -d ';')

if [ -z "$VERSION" ]; then
    echo "ERROR: Could not read version from debian/changelog"
    exit 1
fi

echo "=== VTT PPA Release v${VERSION} ==="
echo ""

# ═══════════════════════════════════════════════════════════════
# BUILD
# ═══════════════════════════════════════════════════════════════

echo "[1/8] Building..."
make -f Makefile.linux clean
make -f Makefile.linux
echo ""

# ═══════════════════════════════════════════════════════════════
# NOBLE
# ═══════════════════════════════════════════════════════════════

echo "[2/8] Building Noble source package..."
debuild -S -sa -k"emmanuel@powellclark.com"
echo ""

echo "[3/8] Uploading Noble to PPA..."
dput powellclark-voice-to-text "../voice-to-text_${VERSION}_source.changes"
echo ""

# ═══════════════════════════════════════════════════════════════
# JAMMY
# ═══════════════════════════════════════════════════════════════

JAMMY_VERSION="${VERSION}~jammy1"
echo "[4/8] Building Jammy source package (${JAMMY_VERSION})..."
sed -i "1s/${DISTRO}/jammy/" debian/changelog
sed -i "1s/${VERSION}/${JAMMY_VERSION}/" debian/changelog
debuild -S -sa -k"emmanuel@powellclark.com"
echo ""

echo "[5/8] Uploading Jammy to PPA..."
dput powellclark-voice-to-text "../voice-to-text_${JAMMY_VERSION}_source.changes"
echo ""

# ═══════════════════════════════════════════════════════════════
# RESTORE + TAG
# ═══════════════════════════════════════════════════════════════

echo "[6/8] Restoring changelog..."
git checkout debian/changelog
echo ""

echo "[7/8] Tagging v${VERSION}..."
if git tag -l "v${VERSION}" | grep -q .; then
    echo "  Tag v${VERSION} already exists, skipping"
else
    git tag -a "v${VERSION}" -m "Release ${VERSION}"
    git push origin "v${VERSION}"
    echo "  Tagged and pushed v${VERSION}"
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# ARCHIVE BUILD ARTIFACTS
# ═══════════════════════════════════════════════════════════════

echo "[8/8] Archiving build artifacts..."
mkdir -p build-archives
for ext in dsc tar.xz build buildinfo changes; do
    mv -f "../voice-to-text_${VERSION}"*."${ext}" build-archives/ 2>/dev/null || true
    mv -f "../voice-to-text_${JAMMY_VERSION}"*."${ext}" build-archives/ 2>/dev/null || true
done
echo ""

echo "=== Done! v${VERSION} uploaded for Noble and Jammy ==="
echo ""
echo "Monitor: https://launchpad.net/~powellclark/+archive/ubuntu/voice-to-text/+packages"
echo "Install: sudo apt update && sudo apt install voice-to-text"
echo ""
echo "After Launchpad builds (~15min), verify with:"
echo "  apt-cache policy voice-to-text"
