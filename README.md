# Voice to Text

Local push-to-talk voice transcription using OpenAI Whisper.

[![CI](https://github.com/powell-clark/voice-to-text/actions/workflows/ci.yml/badge.svg)](https://github.com/powell-clark/voice-to-text/actions/workflows/ci.yml)
[![macOS](https://img.shields.io/badge/macOS-11.0+-blue?logo=apple)](https://github.com/powell-clark/voice-to-text)
[![Linux](https://img.shields.io/badge/Linux-Ubuntu%2024.04+-orange?logo=linux)](https://github.com/powell-clark/voice-to-text)
[![License](https://img.shields.io/badge/License-Apache%202.0-green.svg)](LICENSE)

**100% offline. No cloud. No subscriptions.**

---

## Install

### macOS

```bash
brew tap powell-clark/voice-to-text
brew install --cask voice-to-text
```

### Linux

```bash
sudo add-apt-repository ppa:powellclark/voice-to-text
sudo apt update && sudo apt install voice-to-text
```

That's it. Hold **Scroll Lock** and speak.

---

## Usage

**macOS:** Hold **Right Alt** + speak

**Linux:** Hold **Scroll Lock** + speak (customizable from tray menu)

Text appears instantly in any application - Slack, Terminal, VS Code, browsers, email, anywhere you can type.

<details>
<summary>View menu screenshots</summary>

<p align="center">
  <img src="docs/images/mac-menu.png" width="300" alt="macOS Menu">
  <img src="docs/images/voice-to-text-linux-menu.png" width="300" alt="Linux Menu">
</p>

</details>

---

## Features

### 🎯 Core
- **Push-to-talk recording** - Hold key, speak, release
- **Instant transcription** - Text types into your active app
- **100% offline** - No internet, no cloud, no tracking
- **Menu/tray integration** - Configure without opening an app

### 🚀 Performance
- **Multiple models** - Balance speed vs accuracy (small to large-v3)
- **Vulkan GPU acceleration** - Works on NVIDIA, AMD, and Intel GPUs — no CUDA toolkit required
- **In-process inference** - Model loads once at startup, sub-second transcription per press
- **Optimized English mode** - Uses .en model variants where available for extra speed

### 🌍 Languages
- **English-only mode** - Fastest, uses optimized .en variants for small/medium
- **99+ languages** - Auto-detects Chinese, Spanish, French, German, Japanese, Arabic, and more (via multilingual models)

---

## Configuration

Click the menu/tray icon to adjust:

| Setting | Options | Default |
|---------|---------|---------|
| **Model** | **small** / medium / large-v3-turbo / large-v3 | small |
| **Language** | English-only / Multilingual | English-only |
| **Hotkey (Linux)** | Customize recording key | Scroll Lock |
| **Initial prompt** | Free text (≤240 chars) — primes Whisper for custom vocab | (empty) |
| **Logging** | On / Off (daily log files under `~/.local/share/voice-to-text/`) | On |

### Model Comparison

| Model | Size | Speed | Use Case |
|-------|------|-------|----------|
| **small.en / small** | 466 MB | ⚡⚡⚡⚡ | **Recommended — best balance, shipped pre-downloaded** |
| **medium.en / medium** | 1.4 GB | ⚡⚡⚡ | Higher accuracy, noticeably slower |
| **large-v3-turbo** | 1.5 GB | ⚡⚡⚡⚡ | Large-model accuracy at small-model speed (multilingual only) |
| **large-v3** | 2.9 GB | ⚡⚡ | Maximum accuracy (multilingual only) |

Models download on first selection from ggerganov/whisper.cpp. `small.en`
is pre-downloaded by `postinst` so first-run works offline.

**Recommendation:** Start with **small** in **English-only mode**. Most GPUs
on Linux (Intel integrated, AMD, NVIDIA) automatically use Vulkan acceleration
without any driver configuration.

---

## Advanced Setup

### GPU Acceleration (Linux)

VTT v2.0+ uses Vulkan, which works on any GPU with a modern driver —
no CUDA toolkit required.

```bash
# Verify Vulkan is working:
vulkaninfo --summary | head -20

# If missing, install driver + runtime:
#   NVIDIA: sudo apt install libvulkan1 mesa-vulkan-drivers nvidia-driver-550
#   AMD:    sudo apt install libvulkan1 mesa-vulkan-drivers
#   Intel:  sudo apt install libvulkan1 mesa-vulkan-drivers

# Restart VTT to pick up the new driver:
systemctl --user restart vtt
```

First transcription after model switch takes a few seconds as the model
loads into VRAM; subsequent transcriptions are sub-second.

### Service Management (Linux)

```bash
# Start/stop/restart
systemctl --user start vtt
systemctl --user stop vtt
systemctl --user restart vtt

# View logs (daily rotated files, kept 7 days)
journalctl --user -u vtt -f
tail -f ~/.local/share/voice-to-text/vtt-$(date +%Y-%m-%d).log

# Open today's log from the tray: right-click icon → Logs → Today

# Disable auto-start
systemctl --user disable vtt
```

---

## Build From Source

### macOS

**Requirements:** macOS 11.0+, Xcode Command Line Tools

```bash
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text

# Install dependencies
brew install cmake portaudio

# Build
make vendor-whisper
make whisper-lib
make complete

# Run or install
open VTT.app
# OR: cp -R VTT.app /Applications/
```

### Linux

**Requirements:** Ubuntu 24.04+ (jammy 22.04 also supported), Rust stable (1.75+ via rustup).

```bash
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text

# Install system build deps (matches debian/control)
sudo apt install build-essential pkg-config cmake glslc \
  libgtk-3-dev libayatana-appindicator3-dev \
  libasound2-dev libvulkan-dev libxtst-dev libnotify-bin

# Install rustup if you don't have Rust (Ubuntu's cargo 1.75 can't build our dep tree):
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build --release
./target/release/vtt-linux
```

To build a .deb locally for a friend: `bash scripts/release-local.sh --install`.

---

## Troubleshooting

### macOS

**First-time setup: Grant permissions**

On first run, macOS requires permissions for microphone, accessibility, and input monitoring. Click "Check Permissions..." from the menu bar icon:

<details>
<summary>View permission setup steps</summary>

1. Click "Open System Settings" when prompted:

   <img src="docs/images/check-permissions-warning.png" width="400">

2. Allow microphone access:

   <img src="docs/images/mac-allow-microphone.png" width="400">

3. Verify all permissions are enabled:

   <img src="docs/images/check-permissions-mac-successful.png" width="400">

</details>

**Permissions not working**
- System Settings → Privacy & Security
- Remove VTT from Microphone, Accessibility, and Input Monitoring
- Re-add by launching VTT and clicking "Check Permissions..."
- Restart the app

**No transcription**
- Enable logging from the menu icon
- Check today's log: `tail -f ~/Library/Application\ Support/voice-to-text/vtt-$(date +%Y-%m-%d).log`
- Try switching to a smaller model (tray menu → **Model** → small)

### Linux

**Confirm what's installed**
```bash
vtt-linux --version         # prints version
apt-cache policy voice-to-text   # verifies PPA source
```

**Hotkey not working**
- Verify X11 (not Wayland): `echo $XDG_SESSION_TYPE`
- Must return `x11` — Wayland support is on the roadmap
- Check today's log: `tail -f ~/.local/share/voice-to-text/vtt-$(date +%Y-%m-%d).log`
- Open logs from the tray menu → **Logs** → Today
- Re-bind hotkey from the tray menu → **Hotkey**

**Typing stops partway through a transcription**
- Fixed in 2.0.5 — upgrade with `sudo apt update && sudo apt upgrade voice-to-text`
- If on 2.0.5+ and still happening, open an issue with a sample of the missing text
  and which application was receiving focus

**Transcription is typing `[Music]` / `[Blank Audio]`**
- Also filtered in 2.0.5+. Upgrade, or set a non-empty *Initial Prompt*
  under the tray's customise menu to prime Whisper for speech

**No system tray icon**
- Install AppIndicator: `sudo apt install libayatana-appindicator3-1`
- Check user service: `systemctl --user status vtt`
- GNOME users need the [AppIndicator extension](https://extensions.gnome.org/extension/615/appindicator-support/)

**GPU not detected / slow transcription**
- v2.0 uses Vulkan (not CUDA): `vulkaninfo --summary | head -20`
- Install drivers:
  - NVIDIA: `sudo apt install libvulkan1 mesa-vulkan-drivers nvidia-driver-550`
  - AMD/Intel: `sudo apt install libvulkan1 mesa-vulkan-drivers`
- Restart VTT after driver changes: `systemctl --user restart vtt`
- First-transcription is always slower (model loads into VRAM); subsequent
  presses are sub-second on a healthy GPU

**Microphone issues**
- List devices: `pactl list sources short`
- Test the default mic: `arecord -d 3 -f cd /tmp/mic-test.wav && aplay /tmp/mic-test.wav`
- Change default via PulseAudio: `pactl set-default-source <source-name>`

**PPA install shows wrong version**
```bash
sudo apt update
apt-cache policy voice-to-text    # confirm which version apt will install
sudo apt install voice-to-text=2.0.5  # pin a specific version if needed
```

---

## Architecture

Since v2.0 (April 2026), the whole application is a single Rust crate.
The older C + Python + ObjC hybrid was removed in favour of in-process
whisper-rs inference. See `CONSCIOUSNESS/adr/0003-whisper-rs-in-process-model.md`
for the rewrite rationale.

```
src/
├── main.rs             # Daemon entry — arg parsing, signal handling,
│                         hotkey wiring, transcription worker loop
├── audio.rs            # cpal 16 kHz capture + bounded-buffer accumulation
├── hotkey/             # Global push-to-talk key
│   ├── mod.rs          # KeyEvent / HotkeyCmd types
│   ├── linux.rs        # X11 XGrabKey + XTestFakeKeyEvent
│   └── portable.rs     # rdev-backed macOS/Windows variant
├── logging.rs          # Daily-rotated log files under XDG data dir
├── models.rs           # GGML model catalogue + HuggingFace download
├── settings.rs         # settings.conf parser / writer
├── transcribe.rs       # Thin bridge to whisper.rs
├── tray/               # System tray icon + menu
│   ├── mod.rs          # UiMessage types shared across platforms
│   ├── linux.rs        # libappindicator + GTK menu
│   └── portable.rs     # tray-icon + muda (macOS/Windows stubs)
├── typing.rs           # enigo Key::Unicode text injection
└── whisper.rs          # WhisperEngine wrapper around whisper-rs
```

**Tech stack:**
- **Audio**: cpal (cross-platform)
- **Transcription**: whisper-rs 0.16 with Vulkan (Linux/Windows) or
  Metal (macOS) GPU features
- **Models**: GGML-format Whisper from ggerganov/whisper.cpp
- **Tray**: libappindicator + gtk-rs (Linux) / tray-icon + muda
  (macOS/Windows)
- **Input injection**: enigo with X11 backend on Linux, native on mac/Win
- **Hotkey capture**: X11 XGrabKey (Linux) / rdev (macOS/Windows)

---

## Contributing

Pull requests welcome. Use [conventional commits](https://www.conventionalcommits.org/):

```
feat: Add real-time streaming transcription
fix: Resolve microphone detection on Ubuntu 24.10
docs: Update GPU installation guide
chore: Bump whisper.cpp to v1.5.4
```

### Development setup

After cloning, install the git pre-push hook so you catch fmt/clippy/test/build
failures locally before they turn CI red:

```bash
bash scripts/git-hooks/install.sh
```

The hook runs the same checks as `.github/workflows/ci.yml`:

- `cargo fmt --all -- --check`
- `cargo clippy --release --all-targets -- -D warnings`
- `cargo test --release`
- `cargo build --release`

Bypass once with `git push --no-verify` only in genuine emergencies.

### Ideas for Contributions

**Features:**
- Custom hotkey combinations (e.g., Cmd+Shift+Space)
- Transcription history viewer
- Real-time streaming (transcribe while speaking)
- Windows/iOS/Android ports

**Improvements:**
- Voice activity detection (auto-stop recording)
- Smaller model downloads (quantization)
- Wayland support (replace X11)

**Documentation:**
- Video tutorials
- Performance benchmarks
- Integration guides (Vim, VS Code plugins)

---

## Roadmap

- [ ] **Windows support** - Native Win32 implementation
- [ ] **Wayland support** - Replace X11 on Linux
- [ ] **Streaming transcription** - Real-time as you speak
- [ ] **Custom wake words** - "Computer, write this..."
- [ ] **Model compression** - Smaller downloads via quantization
- [ ] **Auto-punctuation** - Smart capitalization and punctuation

---

## Credits

Built with:
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) by Georgi Gerganov — the GGML inference engine we call via whisper-rs
- [whisper-rs](https://github.com/tazz4843/whisper-rs) — Rust bindings for whisper.cpp
- [cpal](https://github.com/RustAudio/cpal) — cross-platform audio I/O
- [enigo](https://github.com/enigo-rs/enigo) — cross-platform text injection
- [OpenAI Whisper](https://github.com/openai/whisper) — the underlying speech model

---

## License

Apache License 2.0 • Copyright © 2025 Powell-Clark Limited

See [LICENSE](LICENSE) for details.

---

**Made with ❤️ for developers, writers, and anyone tired of typing.**

⭐ Star this repo if it saved your wrists
