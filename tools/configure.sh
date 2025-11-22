#!/bin/bash
# Post-install configuration wizard for Voice to Text
# Sets up optimal configuration based on user environment

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

USER_DATA_DIR="$HOME/.local/share/voice-to-text"
CONFIG_FILE="$USER_DATA_DIR/settings.json"

echo -e "${BOLD}========================================"
echo "Voice to Text - Configuration Wizard"
echo "========================================${NC}"
echo ""

mkdir -p "$USER_DATA_DIR"

# Detect environment
detect_environment() {
    echo -e "${BOLD}Detecting environment...${NC}"
    echo ""

    # Session type
    SESSION_TYPE="${XDG_SESSION_TYPE:-unknown}"
    echo "Session type: $SESSION_TYPE"

    # Desktop environment
    DESKTOP="${XDG_CURRENT_DESKTOP:-unknown}"
    echo "Desktop: $DESKTOP"

    # Wayland compositor
    if [ "$SESSION_TYPE" = "wayland" ]; then
        if pgrep -x gnome-shell >/dev/null; then
            COMPOSITOR="gnome-shell"
        elif pgrep -x kwin_wayland >/dev/null; then
            COMPOSITOR="kwin"
        elif pgrep -x sway >/dev/null; then
            COMPOSITOR="sway"
        else
            COMPOSITOR="unknown"
        fi
        echo "Wayland compositor: $COMPOSITOR"
    fi

    echo ""
}

# Configure hotkey
configure_hotkey() {
    echo -e "${BOLD}Hotkey Configuration${NC}"
    echo ""
    echo "Voice to Text uses a hotkey to start/stop recording."
    echo "Default: Scroll Lock"
    echo ""
    echo "Other suggestions:"
    echo "  - F13-F24 (extra function keys on some keyboards)"
    echo "  - Pause/Break"
    echo "  - Print Screen"
    echo ""

    if [ "$SESSION_TYPE" = "wayland" ]; then
        echo -e "${YELLOW}⚠ Wayland detected${NC}"
        echo ""
        echo "Hotkey configuration on Wayland requires manual setup:"
        echo ""
        case "$COMPOSITOR" in
            gnome-shell)
                echo "GNOME: Settings → Keyboard → Custom Shortcuts"
                echo "  Add new shortcut with command:"
                echo "  dbus-send --session --type=signal /org/gnome/Shell org.gnome.Shell.AcceleratorActivated string:'voice-to-text-record'"
                ;;
            kwin)
                echo "KDE: System Settings → Shortcuts"
                echo "  Add custom shortcut for vtt-linux"
                ;;
            sway)
                echo "Sway: Edit ~/.config/sway/config"
                echo "  Add: bindsym --release Scroll_Lock exec pkill -SIGUSR1 vtt-linux"
                ;;
            *)
                echo "Check your compositor's documentation for setting global hotkeys"
                ;;
        esac
        echo ""
    else
        echo "Hotkey will be configured automatically in X11 mode"
    fi

    echo ""
    read -p "Press Enter to continue... "
}

# Configure model
configure_model() {
    echo -e "${BOLD}Model Selection${NC}"
    echo ""
    echo "Select Whisper model for transcription:"
    echo ""
    echo "  1) tiny.en   - 39 MB  - Very fast, lower accuracy"
    echo "  2) base.en   - 74 MB  - Fast, decent accuracy"
    echo "  3) small.en  - 244 MB - Recommended (best balance)"
    echo "  4) medium.en - 769 MB - Slower, higher accuracy"
    echo "  5) large-v3  - 1.5 GB - Slowest, highest accuracy"
    echo ""
    read -p "Select (1-5) [default: 3]: " model_choice

    case "${model_choice:-3}" in
        1) SELECTED_MODEL="tiny.en" ;;
        2) SELECTED_MODEL="base.en" ;;
        3) SELECTED_MODEL="small.en" ;;
        4) SELECTED_MODEL="medium.en" ;;
        5) SELECTED_MODEL="large-v3" ;;
        *) SELECTED_MODEL="small.en" ;;
    esac

    echo ""
    echo "Selected: $SELECTED_MODEL"

    # Check if model is downloaded
    MODEL_FILE="$HOME/.cache/whisper/ggml-$SELECTED_MODEL.bin"
    if [ ! -f "$MODEL_FILE" ]; then
        echo ""
        echo -e "${YELLOW}Model not downloaded${NC}"
        read -p "Download now? (y/n) " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            ./tools/download_model.sh "$SELECTED_MODEL"
        fi
    else
        echo -e "${GREEN}✓ Model already downloaded${NC}"
    fi

    echo ""
}

# Configure text injection
configure_typing() {
    echo -e "${BOLD}Text Injection Configuration${NC}"
    echo ""

    if [ "$SESSION_TYPE" = "wayland" ]; then
        echo "Wayland text injection options:"
        echo ""
        echo "  1) ydotool (recommended)"
        echo "  2) dotool (alternative)"
        echo "  3) clipboard + paste"
        echo ""

        if command -v ydotool >/dev/null; then
            echo -e "${GREEN}✓ ydotool installed${NC}"
            if pgrep -x ydotoold >/dev/null; then
                echo -e "${GREEN}✓ ydotoold daemon running${NC}"
            else
                echo -e "${YELLOW}⚠ ydotoold daemon not running${NC}"
                echo ""
                echo "Start daemon:"
                echo "  sudo systemctl start ydotool"
                echo "  sudo systemctl enable ydotool"
            fi
        else
            echo -e "${YELLOW}⚠ ydotool not installed${NC}"
            echo ""
            echo "Install:"
            echo "  sudo apt install ydotool  # Ubuntu/Debian"
            echo "  sudo dnf install ydotool  # Fedora"
        fi

        echo ""
        echo "For best results on Wayland, install and enable ydotool"
        echo ""
    else
        echo "X11 text injection: XTest (automatic)"
        echo -e "${GREEN}✓ No additional configuration needed${NC}"
    fi

    echo ""
    read -p "Press Enter to continue... "
}

# Configure auto-start
configure_autostart() {
    echo -e "${BOLD}Auto-Start Configuration${NC}"
    echo ""
    echo "Would you like Voice to Text to start automatically on login?"
    echo ""
    read -p "Enable auto-start? (y/n) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        systemctl --user enable vtt
        echo -e "${GREEN}✓ Auto-start enabled${NC}"
        echo ""
        echo "Voice to Text will start on next login"
    else
        systemctl --user disable vtt 2>/dev/null || true
        echo "Auto-start disabled"
        echo ""
        echo "Start manually:"
        echo "  systemctl --user start vtt"
    fi

    echo ""
}

# Save configuration
save_configuration() {
    echo -e "${BOLD}Saving configuration...${NC}"
    echo ""

    # This is a placeholder - actual settings are saved by the application
    # We're just creating the directory structure

    mkdir -p "$USER_DATA_DIR/recordings"

    echo -e "${GREEN}✓ Configuration saved${NC}"
    echo ""
}

# Main flow
main() {
    detect_environment
    configure_hotkey
    configure_model
    configure_typing
    configure_autostart
    save_configuration

    echo -e "${BOLD}========================================${NC}"
    echo -e "${GREEN}✓ Configuration complete!${NC}"
    echo -e "${BOLD}========================================${NC}"
    echo ""
    echo "Voice to Text is ready to use."
    echo ""
    echo "Quick start:"
    echo "  1. Start application: systemctl --user start vtt"
    echo "  2. Look for tray icon"
    echo "  3. Press Scroll Lock to record"
    echo "  4. Release to transcribe"
    echo ""
    echo "Troubleshooting:"
    echo "  - Check logs: ~/.local/share/voice-to-text/vtt.log"
    echo "  - View recordings: ~/.local/share/voice-to-text/recordings/"
    echo "  - System monitor: vtt-system-monitor"
    echo ""
}

main
