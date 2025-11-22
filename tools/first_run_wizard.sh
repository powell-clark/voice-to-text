#!/bin/bash
# First-Run Setup Wizard for Voice to Text
# Checks system requirements, tests microphone, verifies installation

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=========================================="
echo "Voice to Text - First Run Setup Wizard"
echo "=========================================="
echo ""

# Check if running on Linux
if [ "$(uname)" != "Linux" ]; then
    echo -e "${RED}ERROR: This wizard is for Linux only${NC}"
    exit 1
fi

# Check if X11 is running
if [ -z "$DISPLAY" ]; then
    echo -e "${RED}ERROR: X11 display not found${NC}"
    echo "Voice to Text requires X11 (Wayland support coming soon)"
    exit 1
fi

echo -e "${GREEN}✓${NC} X11 display detected: $DISPLAY"

# Check session type
SESSION_TYPE="${XDG_SESSION_TYPE:-unknown}"
if [ "$SESSION_TYPE" = "wayland" ]; then
    echo -e "${YELLOW}⚠${NC} Wayland session detected - X11 compatibility mode required"
    echo "  Note: Native Wayland support is not yet available"
else
    echo -e "${GREEN}✓${NC} Session type: $SESSION_TYPE"
fi

# Check for required commands
echo ""
echo "Checking system dependencies..."

DEPS_OK=true

check_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1 found"
    else
        echo -e "${RED}✗${NC} $1 not found"
        DEPS_OK=false
    fi
}

check_command "python3"
check_command "pactl"
check_command "arecord"

# Check Python version
PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
PYTHON_MAJOR=$(echo "$PYTHON_VERSION" | cut -d. -f1)
PYTHON_MINOR=$(echo "$PYTHON_VERSION" | cut -d. -f2)

if [ "$PYTHON_MAJOR" -eq 3 ] && [ "$PYTHON_MINOR" -ge 10 ]; then
    echo -e "${GREEN}✓${NC} Python $PYTHON_VERSION (>= 3.10 required)"
else
    echo -e "${RED}✗${NC} Python $PYTHON_VERSION (>= 3.10 required)"
    DEPS_OK=false
fi

# Check Python packages
echo ""
echo "Checking Python packages..."

check_python_package() {
    if python3 -c "import $1" 2>/dev/null; then
        echo -e "${GREEN}✓${NC} $1 installed"
    else
        echo -e "${YELLOW}⚠${NC} $1 not installed (install with: pip3 install --break-system-packages $1)"
    fi
}

check_python_package "faster_whisper"
check_python_package "ctranslate2"

# Check for vtt-linux binary
echo ""
echo "Checking Voice to Text installation..."

if command -v vtt-linux &> /dev/null; then
    echo -e "${GREEN}✓${NC} vtt-linux found: $(which vtt-linux)"
elif [ -f "./vtt-linux" ]; then
    echo -e "${GREEN}✓${NC} vtt-linux found: ./vtt-linux (local build)"
else
    echo -e "${RED}✗${NC} vtt-linux not found"
    echo "  Build with: make -f Makefile.linux"
    DEPS_OK=false
fi

# Test microphone
echo ""
echo "=========================================="
echo "Microphone Test"
echo "=========================================="
echo ""
echo "Testing microphone access..."

# List audio devices
echo ""
echo "Available input devices:"
pactl list sources short || echo "Failed to list audio sources"

echo ""
read -p "Test your microphone? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    TEST_FILE="/tmp/vtt-mic-test.wav"

    echo "Recording 3 seconds of audio..."
    echo "Speak now!"

    if arecord -d 3 -f S16_LE -r 16000 -c 1 "$TEST_FILE" 2>/dev/null; then
        echo -e "${GREEN}✓${NC} Recording successful"

        echo ""
        read -p "Play back recording? (y/n) " -n 1 -r
        echo ""

        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "Playing back..."
            aplay "$TEST_FILE" 2>/dev/null || echo "Playback failed"
        fi

        # Check amplitude
        MAX_AMP=$(ffmpeg -i "$TEST_FILE" -af "volumedetect" -f null - 2>&1 | grep max_volume | awk '{print $5}')
        echo "Max volume: $MAX_AMP dB"

        if [ -n "$MAX_AMP" ]; then
            AMP_NUM=$(echo "$MAX_AMP" | tr -d '-' | cut -d. -f1)
            if [ "$AMP_NUM" -lt 20 ]; then
                echo -e "${GREEN}✓${NC} Audio level good"
            else
                echo -e "${YELLOW}⚠${NC} Audio level low - speak louder or adjust mic gain"
            fi
        fi

        rm -f "$TEST_FILE"
    else
        echo -e "${RED}✗${NC} Recording failed"
        echo "Check microphone permissions and connections"
        DEPS_OK=false
    fi
fi

# Summary
echo ""
echo "=========================================="
echo "Setup Summary"
echo "=========================================="
echo ""

if [ "$DEPS_OK" = true ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo ""
    echo "You're ready to use Voice to Text."
    echo ""
    echo "Start the application:"
    echo "  systemctl --user start vtt"
    echo ""
    echo "Or run manually:"
    echo "  ./vtt-linux"
    echo ""
else
    echo -e "${RED}✗ Some checks failed${NC}"
    echo ""
    echo "Please install missing dependencies and re-run this wizard."
    exit 1
fi
