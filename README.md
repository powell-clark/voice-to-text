# Voice to Text

**macOS • Linux**

Offline voice transcription with push-to-talk. Powered by OpenAI Whisper.

| Platform | Install Method | Status |
|----------|---------------|--------|
| **macOS** | Homebrew Cask | ✅ Available |
| **Linux** (Ubuntu/Debian) | PPA | ✅ Available |
| **Windows** | — | 🔜 Coming Soon |

---

## Install

### macOS

```bash
brew tap powell-clark/voice-to-text
brew install --cask voice-to-text
```

App runs in your menu bar. Grant permissions for Microphone, Accessibility, and Input Monitoring when prompted.

### Linux (Ubuntu/Debian)

```bash
sudo add-apt-repository ppa:powellclark/voice-to-text
sudo apt update
sudo apt install voice-to-text
```

Runs as a systemd service with system tray icon. Starts automatically on login.

---

## Usage

1. **Press and hold** the hotkey (Right Alt on macOS, Scroll Lock on Linux)
2. **Speak** into your microphone
3. **Release** to transcribe
4. Text appears in your current application

Everything runs locally. No internet required.

---

## Configuration

Click the menu/tray icon to configure:

- **Microphone** - Select input device
- **Model** - Balance speed and accuracy (tiny/base/small/medium/large)
- **Language** - English-only (faster) or multilingual (99+ languages)
- **Hotkey** (Linux) - Customize recording key
- **Logging** - Enable for debugging

### Available Models

Models download automatically on first use:

| Model | Size | Speed | Best For |
|-------|------|-------|----------|
| **tiny** | 39MB | Fastest | Quick tests |
| **base** | 74MB | Fast | Simple dictation |
| **small** | 244MB | **Recommended** | Best balance |
| **medium** | 769MB | Slower | Higher accuracy |
| **large-v3** | 1550MB | Slowest | Maximum accuracy |

**Two backends available:**

- **W models** - whisper.cpp (C++, lightweight)
- **CT2 models** - CTranslate2 + faster-whisper (Python, 5-10x faster with GPU)

**Recommended:** Start with **CT2 small** in English-only mode for best performance.

### GPU Acceleration (Linux)

With NVIDIA GPU, transcription runs 5-10x faster. Install CUDA 12.6:

```bash
sudo apt install cuda-toolkit-12-6 libcudnn9-cuda-12
```

Add to `~/.bashrc`:

```bash
export PATH=/usr/local/cuda-12.6/bin:$PATH
export LD_LIBRARY_PATH=/usr/local/cuda-12.6/lib64:$LD_LIBRARY_PATH
```

Restart the service:

```bash
systemctl --user restart vtt
```

### Linux Service Management

```bash
# Start/stop/restart
systemctl --user start vtt
systemctl --user stop vtt
systemctl --user restart vtt

# View logs
journalctl --user -u vtt -f
tail -f ~/.local/share/voice-to-text/vtt.log

# Disable auto-start
systemctl --user disable vtt
```

---

## Supported Languages

**English-only mode:** Uses optimized .en models for faster transcription

**Multilingual mode:** Auto-detects from 99+ languages:

- **European:** English, Spanish, French, German, Italian, Portuguese, Dutch, Polish, Russian, Turkish, Greek, Swedish, Danish, Norwegian, Finnish, Czech, Romanian, Hungarian, Ukrainian
- **Asian:** Chinese, Japanese, Korean, Hindi, Vietnamese, Thai, Indonesian, Tamil
- **Middle Eastern:** Arabic, Hebrew, Persian
- **African:** Swahili, Afrikaans
- **And many more...**

---

## Build from Source

### macOS

```bash
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text

# Install dependencies
brew install cmake portaudio

# Build whisper.cpp dependencies
make vendor-whisper
make whisper-lib

# Build app
make complete
open VTT.app

# Install system-wide
cp -R VTT.app /Applications/
```

**Requirements:** macOS 11.0+, Xcode Command Line Tools

### Linux (Ubuntu/Debian)

```bash
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text

# Install dependencies
sudo apt install build-essential pkg-config portaudio19-dev \
    libx11-dev libxtst-dev libxext-dev libgtk-3-dev \
    libayatana-appindicator3-dev libnotify-dev \
    python3.12 python3-pip

# Install Python transcription backend
python3.12 -m pip install --break-system-packages faster-whisper ctranslate2

# Build
make -f Makefile.linux
./vtt-linux
```

**Requirements:** Ubuntu 24.04+, GCC 11+, Python 3.12+

---

## Troubleshooting

### macOS

**Hotkey not working:**
- System Settings → Privacy & Security → Input Monitoring
- Enable Voice to Text, restart app

**No transcription:**
- Enable logging from menu
- Check logs: `log stream --predicate 'process == "VTT"'`
- Try CT2 small model

### Linux

**Hotkey not working:**
- Verify X11: `echo $XDG_SESSION_TYPE` (must be x11, not wayland)
- Check logs: `tail -f ~/.local/share/voice-to-text/vtt.log`
- Try customizing hotkey from tray menu

**No system tray icon:**
- Install AppIndicator: `sudo apt install libayatana-appindicator3-1`
- Check service status: `systemctl --user status vtt`
- GNOME users need AppIndicator extension

**GPU not working:**
- Check CUDA: `nvcc --version`
- Test: `python3.12 -c "import ctranslate2; print(ctranslate2.get_cuda_device_count())"`
- Restart after installing CUDA: `systemctl --user restart vtt`

**Microphone issues:**
- List devices: `pactl list sources short`
- Test: `arecord -d 3 test.wav && aplay test.wav`
- Select different mic from tray menu

---

## Contributing

Pull requests welcome. Ideas to contribute:

**Features:**
- Custom hotkey combinations
- Transcription history viewer
- Real-time streaming transcription
- Windows/iOS/Android ports

**Improvements:**
- Better voice activity detection
- Reduce model download sizes
- Wayland support (Linux)

**Documentation:**
- Video tutorials
- Performance benchmarks
- Integration guides (Vim, VS Code, etc.)

Use conventional commits: `feat:` `fix:` `docs:` `chore:`

### Repository Structure

```
src/
├── common/              # Shared cross-platform code
│   ├── logging.c/h
│   ├── queue.c/h
│   ├── settings.c/h
│   └── transcribe.py    # Python transcription backend
├── macos/               # macOS Objective-C implementation
│   └── VTTDaemon.m      # Main app + menu bar
└── linux/               # Linux C implementation
    ├── audio.c          # PortAudio recording
    ├── keyboard.c       # X11 global hotkey hook
    ├── typing.c         # XTest text injection
    └── gui.c            # GTK3 system tray
```

---

## License

Apache License 2.0 • Copyright © 2025 Powell-Clark Limited

See [LICENSE](LICENSE) for details.

---

## Credits

Built with:
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) by Georgi Gerganov
- [faster-whisper](https://github.com/guillaumekln/faster-whisper) by Guillaume Klein
- [CTranslate2](https://github.com/OpenNMT/CTranslate2) by OpenNMT
- [OpenAI Whisper](https://github.com/openai/whisper) models
