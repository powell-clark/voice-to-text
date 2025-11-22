#!/bin/bash
# Uninstaller for Voice to Text (Linux)
# Removes binaries, services, and optionally user data

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
INSTALL_BIN="$INSTALL_PREFIX/bin"
INSTALL_SHARE="$INSTALL_PREFIX/share/voice-to-text"
USER_DATA_DIR="$HOME/.local/share/voice-to-text"
USER_CACHE_DIR="$HOME/.cache/whisper"

echo -e "${BOLD}========================================"
echo "Voice to Text - Uninstaller"
echo "========================================${NC}"
echo ""

echo "This will remove:"
echo "  - Binaries from $INSTALL_BIN"
echo "  - Tools from $INSTALL_SHARE"
echo "  - Systemd service"
echo ""

# Ask about user data
REMOVE_USER_DATA=false
echo "User data locations:"
echo "  - Settings and logs: $USER_DATA_DIR"
echo "  - Whisper models: $USER_CACHE_DIR"
echo ""
read -p "Remove user data too? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    REMOVE_USER_DATA=true
    echo -e "${YELLOW}⚠ User data will be removed${NC}"
else
    echo "User data will be preserved"
fi

echo ""
read -p "Continue with uninstallation? (y/n) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Uninstallation cancelled"
    exit 0
fi

echo ""
echo -e "${BOLD}Uninstalling Voice to Text...${NC}"
echo ""

# Stop systemd service
if systemctl --user is-active --quiet vtt 2>/dev/null; then
    echo "Stopping systemd service..."
    systemctl --user stop vtt
    echo -e "${GREEN}✓ Service stopped${NC}"
fi

# Disable systemd service
if systemctl --user is-enabled --quiet vtt 2>/dev/null; then
    echo "Disabling systemd service..."
    systemctl --user disable vtt
    echo -e "${GREEN}✓ Service disabled${NC}"
fi

# Remove systemd service file
SYSTEMD_SERVICE="$HOME/.config/systemd/user/vtt.service"
if [ -f "$SYSTEMD_SERVICE" ]; then
    echo "Removing systemd service file..."
    rm -f "$SYSTEMD_SERVICE"
    systemctl --user daemon-reload
    echo -e "${GREEN}✓ Service file removed${NC}"
fi

# Remove binaries
echo "Removing binaries..."
sudo rm -f "$INSTALL_BIN/vtt-linux"
sudo rm -f "$INSTALL_BIN/vtt-transcribe.py"
echo -e "${GREEN}✓ Binaries removed${NC}"

# Remove tools and documentation
echo "Removing tools and documentation..."
sudo rm -rf "$INSTALL_SHARE"
echo -e "${GREEN}✓ Tools and documentation removed${NC}"

# Remove user data if requested
if [ "$REMOVE_USER_DATA" = true ]; then
    echo "Removing user data..."

    if [ -d "$USER_DATA_DIR" ]; then
        echo "  Removing $USER_DATA_DIR..."
        rm -rf "$USER_DATA_DIR"
    fi

    if [ -d "$USER_CACHE_DIR" ]; then
        CACHE_SIZE=$(du -sh "$USER_CACHE_DIR" 2>/dev/null | cut -f1)
        echo "  Whisper models ($CACHE_SIZE) at $USER_CACHE_DIR"
        read -p "  Remove Whisper models too? (y/n) " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$USER_CACHE_DIR"
            echo "  Removed Whisper models"
        else
            echo "  Kept Whisper models"
        fi
    fi

    echo -e "${GREEN}✓ User data removed${NC}"
else
    echo "User data preserved at:"
    echo "  - $USER_DATA_DIR"
    echo "  - $USER_CACHE_DIR"
fi

echo ""
echo -e "${BOLD}========================================${NC}"
echo -e "${GREEN}✓ Uninstallation complete${NC}"
echo -e "${BOLD}========================================${NC}"
echo ""

if [ "$REMOVE_USER_DATA" = false ]; then
    echo "User data was preserved. To remove manually:"
    echo "  rm -rf $USER_DATA_DIR"
    echo "  rm -rf $USER_CACHE_DIR"
    echo ""
fi

echo "Voice to Text has been uninstalled."
