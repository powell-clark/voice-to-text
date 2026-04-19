#!/bin/bash
set -e

# Voice-to-Text PPA Release Script (2.0.0+)
# Usage: ./scripts/release-ppa.sh [--force] [--dry-run]
#
# Pre-flight checks, vendored cargo build, debuild source package,
# sign with GPG, upload to Launchpad PPA for all supported Ubuntu
# releases, tag the release, and archive artifacts.

cd "$(dirname "$0")/.."

# ═══════════════════════════════════════════════════════════════
# CONFIG
# ═══════════════════════════════════════════════════════════════

GPG_KEY="${VTT_GPG_KEY:-emmanuel@powellclark.com}"
PPA_TARGET="${VTT_PPA_TARGET:-powellclark-voice-to-text}"
DISTROS=("noble" "jammy")  # LTS releases

# ═══════════════════════════════════════════════════════════════
# FLAGS
# ═══════════════════════════════════════════════════════════════

FORCE=false
DRY_RUN=false
for arg in "$@"; do
    [[ "$arg" == "--force" ]] && FORCE=true
    [[ "$arg" == "--dry-run" ]] && DRY_RUN=true
done

# ═══════════════════════════════════════════════════════════════
# PRE-FLIGHT CHECKS
# ═══════════════════════════════════════════════════════════════

echo "=== Pre-flight checks ==="

DIRTY=$(git diff --name-only HEAD -- src/ debian/ Cargo.toml vtt.service scripts/ | head -20)
if [ -n "$DIRTY" ] && [ "$FORCE" = false ]; then
    echo "ERROR: Uncommitted source changes:"
    echo "$DIRTY"
    echo ""
    echo "Commit first, or use --force to override."
    exit 1
fi

BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ] && [ "$FORCE" = false ]; then
    echo "ERROR: Not on main branch (on $BRANCH). Use --force to override."
    exit 1
fi

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
# VERSION VALIDATION
# ═══════════════════════════════════════════════════════════════

VERSION=$(head -1 debian/changelog | grep -oP '\(.*?\)' | tr -d '()')
DISTRO=$(head -1 debian/changelog | awk '{print $3}' | tr -d ';')

if [ -z "$VERSION" ]; then
    echo "ERROR: Could not read version from debian/changelog"
    exit 1
fi

CARGO_VERSION=$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')
if [ "$CARGO_VERSION" != "$VERSION" ] && [ "$FORCE" = false ]; then
    echo "ERROR: Cargo.toml version ($CARGO_VERSION) != debian/changelog ($VERSION)"
    echo "Align both before releasing."
    exit 1
fi

LAST_TAG=$(git tag -l 'v*' --sort=-v:refname | head -1 | sed 's/^v//')
if [ -n "$LAST_TAG" ]; then
    if dpkg --compare-versions "$VERSION" le "$LAST_TAG" 2>/dev/null; then
        if [ "$FORCE" = false ]; then
            echo "ERROR: Version $VERSION is not newer than last tag v$LAST_TAG"
            exit 1
        fi
    fi
    echo "  Last release: v$LAST_TAG"
fi

if git tag -l "v${VERSION}" | grep -q .; then
    if [ "$FORCE" = false ]; then
        echo "ERROR: Tag v${VERSION} already exists."
        exit 1
    fi
fi

CHANGELOG_LINES=$(sed -n '2,/^ --/p' debian/changelog | grep -c '^\s\+\*' || true)
if [ "$CHANGELOG_LINES" -eq 0 ] && [ "$FORCE" = false ]; then
    echo "ERROR: No changelog entries found for v${VERSION}."
    exit 1
fi

echo "  Version: $VERSION ($CHANGELOG_LINES changelog entries)"
echo "  Distros: ${DISTROS[*]}"
echo ""

# ═══════════════════════════════════════════════════════════════
# VENDOR CARGO DEPENDENCIES
# ═══════════════════════════════════════════════════════════════

echo "[1/${STEP_TOTAL:-?}] Vendoring cargo dependencies..."
cargo vendor > /tmp/vtt-cargo-config.toml
# The .cargo/config.toml is committed; we don't need to overwrite it.
echo "  vendor/ size: $(du -sh vendor/ | cut -f1)"

# Force Cargo.lock v3 format so Ubuntu Noble cargo 1.75 can parse it.
# rustup >=1.78 writes v4 by default, which breaks Launchpad builds.
if grep -q '^version = 4$' Cargo.lock; then
    echo "  Downgrading Cargo.lock v4 -> v3 for Ubuntu 1.75 compatibility"
    sed -i 's/^version = 4$/version = 3/' Cargo.lock
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# DRY RUN
# ═══════════════════════════════════════════════════════════════

if [ "$DRY_RUN" = true ]; then
    echo "=== DRY RUN — would release v${VERSION} ==="
    echo ""
    echo "Steps:"
    echo "  1. cargo vendor (done above)"
    for distro in "${DISTROS[@]}"; do
        if [ "$distro" = "$DISTRO" ]; then
            echo "  2. debuild -S -sa -k$GPG_KEY (${distro})"
            echo "  3. dput $PPA_TARGET voice-to-text_${VERSION}_source.changes"
        else
            suffix="~${distro}1"
            echo "  2. debuild -S -sa -k$GPG_KEY (${distro}, ${VERSION}${suffix})"
            echo "  3. dput $PPA_TARGET voice-to-text_${VERSION}${suffix}_source.changes"
        fi
    done
    echo "  4. git tag v${VERSION} && git push origin v${VERSION}"
    echo "  5. Archive artifacts to build-archives/"
    exit 0
fi

# ═══════════════════════════════════════════════════════════════
# BUILD SOURCE PACKAGES + UPLOAD
# ═══════════════════════════════════════════════════════════════

TOTAL_STEPS=$(( 2 + ${#DISTROS[@]} * 2 + 2 ))
STEP=1

FIRST=true
for distro in "${DISTROS[@]}"; do
    if [ "$FIRST" = true ]; then
        STEP=$((STEP+1))
        echo "[$STEP/$TOTAL_STEPS] Building ${distro} source package..."
        debuild -S -sa -k"$GPG_KEY" -d
        echo ""

        STEP=$((STEP+1))
        echo "[$STEP/$TOTAL_STEPS] Uploading ${distro} to PPA..."
        dput "$PPA_TARGET" "../voice-to-text_${VERSION}_source.changes"
        echo ""

        FIRST=false
    else
        VARIANT="${VERSION}~${distro}1"
        STEP=$((STEP+1))
        echo "[$STEP/$TOTAL_STEPS] Building ${distro} source package (${VARIANT})..."
        sed -i "1s/${DISTRO}/${distro}/" debian/changelog
        sed -i "1s/${VERSION}/${VARIANT}/" debian/changelog
        debuild -S -sa -k"$GPG_KEY" -d
        echo ""

        STEP=$((STEP+1))
        echo "[$STEP/$TOTAL_STEPS] Uploading ${distro} to PPA..."
        dput "$PPA_TARGET" "../voice-to-text_${VARIANT}_source.changes"
        echo ""

        git checkout debian/changelog
    fi
done

# ═══════════════════════════════════════════════════════════════
# TAG + ARCHIVE
# ═══════════════════════════════════════════════════════════════

STEP=$((STEP+1))
echo "[$STEP/$TOTAL_STEPS] Tagging v${VERSION}..."
if git tag -l "v${VERSION}" | grep -q .; then
    echo "  Tag already exists, skipping"
else
    git tag -a "v${VERSION}" -m "Release ${VERSION}"
    git push origin "v${VERSION}"
    echo "  Tagged and pushed v${VERSION}"
fi
echo ""

STEP=$((STEP+1))
echo "[$STEP/$TOTAL_STEPS] Archiving build artifacts..."
mkdir -p build-archives
for f in ../voice-to-text_${VERSION}* ../voice-to-text_${VERSION}~*; do
    [ -f "$f" ] && mv -f "$f" build-archives/ 2>/dev/null || true
done
echo ""

echo "=== Done! v${VERSION} uploaded for ${DISTROS[*]} ==="
echo ""
echo "Monitor: https://launchpad.net/~powellclark/+archive/ubuntu/voice-to-text/+packages"
echo "Install: sudo apt update && sudo apt install voice-to-text"
echo ""
echo "After Launchpad builds (~15min), verify with:"
echo "  apt-cache policy voice-to-text"
