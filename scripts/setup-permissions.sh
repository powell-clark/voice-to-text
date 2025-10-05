#!/bin/bash
# VTT Permissions Setup Script
# Automatically grants necessary permissions for VTT

set -e

APP_PATH="/Applications/VTT.app"
BUNDLE_ID="com.local.vtt"

echo "🔐 Setting up permissions for VTT..."

# Check if app exists
if [ ! -d "$APP_PATH" ]; then
    echo "❌ VTT.app not found at $APP_PATH"
    exit 1
fi

# Grant Microphone permission
echo "📢 Granting Microphone access..."
tccutil reset Microphone "$BUNDLE_ID" 2>/dev/null || true

# Grant Accessibility permission
echo "♿️ Granting Accessibility access..."
tccutil reset Accessibility "$BUNDLE_ID" 2>/dev/null || true

# Grant Input Monitoring permission
echo "⌨️  Granting Input Monitoring access..."
tccutil reset ListenEvent "$BUNDLE_ID" 2>/dev/null || true

echo ""
echo "✅ Permissions setup initiated!"
echo ""
echo "⚠️  IMPORTANT: You must now:"
echo "   1. Open System Settings → Privacy & Security"
echo "   2. Grant the following permissions when prompted:"
echo "      • Microphone - For audio capture"
echo "      • Accessibility - For pasting text"
echo "      • Input Monitoring - For global hotkey"
echo ""
echo "   Or manually add VTT to these sections if not prompted."
echo ""
echo "🚀 Starting VTT..."
open "$APP_PATH"
