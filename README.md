# Voice to Text

Local push-to-talk voice transcription using OpenAI Whisper. Hold a key, speak,
and the text types into whatever app you're in.

[![Windows](https://img.shields.io/badge/Windows-11-blue?logo=windows)](https://github.com/powell-clark/voice-to-text/releases/latest)
[![macOS](https://img.shields.io/badge/macOS-Intel%20%26%20Apple%20Silicon-blue?logo=apple)](https://github.com/powell-clark/voice-to-text/releases/latest)
[![Linux](https://img.shields.io/badge/Linux-Ubuntu%2022.04+-orange?logo=linux)](https://github.com/powell-clark/voice-to-text/releases/latest)

**100% offline. No cloud. No subscriptions. GPU-accelerated.**

---

## Install

**Windows 11** — download **`voice-to-text-installer.msi`** from the
[latest release](https://github.com/powell-clark/voice-to-text/releases/latest)
and run it. Installs to `C:\Program Files\Voice to Text\`.

**Linux (Ubuntu 22.04 / 24.04)**
```bash
sudo add-apt-repository ppa:powellclark/voice-to-text
sudo apt update && sudo apt install voice-to-text
```

**macOS** — download the binary for your chip from the
[latest release](https://github.com/powell-clark/voice-to-text/releases/latest)
(**`vtt-macos-intel`** or **`vtt-macos-arm64`**), then:
```bash
chmod +x vtt-macos-intel && xattr -d com.apple.quarantine vtt-macos-intel && ./vtt-macos-intel
```
(A signed `.app` bundle is in progress.)

Then **hold Scroll Lock and speak.** A tray icon gives you model, language, and
settings.

---

## Usage

Hold **Scroll Lock**, speak, release — the transcription types into the focused
app (Slack, terminal, VS Code, browser, anywhere). The tray/menu icon turns red
while recording and amber while transcribing.

Right-click the tray icon to pick a model, switch language, toggle logging, or
(Windows) enable **Start at login**.

---

## Features

- **Push-to-talk** — hold a key, speak, release. Text appears instantly.
- **100% offline** — no internet, no cloud, no tracking.
- **GPU-accelerated** — Vulkan on Windows/Linux (NVIDIA/AMD/Intel, no CUDA),
  Metal on macOS. In-process inference: the model loads once, then each press is
  sub-second.
- **Multiple models** — small → large-v3, balance speed vs accuracy.
- **99+ languages** — English-only mode (fastest) or multilingual auto-detect.
- **System tray** — configure model/language/hotkey without opening an app.

---

## Configuration

Click the tray icon to adjust:

| Setting | Options | Default |
|---------|---------|---------|
| **Model** | small / medium / large-v3-turbo / large-v3 | small |
| **Language** | English-only / Multilingual | English-only |
| **Hotkey** | Customisable recording key | Scroll Lock |
| **Initial prompt** | Free text (≤240 chars) to prime custom vocab | (empty) |
| **Logging** | On / Off (daily log files) | On |

| Model | Size | Speed | Use case |
|-------|------|-------|----------|
| **small.en / small** | 466 MB | ⚡⚡⚡⚡ | Recommended — best balance |
| **medium.en / medium** | 1.4 GB | ⚡⚡⚡ | Higher accuracy, slower |
| **large-v3-turbo** | 1.5 GB | ⚡⚡⚡⚡ | Large accuracy at small speed (multilingual) |
| **large-v3** | 2.9 GB | ⚡⚡ | Maximum accuracy (multilingual) |

Models download on first selection from `ggerganov/whisper.cpp`. Start with
**small** in **English-only** mode.

---

## Build from source

One Rust crate, built with `cargo` on every platform. You need Rust (rustup),
cmake, and libclang (for whisper-rs bindgen).

```bash
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text
cargo build --release      # output: target/release/vtt-linux[.exe]
```

- **Windows** — needs the Vulkan SDK + VS Build Tools (C++); the helper
  `scripts/build-windows.ps1` auto-discovers cmake/libclang and builds.
- **Linux** — `sudo apt install build-essential pkg-config cmake glslc
  libgtk-3-dev libayatana-appindicator3-dev libasound2-dev libvulkan-dev
  libxtst-dev libnotify-bin`. Build a local `.deb`:
  `bash scripts/release-local.sh --install`.
- **macOS** — `brew install cmake`, then `cargo build --release`.

See [`docs/PLATFORM-PARITY.md`](docs/PLATFORM-PARITY.md) for the per-platform
capability spec and [`CLAUDE.md`](CLAUDE.md) for build/packaging commands.

---

## Troubleshooting

- **Confirm the version:** `vtt --version` (Windows: `vtt`, Linux: `vtt-linux`).
- **No transcription / GPU not used:** enable logging from the tray, then check
  today's log (`%APPDATA%\voice-to-text\logs\` on Windows,
  `~/.local/share/voice-to-text/` on Linux). The log shows the GPU device and
  per-transcription timing.
- **Linux hotkey not working:** requires X11 (`echo $XDG_SESSION_TYPE` →
  `x11`); Wayland is on the roadmap. No tray icon? Install
  `libayatana-appindicator3-1` (GNOME also needs the AppIndicator extension).
- **macOS:** grant Microphone, Accessibility, and Input Monitoring permissions
  (System Settings → Privacy & Security) on first run.

Still stuck? [Open an issue](https://github.com/powell-clark/voice-to-text/issues).

---

## Architecture

Since v2.0, the whole app is a single Rust crate doing in-process `whisper-rs`
inference (the older C/Python/ObjC hybrid was removed). Audio via `cpal`,
transcription via `whisper-rs` (Vulkan/Metal), tray via `gtk`/AppIndicator on
Linux and `tray-icon`/`muda` on Windows+macOS, text injection via `enigo`,
hotkey via X11 `XGrabKey` (Linux) / `rdev` (Windows/macOS).

## Contributing

PRs welcome — [conventional commits](https://www.conventionalcommits.org/).
Install the pre-push hook (`bash scripts/git-hooks/install.sh`) to run the same
fmt/clippy/test/build checks as CI before pushing.

## Credits

[whisper.cpp](https://github.com/ggerganov/whisper.cpp) ·
[whisper-rs](https://github.com/tazz4843/whisper-rs) ·
[cpal](https://github.com/RustAudio/cpal) ·
[enigo](https://github.com/enigo-rs/enigo) ·
[OpenAI Whisper](https://github.com/openai/whisper)

## License

Apache License 2.0 — see [LICENSE](LICENSE). Copyright © 2025–2026 Powell-Clark Limited.
