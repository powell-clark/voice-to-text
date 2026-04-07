#!/bin/bash
set -e

# Voice-to-Text PPA Release Script
# Usage: ./scripts/release-ppa.sh
#
# Reads version from debian/changelog, builds, signs, and uploads
# to ppa:powellclark/voice-to-text for both Noble and Jammy.

cd "$(dirname "$0")/.."

# Extract version from debian/changelog
VERSION=$(head -1 debian/changelog | grep -oP '\(.*?\)' | tr -d '()')
DISTRO=$(head -1 debian/changelog | awk '{print $3}' | tr -d ';')

if [ -z "$VERSION" ]; then
    echo "ERROR: Could not read version from debian/changelog"
    exit 1
fi

echo "=== VTT PPA Release v${VERSION} ==="
echo ""

# Step 1: Build
echo "[1/6] Building..."
make -f Makefile.linux clean
make -f Makefile.linux
echo ""

# Step 2: Build Noble source package
echo "[2/6] Building Noble source package..."
debuild -S -sa -k"emmanuel@powellclark.com"
echo ""

# Step 3: Upload Noble
echo "[3/6] Uploading Noble to PPA..."
dput powellclark-voice-to-text "../voice-to-text_${VERSION}_source.changes"
echo ""

# Step 4: Build Jammy variant
JAMMY_VERSION="${VERSION}~jammy1"
echo "[4/6] Building Jammy source package (${JAMMY_VERSION})..."
sed -i "1s/${DISTRO}/jammy/" debian/changelog
sed -i "1s/${VERSION}/${JAMMY_VERSION}/" debian/changelog
debuild -S -sa -k"emmanuel@powellclark.com"
echo ""

# Step 5: Upload Jammy
echo "[5/6] Uploading Jammy to PPA..."
dput powellclark-voice-to-text "../voice-to-text_${JAMMY_VERSION}_source.changes"
echo ""

# Step 6: Restore changelog
echo "[6/6] Restoring changelog..."
git checkout debian/changelog
echo ""

echo "=== Done! v${VERSION} uploaded for Noble and Jammy ==="
echo "Monitor builds: https://launchpad.net/~powellclark/+archive/ubuntu/voice-to-text/+packages"
