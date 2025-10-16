# Voice to Text - Offline Voice Transcription

**Push-to-talk voice transcription** using OpenAI Whisper AI. Supports **99+ languages** with automatic detection. **100% offline** - no internet required.

**Available for macOS and Linux** • **99+ Languages** • **GPU Accelerated** • **Push-to-Talk Interface**

---

## 🚀 Installation

### macOS (Homebrew)

```bash
brew tap powell-clark/voice-to-text
brew install --cask voice-to-text
```

The app installs to `/Applications/Voice to Text.app` and runs in your menu bar.

**First Run:** Grant permissions for Microphone, Accessibility, and Input Monitoring when prompted.

### Linux (Ubuntu/Debian) - Coming Soon via PPA

```bash
# PPA installation (coming soon!)
sudo add-apt-repository ppa:powell-clark/voice-to-text
sudo apt update
sudo apt install voice-to-text
```

**For now, see [Build from Source](#-build-from-source-and-contributing) below.**

---

## 💡 How It Works

1. **Press and hold** the hotkey:
   - **macOS:** Right Alt/Option key
   - **Linux:** Scroll Lock key

2. **Speak** in any of 99+ languages (English, Spanish, French, German, Chinese, Japanese, Korean, Arabic, Hindi, Portuguese, Russian, Italian, and more)

3. **Release** when finished speaking

4. **Text appears instantly** in your focused application (email, document, chat, terminal, browser, etc.)

**Everything runs locally** - no internet connection required. Your voice never leaves your device.

---

## ✨ Features

### 99+ Language Support

**Two modes available:**

- **English only (fastest)** - Optimized for English transcription using .en models
- **Multilingual (99 languages)** - Automatically detects language from 99+ supported languages

**Supported Languages:**
- **European:** English, Spanish, French, German, Italian, Portuguese, Dutch, Polish, Russian, Turkish, Greek, Swedish, Danish, Norwegian, Finnish, Czech, Romanian, Hungarian, Bulgarian, Croatian, Ukrainian, and more
- **Asian:** Chinese (Mandarin), Japanese, Korean, Hindi, Bengali, Vietnamese, Thai, Indonesian, Malay, Tagalog, Tamil, Telugu, Urdu, and more
- **Middle Eastern:** Arabic, Hebrew, Persian (Farsi), and more
- **African:** Swahili, Afrikaans, Zulu, and more
- **Other:** Welsh, Icelandic, Estonian, Latvian, Lithuanian, Slovenian, Slovak, Serbian, Macedonian, Bosnian, Albanian, Maltese, and more

### Multiple AI Models

Choose from different model sizes - the app automatically downloads them on first use:

- **tiny** - Fastest, less accurate (~39MB)
- **base** - Fast, good for simple dictation (~74MB)
- **small** - **Recommended** - Best balance of speed and accuracy (~244MB)
- **medium** - Slower, more accurate (~769MB)
- **large/large-v3** - Best accuracy, slowest (~1550MB)

**Two backends available:**
- **W models** - whisper.cpp (C++, no Python required)
- **CT2 models** - CTranslate2/faster-whisper (Python, 5-10x faster with GPU)

**Recommended:** Start with **CT2 small in English-only mode** for best speed and accuracy.

### GPU Acceleration

**With NVIDIA GPU (Linux):** 5-10x faster transcription using CUDA
**Without GPU:** Automatic fallback to CPU with INT8 quantization

### Recording Limits & Notifications

To prevent accidental long recordings:
- **Linux:** 300 seconds (5 minutes) maximum
- **macOS:** 10 seconds (for testing)

Desktop notification appears when you reach the limit. Transcription includes **[Truncated - Xs limit]** prefix so you know it was cut off.

---

## 🔧 Managing Voice to Text

### macOS

Access from the menu bar icon:
- Click icon → Select microphone
- Click icon → Choose language mode
- Click icon → Select AI model
- Click icon → Enable/disable logging
- Click icon → Quit

View logs: `log stream --predicate 'process == "VTT"'`

### Linux (systemd service)

**System tray menu:**
- Click icon → Select microphone
- Click icon → Choose language mode
- Click icon → Select AI model
- Click icon → Customize hotkey
- Click icon → Enable/disable logging
- Click icon → Quit

**Customizing the hotkey:**
- Default: **Scroll Lock** (recommended, doesn't interfere with other applications)
- To change: Click tray icon → "Hotkey: Scroll Lock" → Press and hold your desired key
- Best options: F1-F12, Pause, Insert, Home, End, Page Up, Page Down
- Note: Some keys may show "invalid key" error - just try a different key
- Changes save automatically to `~/.local/share/voice-to-text/settings.conf`

**Managing the service:**
```bash
# Start/stop/restart
systemctl --user start vtt
systemctl --user stop vtt
systemctl --user restart vtt

# Check status and logs
systemctl --user status vtt
journalctl --user -u vtt -f

# Enable/disable auto-start on login
systemctl --user enable vtt
systemctl --user disable vtt
```

**Convenient aliases** - Add to your `~/.bashrc`:
```bash
alias vtt-start='systemctl --user start vtt'
alias vtt-stop='systemctl --user stop vtt'
alias vtt-restart='systemctl --user restart vtt'
alias vtt-status='systemctl --user status vtt'
alias vtt-log='tail -f ~/.local/share/voice-to-text/vtt.log'
```

---

## 🛠️ Build from Source and Contributing

Want to build from source, contribute code, or port to other platforms? Here's how to get started.

### macOS Development Build

```bash
# Clone repository
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text

# Install dependencies (if needed)
brew install cmake portaudio

# Build whisper.cpp dependencies
make vendor-whisper  # Download whisper.cpp
make whisper-lib     # Build whisper library

# Build and run
make complete        # Build VTT.app
open VTT.app         # Launch for testing

# Install system-wide
cp -R VTT.app /Applications/
```

**Requirements:**
- macOS 11.0+
- Xcode Command Line Tools
- CMake (for whisper.cpp)

### Linux Development Build

```bash
# Clone repository
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text

# Install build dependencies
sudo apt install build-essential pkg-config \
    portaudio19-dev libx11-dev libxtst-dev libxext-dev \
    libgtk-3-dev libayatana-appindicator3-dev libnotify-dev \
    python3.12 python3-pip

# Install faster-whisper for transcription
python3.12 -m pip install --break-system-packages faster-whisper ctranslate2

# Build and run
make -f Makefile.linux
./vtt-linux

# Install system-wide (optional)
sudo make -f Makefile.linux install

# Set up systemd service (optional, for auto-start)
mkdir -p ~/.config/systemd/user
cp vtt.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable vtt
systemctl --user start vtt
```

**Requirements:**
- Ubuntu 24.04+ or Debian-based distro
- GCC 11+ with C11 support
- Development libraries: PortAudio, X11, XTest, GTK3, AppIndicator3
- Python 3.12+ with faster-whisper

#### GPU Acceleration Setup (Linux, Optional but Recommended)

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

### Repository Structure

```
voice-to-text/
├── src/
│   ├── common/              # Shared cross-platform code
│   │   ├── logging.c/h      # Logging system
│   │   ├── queue.c/h        # Thread-safe queue
│   │   ├── settings.c/h     # Configuration persistence
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
├── debian/                  # Debian packaging (for PPA)
├── homebrew-cask/          # Homebrew formula (for macOS)
├── Makefile                # macOS build
└── Makefile.linux          # Linux build
```

### How to Contribute

We welcome contributions! Here's how:

1. **Fork the repository** on GitHub
2. **Clone your fork** locally: `git clone https://github.com/YOUR-USERNAME/voice-to-text.git`
3. **Create a feature branch**: `git checkout -b feature/your-feature-name`
4. **Make your changes** and test thoroughly on your platform
5. **Commit with clear messages**: `git commit -m "feat: add feature description"`
6. **Push to your fork**: `git push origin feature/your-feature-name`
7. **Open a Pull Request** on the main repository

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
- Custom hotkey combinations
- Transcription history/clipboard manager
- Streaming transcription (real-time as you speak)
- Punctuation restoration
- Speaker diarization (multiple speakers)
- Windows/Android/iOS ports

**Improvements:**
- Better VAD (Voice Activity Detection)
- Reduce model download sizes
- macOS Shortcuts integration
- AppleScript support
- Better error messaging
- Wayland support (Linux)

**Documentation:**
- Usage tutorials and videos
- Performance benchmarks
- Accuracy comparisons between models
- Integration guides (Vim, VS Code, etc.)

**Testing:**
- Automated UI tests
- Audio pipeline tests
- Model accuracy benchmarks
- Performance regression tests

### Code Style

- **macOS:** Use Objective-C ARC (Automatic Reference Counting), follow Apple conventions
- **Linux:** Use C11, follow Linux kernel style guidelines
- Keep functions focused and well-documented
- Add logging statements for debugging
- Test on target platforms before submitting PR

### Reporting Issues

Found a bug? Have a feature request?

1. Check existing [GitHub Issues](https://github.com/powell-clark/voice-to-text/issues)
2. If it doesn't exist, create a new issue with:
   - Clear title and description
   - Steps to reproduce (for bugs)
   - Expected vs actual behavior
   - OS version and hardware details
   - Relevant logs (enable logging in menu)

### Questions?

- Open a [GitHub Discussion](https://github.com/powell-clark/voice-to-text/discussions)
- Check existing issues and discussions first

---

## 🔍 Technical Details

### macOS Architecture
- Pure Objective-C implementation for minimal overhead
- CoreAudio for low-latency audio capture (16kHz mono)
- CGEventTap for global hotkey monitoring
- Two backends: whisper.cpp (C++) or faster-whisper (Python/CTranslate2)
- NSUserNotification for buffer limit alerts
- App Bundle structure with embedded models

### Linux Architecture
- C implementation with GTK3 system tray
- PortAudio for cross-platform audio capture (16kHz mono)
- X11 XRecord for global keyboard hook (Scroll Lock)
- XTest for text input simulation
- CTranslate2/faster-whisper backend for transcription
- pthread worker for background processing
- libnotify for desktop notifications
- systemd user service integration

---

## 🐛 Troubleshooting

### macOS

#### Microphone not activating
1. Check **Input Monitoring** permission: System Settings → Privacy & Security → Input Monitoring
2. Ensure "Voice to Text" is checked
3. Toggle permission OFF then ON
4. Restart Voice to Text

#### No transcription output
1. Enable logging: Voice to Text menu → Logging: On
2. Check logs: `log stream --predicate 'process == "VTT"'`
3. Verify model downloaded: Check `~/Library/Application Support/voice-to-text/models/`
4. Try a different model (e.g., switch to CT2 small)

#### Right Alt key not detected
1. Check System Settings → Keyboard → Modifier Keys
2. Ensure Right Option isn't remapped
3. Try with logging enabled to see key events in Console

### Linux

#### Scroll Lock not detected
1. Check if XRecord extension is available: `xdpyinfo | grep RECORD`
2. Ensure running under X11 (not pure Wayland)
3. Check logs: `tail -f ~/.local/share/voice-to-text/vtt.log`
4. Verify Scroll Lock is not remapped: `xmodmap -pm`

#### No system tray icon
1. Ensure AppIndicator is installed: `apt list --installed | grep libayatana-appindicator3`
2. Check if system tray extension is enabled in your desktop environment (GNOME, KDE, etc.)
3. Verify process is running: `systemctl --user status vtt`

#### Python transcription errors
1. Verify faster-whisper is installed: `python3.12 -m pip list | grep faster-whisper`
2. Check model download location: `ls ~/.cache/huggingface/hub/`
3. Test manually: `python3.12 -c "from faster_whisper import WhisperModel; WhisperModel('base')"`
4. Check Python version: `python3.12 --version` (must be 3.12+)

#### Audio device issues
1. List available devices: `pactl list sources short`
2. Check permissions: ensure user is in `audio` group: `groups | grep audio`
3. Test recording: `arecord -d 3 test.wav && aplay test.wav`
4. Try different microphone from the menu

#### GPU not being used (Linux)
1. Check CUDA installation: `nvcc --version`
2. Verify GPU is detected: `python3.12 -c "import ctranslate2; print(ctranslate2.get_cuda_device_count())"`
3. Check CUDA paths in `~/.bashrc`: `echo $LD_LIBRARY_PATH`
4. Restart VTT service: `systemctl --user restart vtt`

---

## 📄 License

Copyright © 2025 Powell-Clark Limited

Licensed under the Apache License, Version 2.0

See [LICENSE](LICENSE) file for details.

---

## 🙏 Credits

- Built with [whisper.cpp](https://github.com/ggerganov/whisper.cpp) by Georgi Gerganov
- Uses [faster-whisper](https://github.com/guillaumekln/faster-whisper) and [CTranslate2](https://github.com/OpenNMT/CTranslate2)
- Powered by OpenAI's [Whisper](https://github.com/openai/whisper) models

---

**Questions? Issues? Contributions?** → [GitHub Issues](https://github.com/powell-clark/voice-to-text/issues) • [Discussions](https://github.com/powell-clark/voice-to-text/discussions)
