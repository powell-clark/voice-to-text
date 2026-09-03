# Voice to Text

Local push-to-talk voice transcription using OpenAI Whisper. Hold a key, speak,
and the text types into whatever app you're in.

[![Windows](https://img.shields.io/badge/Windows-11-blue?logo=windows)](https://github.com/powell-clark/voice-to-text/releases/latest)
[![macOS](https://img.shields.io/badge/macOS-Intel%20%26%20Apple%20Silicon-blue?logo=apple)](https://github.com/powell-clark/voice-to-text/releases/latest)
[![Linux](https://img.shields.io/badge/Linux-Ubuntu%2022.04+-orange?logo=linux)](https://github.com/powell-clark/voice-to-text/releases/latest)
[![CI](https://github.com/powell-clark/voice-to-text/actions/workflows/ci.yml/badge.svg)](https://github.com/powell-clark/voice-to-text/actions/workflows/ci.yml)

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

## Archiving your recordings

**Off by default. It saves your voice and your words to your disk. Read this
before turning it on.**

Normally a recording is transcribed, typed, and thrown away — only the last 20
are kept, briefly, so the tray's *Re-transcribe last recording* has something to
work with. Archiving keeps them all instead: every recording you make, at your
microphone's full quality, next to a file containing exactly what you said.

Nothing is uploaded. Nothing leaves your machine. There is no account, no
telemetry and no network call in this feature — it writes two files to a folder
you choose and that is all it does. But it does mean a growing folder of your
own voice and a searchable record of everything you have dictated, so turn it on
deliberately, and think about who else can read that disk.

**Turning it on.** Add to `~/.config/voice-to-text/settings.conf`:

```ini
archive=1
# Optional. Default: ~/.config/voice-to-text/archive
archive_dir="~/voice-archive"
# Optional. Oldest recordings are deleted past this. 0 = keep everything.
archive_max_files=5000
```

Restart the app. With `archive` absent or `0`, nothing is written and the app
behaves exactly as it did before this feature existed.

**What you get**, one folder per day:

```text
~/voice-archive/2026-09-03/vtt_20260903T034812_123.wav    your voice, 48 kHz mono
~/voice-archive/2026-09-03/vtt_20260903T034812_123.json   what you said, plus when
```

The `.json` holds the transcript, the timestamp, the duration, the sample rate,
the model and the language. At the 5,000-file default the folder settles at
roughly 11 hours of audio, under a gigabyte.

**Turning it off.** Set `archive=0` (or delete the line) and restart. Recordings
already archived stay where they are — turning the feature off does not delete
anything.

**Deleting everything.** The archive is an ordinary folder. Delete it the way
you would any other, or from a terminal:

```bash
rm -rf ~/.config/voice-to-text/archive      # or your own archive_dir
```

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
