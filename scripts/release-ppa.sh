#!/bin/bash
set -eo pipefail

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

# Pinned mirror — DO NOT use archive.ubuntu.com or any gb.archive.ubuntu.com
# round-robin. Those CDNs rotate through backends; some serve devel/resolute
# under the "noble" name, which silently corrupts the chroot and fails the
# pbuilder gate with baffling errors (base-passwd postinst mktemp, satisfydepends
# failures). Bytemark is a single UK server, no round-robin, reliably serving
# pinned releases. Override with VTT_MIRROR=... if you know what you're doing.
PBUILDER_MIRROR="${VTT_MIRROR:-http://mirror.bytemark.co.uk/ubuntu}"

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

DIRTY=$(git diff --name-only HEAD -- src/ debian/ Cargo.toml packaging/linux/vtt.service packaging/linux/vtt.prebuilt scripts/ | head -20)
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
# PRE-FLIGHT — build fresh binary and stage as vtt.prebuilt
# ═══════════════════════════════════════════════════════════════
#
# Since 2.0.2, PPA releases ship the pre-built binary (Ubuntu LTS
# cargo 1.75 cannot build the modern Rust dep tree). Build the binary
# locally with rustup 1.91 and copy it to packaging/linux/vtt.prebuilt
# which debian/rules will install verbatim.

echo "[1/${STEP_TOTAL:-?}] Building release binary with local rustup..."
cargo build --release --offline 2>&1 | tail -5
if [ ! -x target/release/vtt ]; then
    echo "  FAIL — build did not produce target/release/vtt"
    exit 1
fi
cp target/release/vtt packaging/linux/vtt.prebuilt
echo "  OK — packaging/linux/vtt.prebuilt at $(du -h packaging/linux/vtt.prebuilt | cut -f1)"

# Pin Cargo.lock to v3 format. rustup ≥ 1.78 writes v4 by default which
# Ubuntu Noble cargo 1.75 cannot parse. debian/rules currently doesn't
# parse the lockfile (ships prebuilt) but we keep v3 defensively so any
# future source-build Noble path (or a user running cargo --locked on
# Noble) doesn't break. Matches the release-local.sh guard.
if grep -q '^version = 4$' Cargo.lock; then
    sed -i 's/^version = 4$/version = 3/' Cargo.lock
    echo "  NOTE: downgraded Cargo.lock v4 -> v3 for Noble compat"
fi

# Commit the prebuilt if it changed so git reflects what ships on PPA.
# Without this, every release leaves the working tree dirty and the tracked
# binary drifts behind the published one by a release. Use --allow-empty in
# case the binary is byte-identical (unlikely but possible with SOURCE_DATE_EPOCH).
if ! git diff --quiet packaging/linux/vtt.prebuilt; then
    git add packaging/linux/vtt.prebuilt
    git commit -m "build: refresh packaging/linux/vtt.prebuilt for v${VERSION} release" 2>&1 | tail -2
    git push origin main 2>&1 | tail -2
fi
echo ""

# ═══════════════════════════════════════════════════════════════
# HARD GATE — pbuilder chroot test matching Launchpad EXACTLY
# ═══════════════════════════════════════════════════════════════
#
# This is the mechanical guard that prevents tonight's 2.0.0/2.0.1/2.0.2
# failures from ever recurring. pbuilder creates a clean chroot of the
# target distro (noble, jammy) and runs the EXACT build Launchpad will
# run. If this fails locally, Launchpad fails too — and we never dput.
#
# One-time setup (sudo required) — use bytemark, NEVER archive.ubuntu.com:
#   sudo pbuilder --create --distribution noble \
#       --basetgz /var/cache/pbuilder/noble-base.tgz \
#       --mirror http://mirror.bytemark.co.uk/ubuntu \
#       --components "main restricted universe multiverse" \
#       --debootstrapopts --variant=buildd
#   sudo pbuilder --create --distribution jammy \
#       --basetgz /var/cache/pbuilder/jammy-base.tgz \
#       --mirror http://mirror.bytemark.co.uk/ubuntu \
#       --components "main restricted universe multiverse" \
#       --debootstrapopts --variant=buildd
#
# The archive.ubuntu.com (and gb.archive.ubuntu.com) mirrors are round-robin
# CDNs — some backends serve devel/resolute content under pinned release names,
# silently corrupting the chroot. Bytemark is a single UK server, pinned, and
# has not lied about what release it's serving in 20+ years of operation.
#
# Bypass with VTT_SKIP_PBUILDER=1 only if you truly know what you're
# doing (e.g. repeating a known-good release on a machine without
# pbuilder set up).

check_pbuilder() {
    local distro="$1"
    local basetgz="/var/cache/pbuilder/${distro}-base.tgz"
    if [ ! -f "$basetgz" ]; then
        echo "  MISSING chroot: $basetgz"
        echo ""
        echo "One-time setup required. Run once per distro:"
        echo "  sudo pbuilder --create --distribution ${distro} \\"
        echo "      --basetgz $basetgz \\"
        echo "      --mirror ${PBUILDER_MIRROR} \\"
        echo "      --components \"main restricted universe multiverse\" \\"
        echo "      --debootstrapopts --variant=buildd"
        echo ""
        echo "NOTE: we pin the mirror to bytemark (single server) because"
        echo "archive.ubuntu.com round-robin sometimes serves devel/resolute"
        echo "content, silently corrupting the chroot. See config block for detail."
        return 1
    fi
    return 0
}

build_in_chroot() {
    local distro="$1"
    local dsc="$2"
    local basetgz="/var/cache/pbuilder/${distro}-base.tgz"
    local logfile="/tmp/vtt-pbuilder-${distro}.log"
    echo "[pre-flight] pbuilder --build ($distro) — simulates Launchpad exactly..."
    # Use apt-based satisfydepends instead of the default aptitude-based one.
    # Must go via --configfile because /usr/share/pbuilder/pbuilderrc
    # hard-sets PBUILDERSATISFYDEPENDSCMD without checking for an override,
    # so an env var alone is clobbered. Our override file is sourced AFTER
    # the system default and takes precedence.
    #
    # The aptitude variant builds a dummy .deb via dpkg-deb and fails with
    #   "dpkg-deb: error: failed to make temporary file (control member)"
    # on dpkg ≥ 1.22. Apt variant installs deps via apt-get directly.
    local cfg="$(dirname "$0")/pbuilderrc-override"
    # Also unset TMPDIR — user's shell sets it to /tmp/user/$UID (systemd
    # private tmp), which leaks through sudo into the chroot where no such
    # directory exists, so dpkg-deb and mktemp fail with ENOENT. Clearing
    # TMPDIR lets everything default to plain /tmp.
    if sudo -n env -u TMPDIR pbuilder --build \
            --configfile "$cfg" \
            --distribution "$distro" \
            --basetgz "$basetgz" \
            --buildresult /tmp/vtt-pbuilder-${distro} \
            "$dsc" > "$logfile" 2>&1; then
        tail -20 "$logfile"
        echo "  OK — $distro chroot build succeeded."
        return 0
    else
        tail -30 "$logfile"
        echo ""
        echo "  FAIL — $distro chroot build failed. Launchpad would fail too."
        echo "  Full log: $logfile"
        echo "  Fix the error shown above, bump version, re-run this script."
        return 1
    fi
}

if [ "${VTT_SKIP_PBUILDER:-0}" = "1" ]; then
    echo "[pre-flight] WARNING — VTT_SKIP_PBUILDER=1 set; skipping chroot tests."
    echo "  This bypasses tonight's hard-learned lesson. Do not do this habitually."
else
    echo "[2/${STEP_TOTAL:-?}] Pre-flight chroot tests (noble + jammy) — HARD GATE"
    for distro in "${DISTROS[@]}"; do
        if ! check_pbuilder "$distro"; then
            exit 1
        fi
    done
    # Require cached sudo up-front. Without this the chroot build silently
    # fails the password prompt while the script proceeds — the exact bug
    # that let 2.0.4 dput without the gate running.
    echo "  Priming sudo (will prompt once if not cached)..."
    sudo -v || { echo "  FAIL — sudo required for pbuilder chroot."; exit 1; }
    ( while true; do sudo -n true; sleep 60; kill -0 "$$" 2>/dev/null || exit; done ) &
    SUDO_KEEPALIVE_PID=$!
    trap 'kill "$SUDO_KEEPALIVE_PID" 2>/dev/null || true' EXIT
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
    echo "  2. cargo build --offline --locked (PASSED above — Launchpad will build)"
    for distro in "${DISTROS[@]}"; do
        if [ "$distro" = "$DISTRO" ]; then
            echo "  3. debuild -S -sa -k$GPG_KEY (${distro})"
            echo "  4. dput $PPA_TARGET voice-to-text_${VERSION}_source.changes"
        else
            suffix="~${distro}1"
            echo "  3. debuild -S -sa -k$GPG_KEY (${distro}, ${VERSION}${suffix})"
            echo "  4. dput $PPA_TARGET voice-to-text_${VERSION}${suffix}_source.changes"
        fi
    done
    echo "  5. git tag v${VERSION} && git push origin v${VERSION}"
    echo "  6. Archive artifacts to build-archives/"
    exit 0
fi

# ═══════════════════════════════════════════════════════════════
# BUILD SOURCE PACKAGES + UPLOAD
# ═══════════════════════════════════════════════════════════════

TOTAL_STEPS=$(( 1 + ${#DISTROS[@]} * 3 + 3 ))
STEP=1

FIRST=true
for distro in "${DISTROS[@]}"; do
    if [ "$FIRST" = true ]; then
        CHANGES_FILE="../voice-to-text_${VERSION}_source.changes"
        DSC_FILE="../voice-to-text_${VERSION}.dsc"
        TARGET_VERSION="${VERSION}"
    else
        VARIANT="${VERSION}~${distro}1"
        CHANGES_FILE="../voice-to-text_${VARIANT}_source.changes"
        DSC_FILE="../voice-to-text_${VARIANT}.dsc"
        TARGET_VERSION="${VARIANT}"
        sed -i "1s/${DISTRO}/${distro}/" debian/changelog
        sed -i "1s/${VERSION}/${VARIANT}/" debian/changelog
    fi

    STEP=$((STEP+1))
    echo "[$STEP/$TOTAL_STEPS] Building ${distro} source package (${TARGET_VERSION})..."
    debuild -S -sa -k"$GPG_KEY" -d
    echo ""

    # HARD GATE — pbuilder chroot build matching Launchpad exactly.
    # If this fails, dput NEVER happens. No exceptions.
    if [ "${VTT_SKIP_PBUILDER:-0}" != "1" ]; then
        STEP=$((STEP+1))
        echo "[$STEP/$TOTAL_STEPS] Pre-flight chroot build (${distro}) — hard gate..."
        if ! build_in_chroot "$distro" "$DSC_FILE"; then
            echo ""
            echo "================================================================"
            echo "REFUSING TO UPLOAD v${TARGET_VERSION} to ${distro}."
            echo ""
            echo "The same build that would run on Launchpad failed locally."
            echo "Uploading now would publish broken software and dent the PPA."
            echo ""
            echo "Fix the error, bump version, re-run this script."
            echo "================================================================"
            [ "$FIRST" != true ] && git checkout debian/changelog
            exit 1
        fi
        echo ""
    fi

    STEP=$((STEP+1))
    echo "[$STEP/$TOTAL_STEPS] Uploading ${distro} to PPA..."
    dput "$PPA_TARGET" "$CHANGES_FILE"
    echo ""

    if [ "$FIRST" = true ]; then
        FIRST=false
    else
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

STEP=$((STEP+1))
echo "[$STEP/$TOTAL_STEPS] Recording Launchpad build timings..."
# Best-effort: the build record exists within seconds of the upload being
# accepted, but the binary is not published for a long while yet, so this
# first pass writes 'pending' availability. Re-run the script after the
# binary lands to fill it in (TASK-VTT134).
bash scripts/record-ppa-times.sh "$VERSION" || \
    echo "  timings not recorded — re-run scripts/record-ppa-times.sh $VERSION later"
echo ""

echo "=== Done! v${VERSION} uploaded for ${DISTROS[*]} ==="
echo ""
echo "Monitor: https://launchpad.net/~powellclark/+archive/ubuntu/voice-to-text/+packages"
echo "Install: sudo apt update && sudo apt install voice-to-text"
echo ""
echo "After Launchpad builds (~15min), verify with:"
echo "  apt-cache policy voice-to-text"
echo ""
echo "Then record when it actually became installable:"
echo "  bash scripts/record-ppa-times.sh ${VERSION}"
