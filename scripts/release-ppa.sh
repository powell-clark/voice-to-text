#!/bin/bash
set -e

# Voice-to-Text PPA Release Script
# Usage: ./scripts/release-ppa.sh [--force] [--dry-run]
#
# Pre-flight checks, builds, signs, uploads to PPA for all supported
# Ubuntu releases, tags the release, and archives artifacts.

cd "$(dirname "$0")/.."

# ═══════════════════════════════════════════════════════════════
# CONFIG
# ═══════════════════════════════════════════════════════════════

GPG_KEY="${VTT_GPG_KEY:-emmanuel@powellclark.com}"
PPA_TARGET="${VTT_PPA_TARGET:-powellclark-voice-to-text}"
DISTROS=("noble" "jammy")  # Add new LTS releases here

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
# VERSION VALIDATION
# ═══════════════════════════════════════════════════════════════

VERSION=$(head -1 debian/changelog | grep -oP '\(.*?\)' | tr -d '()')
DISTRO=$(head -1 debian/changelog | awk '{print $3}' | tr -d ';')

if [ -z "$VERSION" ]; then
    echo "ERROR: Could not read version from debian/changelog"
    exit 1
fi

# Check version is newer than the last tag
LAST_TAG=$(git tag -l 'v*' --sort=-v:refname | head -1 | sed 's/^v//')
if [ -n "$LAST_TAG" ]; then
    if dpkg --compare-versions "$VERSION" le "$LAST_TAG" 2>/dev/null; then
        if [ "$FORCE" = false ]; then
            echo "ERROR: Version $VERSION is not newer than last tag v$LAST_TAG"
            echo "Update debian/changelog with a higher version first."
            exit 1
        else
            echo "WARNING: Version $VERSION is not newer than last tag v$LAST_TAG (--force)"
        fi
    fi
    echo "  Last release: v$LAST_TAG"
fi

# Check tag doesn't already exist
if git tag -l "v${VERSION}" | grep -q .; then
    if [ "$FORCE" = false ]; then
        echo "ERROR: Tag v${VERSION} already exists. Already released?"
        exit 1
    else
        echo "WARNING: Tag v${VERSION} already exists (--force)"
    fi
fi

# Check changelog has actual content (not just version header)
CHANGELOG_LINES=$(sed -n '2,/^ --/p' debian/changelog | grep -c '^\s\+\*' || true)
if [ "$CHANGELOG_LINES" -eq 0 ] && [ "$FORCE" = false ]; then
    echo "ERROR: No changelog entries found for v${VERSION}."
    echo "Add bullet points to debian/changelog first."
    exit 1
fi

echo "  Version: $VERSION ($CHANGELOG_LINES changelog entries)"
echo "  Distros: ${DISTROS[*]}"
echo ""

# ═══════════════════════════════════════════════════════════════
# DRY RUN
# ═══════════════════════════════════════════════════════════════

if [ "$DRY_RUN" = true ]; then
    echo "=== DRY RUN — would release v${VERSION} ==="
    echo ""
    echo "Changelog:"
    sed -n '1,/^ --/p' debian/changelog
    echo ""
    echo "Steps:"
    echo "  1. make -f Makefile.linux clean && make -f Makefile.linux"
    for distro in "${DISTROS[@]}"; do
        if [ "$distro" = "$DISTRO" ]; then
            echo "  2. debuild -S -sa -k\"$GPG_KEY\" (${distro})"
            echo "  3. dput $PPA_TARGET voice-to-text_${VERSION}_source.changes"
        else
            suffix="~${distro}1"
            echo "  2. debuild -S -sa -k\"$GPG_KEY\" (${distro}, ${VERSION}${suffix})"
            echo "  3. dput $PPA_TARGET voice-to-text_${VERSION}${suffix}_source.changes"
        fi
    done
    echo "  4. git tag v${VERSION} && git push origin v${VERSION}"
    echo "  5. Archive artifacts to build-archives/"
    echo ""
    echo "Run without --dry-run to execute."
    exit 0
fi

# ═══════════════════════════════════════════════════════════════
# BUILD
# ═══════════════════════════════════════════════════════════════

TOTAL_STEPS=$(( 2 + ${#DISTROS[@]} * 2 + 2 ))
STEP=0

STEP=$((STEP+1))
echo "[$STEP/$TOTAL_STEPS] Building..."
make -f Makefile.linux clean
make -f Makefile.linux
echo ""

# ═══════════════════════════════════════════════════════════════
# UPLOAD EACH DISTRO
# ═══════════════════════════════════════════════════════════════

FIRST=true
for distro in "${DISTROS[@]}"; do
    if [ "$FIRST" = true ]; then
        # Primary distro — use changelog as-is
        STEP=$((STEP+1))
        echo "[$STEP/$TOTAL_STEPS] Building ${distro} source package..."
        debuild -S -sa -k"$GPG_KEY"
        echo ""

        STEP=$((STEP+1))
        echo "[$STEP/$TOTAL_STEPS] Uploading ${distro} to PPA..."
        dput "$PPA_TARGET" "../voice-to-text_${VERSION}_source.changes"
        echo ""

        FIRST=false
    else
        # Secondary distros — patch changelog
        VARIANT="${VERSION}~${distro}1"
        STEP=$((STEP+1))
        echo "[$STEP/$TOTAL_STEPS] Building ${distro} source package (${VARIANT})..."
        sed -i "1s/${DISTRO}/jammy/" debian/changelog
        sed -i "1s/${VERSION}/${VARIANT}/" debian/changelog
        debuild -S -sa -k"$GPG_KEY"
        echo ""

        STEP=$((STEP+1))
        echo "[$STEP/$TOTAL_STEPS] Uploading ${distro} to PPA..."
        dput "$PPA_TARGET" "../voice-to-text_${VARIANT}_source.changes"
        echo ""

        # Restore for next iteration
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
