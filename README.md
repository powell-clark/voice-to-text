# Voice to Text

Voice-to-text using OpenAI Whisper. Hold a hotkey to speak, release when finished. Your text appears in any focused input box.

**Available for macOS and Linux**

## How It Works

1. Hold the hotkey (**Scroll Lock** on Linux, **Right Alt** on macOS)
2. Speak
3. Release when finished
4. Text appears in your focused input box

**No internet required** - all transcription happens locally on your device.

## Recording Limits & Notifications

To prevent accidental long recordings, Voice to Text has maximum recording durations:
- **Linux:** 300 seconds (5 minutes)
- **macOS:** 10 seconds (for testing)

When you reach the recording limit:
- **Desktop notification** appears immediately alerting you
- Recording automatically stops when you release the key
- Transcription includes **[Truncated - 300s limit]** prefix so you know it was cut off
- You can continue holding the key, but no additional audio is captured

This ensures you don't accidentally leave the microphone open and provides clear feedback when transcriptions are incomplete.

## Installation

### macOS

```bash
brew tap powell-clark/voice-to-text
brew install --cask voice-to-text
```

Voice to Text automatically installs to /Applications.

### Linux (Ubuntu/Debian)

```bash
# Install dependencies
sudo apt install build-essential pkg-config \
    portaudio19-dev libx11-dev libxtst-dev libxext-dev \
    libgtk-3-dev libayatana-appindicator3-dev libnotify-dev \
    python3.12 python3-pip

# Install faster-whisper for transcription
python3.12 -m pip install --break-system-packages faster-whisper ctranslate2

# Clone and build
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text
make -f Makefile.linux

# Set up systemd service (optional, for auto-start)
mkdir -p ~/.config/systemd/user
cp vtt.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable vtt
systemctl --user start vtt
```

#### GPU Acceleration (Optional, Recommended for NVIDIA GPUs)

For 5-10x faster transcription with NVIDIA GPUs:

```bash
# Check if you have an NVIDIA GPU
nvidia-smi

# Install CUDA 12.6 and cuDNN 9
cd /tmp
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2404/x86_64/cuda-keyring_1.1-1_all.deb
sudo dpkg -i cuda-keyring_1.1-1_all.deb
sudo apt-get update
sudo apt-get install -y cuda-toolkit-12-6 libcudnn9-cuda-12 libcudnn9-dev-cuda-12

# Add CUDA to PATH
echo 'export PATH=/usr/local/cuda-12.6/bin:$PATH' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/cuda-12.6/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
source ~/.bashrc

# Verify installation
nvcc --version
python3.12 -c "import ctranslate2; print(f'CUDA devices: {ctranslate2.get_cuda_device_count()}')"

# Restart VTT to use GPU
systemctl --user restart vtt
```

**GPU Requirements:**
- NVIDIA GPU with CUDA support (Compute Capability 7.0+)
- Ubuntu 24.04 or compatible
- ~2GB GPU memory for small model, ~4GB for large models

**Without GPU:** The app will automatically use CPU with INT8 quantization (slower but still works).

### Build from Source

#### macOS
```bash
# Clone repository
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text

# Build with embedded whisper
make vendor-whisper  # Download whisper.cpp
make whisper-lib     # Build whisper library
make complete        # Build VTT.app

# Install
cp -R VTT.app /Applications/
open /Applications/VTT.app
```

#### Linux
```bash
# See Installation section above for dependencies

# Clone and build
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text
make -f Makefile.linux

# Install (optional)
sudo make -f Makefile.linux install
```

## Managing VTT (Linux)

After installation, VTT runs as a systemd user service. Use these commands:

```bash
# Start VTT
systemctl --user start vtt

# Stop VTT
systemctl --user stop vtt

# Restart VTT
systemctl --user restart vtt

# Check status
systemctl --user status vtt

# View logs
journalctl --user -u vtt -f

# Enable auto-start on login
systemctl --user enable vtt

# Disable auto-start
systemctl --user disable vtt
```

You can also add these convenient aliases to your `~/.bashrc`:

```bash
alias vtt-start='systemctl --user start vtt'
alias vtt-stop='systemctl --user stop vtt'
alias vtt-restart='systemctl --user restart vtt'
alias vtt-status='systemctl --user status vtt'
alias vtt-log='less +G ~/.local/share/voice-to-text/vtt.log'
alias vtt-log-tail='tail -f ~/.local/share/voice-to-text/vtt.log'
```

## First Run Setup

### macOS

**IMPORTANT:** Grant these three permissions in System Settings → Privacy & Security or the app won't work:
- **Microphone** - To record your voice
- **Accessibility** - To paste text
- **Input Monitoring** - To detect Right Alt key

The app will prompt you on first launch.

### Linux

No special permissions required. The app uses X11 for keyboard monitoring and XTest for text input. If running under Wayland, you may need to use XWayland compatibility mode.

## Choose a Model

Select a model from the menu bar. **We recommend starting with small** for a good balance of speed and accuracy.

### macOS - Two Backends Available:
- **W models** - whisper.cpp (C++, no Python required, slower)
- **CT2 models** - CTranslate2/faster-whisper (Python, 5-10x faster)

### Linux - Whisper Models (CTranslate2/faster-whisper backend):
- **tiny** - Fastest, less accurate (~39MB)
- **base** - Fast, good for simple dictation (~74MB)
- **small** - **Recommended** - Good accuracy, reasonable speed (~244MB)
- **medium** - Slower, more accurate (~769MB)
- **large** / **large-v3** - Best accuracy, slowest (~1550MB)

**Larger models take longer to transcribe but are more accurate.** Models download automatically on first use.

**With GPU:** Models run 5-10x faster with CUDA acceleration. Small model transcribes ~3 seconds in <1 second.

**Without GPU:** Models use CPU with INT8 quantization (slower but still works).

## Microphone Selection

Choose your microphone from the menu bar. **Use your system default microphone** (usually "Built-in Microphone") for best results.

## Technical Details

### macOS
- Pure Objective-C implementation for minimal overhead
- CoreAudio for low-latency audio capture (16kHz mono)
- CGEventTap for global hotkey monitoring
- Two backends: whisper.cpp (C++) or faster-whisper (Python/CTranslate2)
- Linear resampling for device compatibility
- 4KB audio buffers (~43ms latency)
- NSUserNotification for buffer limit alerts

### Linux
- C implementation with GTK3 system tray
- PortAudio for cross-platform audio capture (16kHz mono)
- X11 XRecord for global keyboard hook (Scroll Lock)
- XTest for text input simulation
- CTranslate2/faster-whisper backend for transcription
- pthread worker for background processing
- libnotify for desktop notifications (buffer limit alerts)

## Troubleshooting

### macOS

#### Microphone not activating
1. Check Input Monitoring permission for Voice to Text in System Settings → Privacy & Security
2. Toggle permission OFF then ON
3. Restart Voice to Text

#### No transcription output
1. Enable logging: Voice to Text menu → Logging: On
2. Check logs: `log stream --predicate 'process == "VTT"'`
3. Verify whisper model is downloaded

#### Key not detected
1. Check System Settings → Keyboard → Modifier Keys
2. Ensure Right Option isn't remapped
3. Try with logging enabled to see key events

### Linux

#### Scroll Lock not detected
1. Check if XRecord extension is available: `xdpyinfo | grep RECORD`
2. Ensure you have X11 permissions (not running in Wayland)
3. Check logs: `tail -f ~/.local/share/voice-to-text/vtt.log`
4. Verify Scroll Lock is not remapped: `xmodmap -pm`

#### No system tray icon
1. Ensure AppIndicator is installed: `apt list --installed | grep libayatana-appindicator3`
2. Check if system tray is enabled in your desktop environment
3. Try alternative: `vtt-status` to verify process is running

#### Python transcription errors
1. Verify faster-whisper is installed: `python3.12 -m pip list | grep faster-whisper`
2. Check model download location: `ls ~/.cache/huggingface/hub/`
3. Test manually: `python3.12 -c "from faster_whisper import WhisperModel; WhisperModel('base')"`

#### Audio device issues
1. List devices: `pactl list sources short`
2. Check permissions: ensure user is in `audio` group
3. Test recording: `arecord -d 3 test.wav && aplay test.wav`

## Building

### macOS Build Requirements
- macOS 11.0+
- Xcode Command Line Tools
- CMake (for whisper.cpp)

The Makefile handles:
- Building embedded whisper.cpp library
- Compiling VTT daemon
- Creating app bundle with icon
- Bundling whisper model

### Linux Build Requirements
- Ubuntu 24.04+ or Debian-based distro
- GCC 11+ with C11 support
- Development libraries: PortAudio, X11, XTest, GTK3, AppIndicator3
- Python 3.12+ with faster-whisper

The Makefile.linux handles:
- Compiling all C sources (audio, keyboard, typing, GUI, transcription)
- Linking against system libraries
- Creating standalone executable

## Architecture

### macOS App Bundle
```
VTT.app/
├── Contents/
│   ├── MacOS/
│   │   ├── VTT              # Main executable
│   │   └── whisper-cli       # Fallback CLI (optional)
│   ├── Resources/
│   │   ├── AppIcon.icns      # App icon
│   │   └── ggml-small.en.bin # Bundled model
│   └── Info.plist
```

### Cross-Platform Repository Structure
```
voice-to-text/
├── src/
│   ├── common/              # Shared cross-platform code
│   │   ├── logging.c/h      # Logging system
│   │   ├── queue.c/h        # Thread-safe queue
│   │   └── transcribe.py    # Python transcription backend
│   ├── macos/               # macOS-specific code
│   │   ├── VTTDaemon.m      # Main app
│   │   ├── VTTOnboarding.m  # Permission onboarding
│   │   └── create_icon.py   # Icon generator
│   └── linux/               # Linux-specific code
│       ├── audio.c/h        # PortAudio recording
│       ├── keyboard.c/h     # X11 keyboard hook
│       ├── typing.c/h       # XTest text input
│       ├── transcribe.c/h   # Python wrapper caller
│       ├── gui.c/h          # GTK3 AppIndicator
│       └── main.c           # Entry point
├── Makefile                 # macOS build
└── Makefile.linux           # Linux build
```

## Contributing

We welcome contributions! Voice to Text is an open-source project and we'd love your help making it better.

### How to Contribute

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Create a feature branch**: `git checkout -b feature/your-feature-name`
4. **Make your changes** and test thoroughly
5. **Commit with clear messages**: `git commit -m "feat: add feature description"`
6. **Push to your fork**: `git push origin feature/your-feature-name`
7. **Open a Pull Request** on the main repository

### Development Setup

#### macOS Development
```bash
# Clone your fork
git clone https://github.com/YOUR-USERNAME/voice-to-text.git
cd voice-to-text

# Install dependencies (if needed)
brew install cmake portaudio

# Build whisper.cpp dependencies
make vendor-whisper
make whisper-lib

# Build and test
make complete
open VTT.app
```

#### Linux Development
```bash
# Clone your fork
git clone https://github.com/YOUR-USERNAME/voice-to-text.git
cd voice-to-text

# Install build dependencies
sudo apt install build-essential pkg-config \
    portaudio19-dev libx11-dev libxtst-dev libxext-dev \
    libgtk-3-dev libayatana-appindicator3-dev libnotify-dev \
    python3.12 python3-pip

# Install Python transcription backend
python3.12 -m pip install --break-system-packages faster-whisper

# Build and test
make -f Makefile.linux
./vtt-linux
```

### Commit Message Guidelines

We follow conventional commits:
- `feat:` New feature
- `fix:` Bug fix
- `chore:` Maintenance tasks
- `docs:` Documentation updates
- `refactor:` Code refactoring
- `test:` Adding tests
- `perf:` Performance improvements

### Ideas for Contributions

**Features:**
- Multi-language support (currently English-only)
- Custom hotkey combinations
- Transcription history/clipboard manager
- Alternative Whisper backends (Faster Whisper, MLX)
- Streaming transcription (real-time as you speak)
- Punctuation restoration
- Speaker diarization (multiple speakers)

**Improvements:**
- Better VAD (Voice Activity Detection)
- Reduce model download sizes
- Support for other audio input devices
- macOS Shortcuts integration
- AppleScript support
- Better error messaging

**Documentation:**
- Usage tutorials and videos
- Performance benchmarks across different Macs
- Accuracy comparisons between models
- Integration guides (Vim, VS Code, etc.)

**Testing:**
- Automated UI tests
- Audio pipeline tests
- Model accuracy benchmarks
- Performance regression tests

### Code Style

- Use Objective-C ARC (Automatic Reference Counting)
- Keep methods focused and well-documented
- Follow existing naming conventions
- Add VTTLog statements for debugging
- Test on both Intel and Apple Silicon Macs

### Reporting Issues

Found a bug? Have a feature request?

1. Check existing [GitHub Issues](https://github.com/powell-clark/voice-to-text/issues)
2. If it doesn't exist, create a new issue with:
   - Clear title and description
   - Steps to reproduce (for bugs)
   - Expected vs actual behavior
   - macOS version and Mac model
   - Relevant logs (enable logging in menu)

### Questions?

- Open a [GitHub Discussion](https://github.com/powell-clark/voice-to-text/discussions)
- Check existing issues and discussions first

### Testing Pull Requests

Want to test someone's PR before it's merged?

```bash
# Add their fork as a remote
git remote add contributor https://github.com/THEIR-USERNAME/voice-to-text.git

# Fetch and checkout their branch
git fetch contributor
git checkout contributor/their-branch-name

# Build and test
make clean && make complete
open VTT.app
```

## License

Copyright © 2025 Powell-Clark Limited

Licensed under the Apache License, Version 2.0

## Credits

- Built with [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- Uses OpenAI's Whisper models
- Icon generated programmatically with CoreGraphics
