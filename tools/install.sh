#!/bin/bash
# Universal installer for Voice to Text (Linux)
# Handles dependencies, builds, and system integration

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
INSTALL_BIN="$INSTALL_PREFIX/bin"
INSTALL_SHARE="$INSTALL_PREFIX/share/voice-to-text"

echo -e "${BOLD}========================================"
echo "Voice to Text - Installer"
echo "========================================${NC}"
echo ""

# Check if running as root (not recommended)
if [ "$EUID" -eq 0 ]; then
    echo -e "${YELLOW}⚠ WARNING: Running as root${NC}"
    echo "This installer should be run as a regular user with sudo access"
    echo ""
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Detect Linux distribution
DISTRO="unknown"
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO="$ID"
fi

echo "Detected distribution: $DISTRO"
echo "Install prefix: $INSTALL_PREFIX"
echo ""

# Install dependencies
install_dependencies() {
    echo -e "${BOLD}Installing system dependencies...${NC}"
    echo ""

    case "$DISTRO" in
        ubuntu|debian|pop|linuxmint)
            echo "Using apt package manager..."
            sudo apt update
            sudo apt install -y \
                build-essential \
                pkg-config \
                libportaudio2 \
                libportaudio-ocaml-dev \
                portaudio19-dev \
                libx11-dev \
                libxtst-dev \
                libxext-dev \
                libgtk-3-dev \
                libayatana-appindicator3-dev \
                libnotify-dev \
                libdbus-1-dev \
                python3 \
                python3-pip \
                python3-venv \
                alsa-utils \
                valgrind \
                wget \
                curl
            ;;
        fedora|rhel|centos)
            echo "Using dnf package manager..."
            sudo dnf install -y \
                gcc \
                make \
                pkg-config \
                portaudio-devel \
                libX11-devel \
                libXtst-devel \
                libXext-devel \
                gtk3-devel \
                libappindicator-gtk3-devel \
                libnotify-devel \
                dbus-devel \
                python3 \
                python3-pip \
                alsa-utils \
                valgrind \
                wget \
                curl
            ;;
        arch|manjaro)
            echo "Using pacman package manager..."
            sudo pacman -S --needed --noconfirm \
                base-devel \
                pkg-config \
                portaudio \
                libx11 \
                libxtst \
                libxext \
                gtk3 \
                libappindicator-gtk3 \
                libnotify \
                dbus \
                python \
                python-pip \
                alsa-utils \
                valgrind \
                wget \
                curl
            ;;
        *)
            echo -e "${YELLOW}⚠ Unknown distribution: $DISTRO${NC}"
            echo "Please install dependencies manually and re-run this script"
            echo ""
            echo "Required packages:"
            echo "  - build-essential (gcc, make)"
            echo "  - pkg-config"
            echo "  - portaudio development libraries"
            echo "  - X11 development libraries (libx11, libxtst, libxext)"
            echo "  - GTK3 development libraries"
            echo "  - libappindicator3 development libraries"
            echo "  - libnotify development libraries"
            echo "  - D-Bus development libraries"
            echo "  - Python 3.10+"
            echo ""
            read -p "Skip dependency installation? (y/n) " -n 1 -r
            echo ""
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                exit 1
            fi
            ;;
    esac

    echo -e "${GREEN}✓ Dependencies installed${NC}"
    echo ""
}

# Install Python packages
install_python_packages() {
    echo -e "${BOLD}Installing Python packages...${NC}"
    echo ""

    # Check if faster-whisper is already installed
    if python3 -c "import faster_whisper" 2>/dev/null; then
        echo -e "${GREEN}✓ faster-whisper already installed${NC}"
    else
        echo "Installing faster-whisper and ctranslate2..."
        pip3 install --user faster-whisper ctranslate2 || \
        pip3 install --break-system-packages faster-whisper ctranslate2
    fi

    echo -e "${GREEN}✓ Python packages installed${NC}"
    echo ""
}

# Build application
build_application() {
    echo -e "${BOLD}Building Voice to Text...${NC}"
    echo ""

    # Clean previous build
    make -f Makefile.linux clean 2>/dev/null || true

    # Build
    if make -f Makefile.linux; then
        echo -e "${GREEN}✓ Build successful${NC}"
    else
        echo -e "${RED}✗ Build failed${NC}"
        exit 1
    fi

    echo ""
}

# Install binaries
install_binaries() {
    echo -e "${BOLD}Installing binaries...${NC}"
    echo ""

    sudo install -D -m 755 vtt-linux "$INSTALL_BIN/vtt-linux"
    sudo install -D -m 755 src/common/transcribe.py "$INSTALL_BIN/vtt-transcribe.py"

    echo -e "${GREEN}✓ Binaries installed to $INSTALL_BIN${NC}"
    echo ""
}

# Install tools
install_tools() {
    echo -e "${BOLD}Installing developer tools...${NC}"
    echo ""

    sudo mkdir -p "$INSTALL_SHARE/tools"

    for tool in tools/*.sh tools/*.py; do
        if [ -f "$tool" ]; then
            sudo install -D -m 755 "$tool" "$INSTALL_SHARE/$tool"
        fi
    done

    echo -e "${GREEN}✓ Tools installed to $INSTALL_SHARE/tools${NC}"
    echo ""
}

# Install documentation
install_documentation() {
    echo -e "${BOLD}Installing documentation...${NC}"
    echo ""

    sudo mkdir -p "$INSTALL_SHARE/docs"

    for doc in README.md CLAUDE.md docs/*.md; do
        if [ -f "$doc" ]; then
            sudo install -D -m 644 "$doc" "$INSTALL_SHARE/$doc"
        fi
    done

    echo -e "${GREEN}✓ Documentation installed to $INSTALL_SHARE/docs${NC}"
    echo ""
}

# Create systemd user service
install_systemd_service() {
    echo -e "${BOLD}Creating systemd user service...${NC}"
    echo ""

    SYSTEMD_DIR="$HOME/.config/systemd/user"
    mkdir -p "$SYSTEMD_DIR"

    cat > "$SYSTEMD_DIR/vtt.service" <<EOF
[Unit]
Description=Voice to Text - Speech-to-text transcription service
After=graphical-session.target

[Service]
Type=simple
ExecStart=$INSTALL_BIN/vtt-linux
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF

    systemctl --user daemon-reload

    echo -e "${GREEN}✓ Systemd service created${NC}"
    echo ""
    echo "To enable auto-start:"
    echo "  systemctl --user enable vtt"
    echo ""
    echo "To start now:"
    echo "  systemctl --user start vtt"
    echo ""
}

# Download recommended model
download_model() {
    echo -e "${BOLD}Checking for Whisper models...${NC}"
    echo ""

    MODEL_DIR="$HOME/.cache/whisper"
    if [ -d "$MODEL_DIR" ] && [ "$(find "$MODEL_DIR" -name 'ggml-*.bin' | wc -l)" -gt 0 ]; then
        echo -e "${GREEN}✓ Models already installed${NC}"
        echo ""
        return
    fi

    echo "No models found. Downloading recommended model (small.en - 244MB)..."
    echo ""
    read -p "Download now? (y/n) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        ./tools/download_model.sh small.en
    else
        echo ""
        echo "You can download models later with:"
        echo "  $INSTALL_SHARE/tools/download_model.sh small.en"
    fi

    echo ""
}

# Main installation flow
main() {
    echo "This installer will:"
    echo "  1. Install system dependencies"
    echo "  2. Install Python packages"
    echo "  3. Build the application"
    echo "  4. Install binaries to $INSTALL_BIN"
    echo "  5. Create systemd service"
    echo "  6. Download Whisper model (optional)"
    echo ""
    read -p "Continue? (y/n) " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled"
        exit 0
    fi

    install_dependencies
    install_python_packages
    build_application
    install_binaries
    install_tools
    install_documentation
    install_systemd_service
    download_model

    echo -e "${BOLD}========================================${NC}"
    echo -e "${GREEN}✓ Installation complete!${NC}"
    echo -e "${BOLD}========================================${NC}"
    echo ""
    echo "Next steps:"
    echo ""
    echo "  1. Start the application:"
    echo "     $INSTALL_BIN/vtt-linux"
    echo ""
    echo "  2. Or enable auto-start:"
    echo "     systemctl --user enable --now vtt"
    echo ""
    echo "  3. Run first-time setup:"
    echo "     $INSTALL_SHARE/tools/first_run_wizard.sh"
    echo ""
    echo "  4. View documentation:"
    echo "     $INSTALL_SHARE/docs/README.md"
    echo ""
    echo "Enjoy using Voice to Text!"
}

main
