#!/bin/bash
set -e

# Install script that preserves permissions during development

APP_NAME="VTT.app"
SOURCE="$PWD/$APP_NAME"
DEST="/Applications/$APP_NAME"
BUNDLE_ID="com.powellclark.voice-to-text"

# Parse arguments
RESET_PERMS=false
if [ "$1" == "--reset-permissions" ] || [ "$1" == "-r" ]; then
    RESET_PERMS=true
fi

echo "📦 Installing VTT to /Applications..."

# Kill running instance
killall VTT 2>/dev/null || true
sleep 1

# Reset permissions if requested
if [ "$RESET_PERMS" = true ]; then
    echo "🔄 Resetting system permissions..."
    tccutil reset Microphone "$BUNDLE_ID" 2>/dev/null || echo "  ℹ️  Microphone permission not found"
    tccutil reset Accessibility "$BUNDLE_ID" 2>/dev/null || echo "  ℹ️  Accessibility permission not found"
    tccutil reset ListenEvent "$BUNDLE_ID" 2>/dev/null || echo "  ℹ️  Input Monitoring permission not found"
    echo "  ✅ Permissions reset"
    sleep 1
fi

# Always do a fresh install to reset permissions
if [ -d "$DEST" ]; then
    echo "🗑️  Removing existing app to reset permissions..."
    rm -rf "$DEST"
fi

echo "📦 Installing fresh app..."
cp -R "$SOURCE" "$DEST"
echo "⚠️  Permissions will need to be granted on launch"

echo "🚀 Launching VTT..."
open "$DEST"

echo ""
echo "✅ Done!"
echo ""
echo "💡 Tips:"
echo "  • To preserve permissions: ./install-dev.sh"
echo "  • To reset permissions:    ./install-dev.sh --reset-permissions"
echo "  • Check logs:              tail -f /tmp/VTT/vtt.log"
echo ""
