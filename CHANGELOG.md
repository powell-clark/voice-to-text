# Changelog

All user-facing changes are documented here. Installed versions on
Debian/Ubuntu can also be inspected via `apt changelog voice-to-text`
or `vtt-linux --version`.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
versioning: [SemVer](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- **Your dictation can now be archived as high-quality audio with its
  transcript.** Off by default. Turned on, every recording is saved at your
  microphone's own sample rate beside a JSON file holding exactly what you
  said, in one folder per day, capped so it cannot fill your disk. Nothing is
  uploaded. See *Archiving your recordings* in the README for what is stored,
  where, how to switch it off and how to delete it.

- **Steady low-frequency noise is filtered out before transcription.** Desk and
  fan rumble is cut by a high-pass on the audio sent to Whisper; measured on real
  dictation, four fifths of the background energy sits in the band it removes.
  On by default, `denoise=0` in `settings.conf` turns it off, and archived audio
  is never filtered.
- **The correction dictionary is editable from the tray.** Open *Customize
  Transcription Settings* and there is now a box for your `misheard => correct`
  pairs, one per line, instead of hand-editing `settings.conf`. Emptying the box
  removes them all.

### Changed
- **Recording now captures at 48 kHz instead of 16 kHz.** Whisper still
  receives 16 kHz — the audio is resampled once, on the finished recording,
  immediately before transcription — so transcription accuracy is unchanged.
  Capturing at the microphone's real rate is what makes an archived recording
  worth keeping; 16 kHz threw away detail that upsampling cannot bring back.

---

## [2.3.11] — 2026-08-18

### Fixed
- **The microphone can no longer be left recording after you let go.** The key
  handler waited for the previous transcription to finish typing on the same
  thread that delivers key-release events, so a real release queued behind
  that wait, arrived a fraction of a millisecond after the recording started,
  and was discarded as an auto-repeat artefact. The recording flag stayed set
  and no further keypress could clear it, so the mic stayed open, the audio
  clipped, and Whisper returned the same hallucinated phrase over and over —
  each one typed out. The wait now runs off the event thread, a release always
  stops the recording, and a watchdog force-stops any hold that outlives the
  buffer cap.
- **A quick tap is no longer reported as a dead microphone.** A tap can end
  before the audio backend delivers its first buffer, and any zero-sample
  capture was diagnosed as a dead stream however briefly the key was held —
  leaving a red error icon in the tray, firing a notification, and re-opening
  the capture stream for nothing. The hold is now timed from the press, so a
  genuinely dead device is still detected while a hasty tap is just discarded.
- **The Customize Hotkey dialog refuses keys you type with.** Push-to-talk
  holds its key globally, so binding space or a letter removed that character
  from every application and started a recording on every press. Space and
  Return are also the keys that activate a focused button, which made them
  easy to capture by accident. Printable keys and the editing keys are now
  rejected with an explanation, and startup warns when an existing
  `settings.conf` already carries one.

## [2.3.10] — 2026-07-17

### Fixed
- **"Copy last transcription" survives the copy.** It now holds the
  X11/Wayland clipboard selection itself (via xclip/xsel, or wl-copy on
  Wayland), instead of silently becoming a no-op for anyone not running a
  clipboard manager such as CopyQ.
- Model downloads hard-fail on a SHA-256 mismatch instead of using the corrupt
  file.
- The macOS paste fallback sends Cmd+V rather than Ctrl+V.
- Coredump capture plus an FFI/unsafe audit hardening pass.

### Added
- **"Re-transcribe last recording"** in the tray re-runs transcription on the
  newest recording and re-types the result — a recovery net for when typed
  output is lost to the wrong window being focused.
- `--file`/`-f` batch-transcribes a 16 kHz WAV to stdout and exits, with clean
  pipes and exit codes (no tray, no hotkey).
- The selected input-device index is honoured when opening the capture stream,
  falling back to the default device.

### Security
- `rustls-webpki` bumped to 0.103.13 (RUSTSEC-2026-0104).

## [2.3.9] — 2026-07-10

### Fixed
- **Recordings no longer silently die when your microphone changes.** vtt held
  one audio capture stream open from startup forever; if the mic re-enumerated
  (USB re-plug), the audio session restarted (logout/login), or
  PipeWire/WirePlumber suspended the source, every recording came back with
  zero samples until vtt was manually restarted. The stream is now
  self-healing: a stream error flags it dead and the next recording re-opens
  it against the current default device, and any zero-sample capture
  immediately re-opens the stream so the following press works.
- **"Recording too short (0.00s)" no longer masks a dead microphone.** A
  capture that produces zero samples is now reported honestly as
  "No audio captured — check microphone" (tray status, error icon, and a
  desktop notification on Linux) instead of blaming your timing.
- **vtt now restarts automatically after a crash.** The systemd unit uses
  `Restart=always` (was `on-failure`, which missed clean-but-wrong exits),
  and tray Quit now stops the service properly via `systemctl --user stop vtt`
  so an intentional quit stays quit instead of fighting the restart policy.

### Added
- **Correction dictionary.** A user-editable list of `misheard -> correct`
  word/phrase pairs in `settings.conf`, applied deterministically after
  transcription — fixes recurring mistranscriptions (names, jargon) that the
  `initial_prompt` bias alone cannot pin down.

### Security
- `anyhow` bumped 1.0.102 → 1.0.103 (RUSTSEC-2026-0190).

## [2.3.8] — 2026-06-26

### Changed
- **Voice to Text now starts at login by default on Windows.** On first launch
  it registers itself under the per-user `HKCU\...\Run` key so it's ready after
  a reboot without hunting for the tray toggle. You stay in control: untick
  **"Start at login"** in the tray menu to turn it off, and that choice sticks
  (the default is applied exactly once, even across upgrades). Existing installs
  pick this up on their next launch. (TASK-VTT109)

## [2.3.7] — 2026-06-26

### Fixed
- **Releases publish smoothly, every time.** The release pipeline no longer
  hangs when GitHub is short on scarce macOS Intel (`macos-13`) runners. The
  Intel binary is now a best-effort job that attaches whenever it gets
  scheduled; un-drafting the release waits only on the three always-available
  runners (Linux, Windows, Apple-Silicon macOS). v2.3.6 sat as a draft for an
  hour because its Intel job was stuck in GitHub's queue — that can't block a
  publish anymore.
- **A warning can never again fail a release.** The release build now runs with
  lint off (`RUSTFLAGS` unset); `-D warnings` + clippy stay enforced on every PR
  by CI. This is the class of bug that silently broke v2.3.4/2.3.5.
- Every release job now has a timeout, so a hung or queued runner fails fast
  instead of stalling the whole release indefinitely.

### Note
- **Windows tray icon:** the icon is registered correctly — Windows 11 hides new
  notification-area icons in the overflow (`^`) flyout by default. Drag it onto
  the taskbar once to pin it. A branded app/installer icon is tracked separately
  (TASK-VTT108).

## [2.3.6] — 2026-06-26

### Fixed
- **Releases were silently broken since v2.3.4** — the new `autostart` module is
  only used by the Windows/macOS tray, so on Linux it was dead code, and the
  Linux release job builds with `-D warnings`. That failed the job that *creates*
  the GitHub release, so v2.3.4 and v2.3.5 never published. `autostart` is now
  cfg-gated to Windows/macOS. **This is the first release to actually ship the
  v2.3.4/2.3.5 work** (autostart, live status, macOS Intel binary, README).
- **CI green again**: removed a no-op `mem::forget` on a `Copy` handle (Windows
  singleton), and upgraded `quinn-proto` to 0.11.15 to close RUSTSEC-2026-0185.

## [2.3.5] — 2026-06-26

### Added
- **macOS Intel builds** — releases now attach **both** `vtt-macos-intel`
  (x86_64) and `vtt-macos-arm64` (Apple Silicon), each built natively. Previously
  only an Apple-Silicon binary shipped, which would not run on Intel Macs.

### Changed
- **README rewritten** — trimmed from 400+ lines, corrected (the old macOS
  Homebrew-cask install and `make`-based build steps no longer existed), and
  given a proper Windows install section. Deep per-platform detail now lives in
  `docs/PLATFORM-PARITY.md`.
- **Release-manager agent** updated to the report-by-default model (ship only on
  an explicit confirmation phrase) and the real CI multi-platform pipeline.

## [2.3.4] — 2026-06-26

### Added
- **Start at login (Windows)** — a "Start at login" toggle in the tray menu
  registers/removes a per-user `HKCU\…\Run` entry (no admin), so VTT launches
  with your session — parity with the Linux systemd user service (TASK-VTT094).
  Linux/macOS backends are stubbed pending their own mechanisms.

### Fixed
- **Tray status stuck on "Initializing…"** — the menu's status line was created
  then dropped, so it never updated. It now reflects the live state
  (Ready / Recording… / Transcribing… / model-download progress), and the tooltip
  matches.

## [2.3.3] — 2026-06-26

### Fixed (Windows ↔ Linux parity)
- **Tray icon now reflects state** — idle (green) → recording (red) → processing
  (amber) — and the tooltip shows the live status (e.g. model-download progress).
  The portable tray previously only logged these and never updated the icon.
- **Hotkey auto-repeat suppressed** — holding a non-toggle hotkey (F-key, letter)
  no longer re-fires the start event via OS key-repeat; only the first press and a
  real release count (parity with the Linux X11 auto-repeat filter).
- **Clipboard set as Ctrl+V fallback** after typing on Windows, matching the Linux
  behaviour where the transcription is also placed on the clipboard.

### Added
- `docs/PLATFORM-PARITY.md` — Linux↔Windows parity spec aggregated from the
  maintained feature cards, with a gap register tracking every remaining Windows
  difference.

## [2.3.2] — 2026-06-26

### Fixed
- **Windows tray icon had no menu**. The event loop never pumped the Win32
  message queue, so the tray icon's hidden window ignored clicks. It now pumps
  messages each tick — right-click opens the menu (quit, model, language).
- **Windows typed text was garbled** — leading/upper-case characters dropped and
  reordered (`[oice] esting … .VT`). Windows now types the whole transcript in one
  `enigo.text()` batch instead of the char-by-char path, and no longer probes for
  Linux-only `xdotool` on every transcription.
- **Tray model menu listed dead names**. The portable tray showed pre-2.0
  `W base` / `CT2 …` labels that didn't match any model; it now lists the real
  catalogue (`small`, `medium`, `large-v3-turbo`, …) so selecting one works.

## [2.3.1] — 2026-06-26

### Fixed
- **Windows: a console window opened alongside the tray app**. The binary was
  built as a console program, so launching it popped a terminal. It now builds
  as a windowed app (`windows_subsystem = "windows"`) — the system-tray icon is
  the only UI. Logs still go to `%APPDATA%\voice-to-text\logs\`. `--version` /
  `--help` still print when run from a terminal (the process re-attaches to the
  parent console).

## [2.3.0] — 2026-06-26

### Added
- **GPU acceleration on Windows (Vulkan)**: Whisper inference now runs on the
  GPU via Vulkan — vendor-neutral, so one build accelerates NVIDIA, AMD, and
  Intel without a CUDA Toolkit dependency. Verified on an NVIDIA RTX 2060 SUPER
  (`using Vulkan0 backend`, NV_coopmat2 tensor cores). Falls back to CPU
  automatically when no Vulkan device is present. Completes the Windows half of
  GPU acceleration; Linux already shipped Vulkan, macOS uses Metal.

### Fixed
- **Windows GPU build exceeded MAX_PATH**: whisper.cpp's nested
  `vulkan-shaders-gen` sub-build tripped MSBuild FileTracker's 260-char path
  limit (`FTK1011`). The Windows build now uses a short `CARGO_TARGET_DIR` so
  generated paths stay under the limit (`scripts/build-windows.ps1` sets this
  automatically; CI does too).

## [2.2.0] — 2026-06-25

### Added
- **Windows 11 support — verified on x86-64 hardware**: voice-to-text now
  builds, launches, and transcribes on Windows. Push-to-talk (Scroll Lock),
  in-process Whisper transcription (CPU), the system-tray menu, and text
  injection all run natively. Ships as a `.msi` installer.
- **End-to-end transcription test**: a synthesized-speech WAV fixture is decoded
  and run through the real Whisper engine, asserting the transcript — proves the
  audio→text path with no microphone (opt-in: `cargo test -- --ignored`).
- **`scripts/build-windows.ps1`**: reproducible Windows build that auto-discovers
  cmake and libclang from a standard VS Build Tools install, so a fresh machine
  builds without hunting for native toolchain bits.

### Fixed
- **Audio capture crashed at launch on Windows**: WASAPI shared mode rejects the
  hardcoded 16 kHz mono stream request that ALSA silently resampled on Linux. The
  capture path now falls back to the device's native format (e.g. 48 kHz),
  down-mixes to mono, and resamples to 16 kHz in software — so the app starts on
  Windows microphones instead of exiting with a stream-configuration error.
- **Debug recordings lost on Windows**: `write_wav` wrote to a hardcoded `/tmp`,
  which resolves to a non-existent `C:\tmp` on Windows. Now uses the platform
  temp directory (still `/tmp` on Linux).

## [2.1.0] — 2026-05-05

### Fixed
- **Sentence-starter characters dropped (I, T, Y, C)**: switched typing
  from `enigo` `Key::Unicode` to `xdotool type --clearmodifiers`,
  which sidesteps the XKB keysym remap race that was eating the first
  character of many sentences. Keystroke delay tuned to 12 ms.
  `xdotool` is now a package dependency; falls back to `enigo` if
  the binary is missing.
- **`apt upgrade` left old binary running**: postinst now `try-restart`s
  vtt-linux so the new version actually takes effect immediately.
- **Hotkey settings silently truncated**: keycode capture and press
  handler both reject values > 255 with a clear error instead of
  wrapping into garbage.
- **Stale postrm message** referenced v1.x cache paths — corrected for
  v2.0+ layout.
- **About dialog** now reads "Hold and release" — matches push-to-talk.

### Added
- **`--version` / `-V` and `--help` / `-h` flags**: print and exit
  before GTK / singleton init, so users can confirm the installed
  version from a terminal without launching the tray.
- **Singleton lock surfaces the holding PID** with a ready-to-paste
  `kill` command instead of a bare "already running".
- **Worker-thread death is surfaced via `notify-send`** instead of
  silently dropped.
- **Helpful error when no default input device is configured**, with
  a pointer at `pavucontrol`.
- **Version logged on startup** for easier issue triage.

### Changed
- **Tray model menu** is now derived from the `MODELS` catalogue —
  adding a model in `src/models.rs` automatically wires it into the
  UI.
- **`resolve_variant`'s `.en` check** is derived from `MODELS` rather
  than hardcoded.
- **`singleton_lock` runs before `logging::init`** so failed start
  attempts no longer pollute the daily log.
- **`vtt.service`**: dropped stale `PYTHONPATH`, now passes
  `XDG_SESSION_TYPE` for Wayland/X11 detection.

### Internal
- GitHub Actions CI with `cargo audit`, pre-push hook for offline
  developer checks.
- 63+ unit tests across audio, settings, models, tray, logging,
  hotkey, and transcribe modules.
- ADR-0004 documents the lightweight regression testing approach.
- SECURITY.md added for responsible disclosure.
- Dead code cleanup, lintian fixes, unused `gdk` dependency removed.
- `Cargo.lock` v3 guard added to `scripts/release-ppa.sh`.

---

## [2.0.5] — 2026-04-20

### Fixed
- **Typing stopped at £/é/—**: non-ASCII characters now type directly
  via `Key::Unicode`, not clipboard paste. The previous Ctrl+V paste
  was silently dropped in Claude Code TUI, Slack, and any app that
  doesn't bind Ctrl+V to paste.
- **Tray "Logs" submenu was stale on first open**: rebuilds now fire
  on the parent menu's `show` signal so the first hover shows today's
  log file.
- **Silent failure when log dir unreadable**: tray now renders an
  explicit `(log dir unreadable: …)` item instead of an empty menu.

### Added
- `--version` / `-V` and `--help` / `-h` CLI flags.
- 45 unit tests on pure helpers (settings, models, prefix composition,
  filler detection, log labels, audio truncation, legacy model-name
  migration, portable hotkey mapping).
- GitHub Actions CI — fmt, clippy (with `-D warnings`), test, build
  on every push and PR.
- Local pre-push git hook matching CI.
- `CHANGELOG.md` (this file).
- `ADR-0004` documenting the testing strategy.

### Changed
- Release script pins the pbuilder mirror to `mirror.bytemark.co.uk`,
  uses apt-based satisfydepends, and unsets `TMPDIR` before calling
  pbuilder. Four environmental chroot bugs fixed permanently.
- 60+ lines of dead code removed (`Settings::config_dir`,
  `logging::is_enabled/get_path`, `transcribe::load_wav`,
  `WorkItem::SwitchModel`, `TrayState.indicator/status_item`,
  `Audio.max_samples`, `RecordingResult::TooShort(f32)/TooQuiet(i16)`
  simplified to unit variants).
- README troubleshooting section updated for v2.0 (Vulkan not CUDA,
  per-day log filenames, no CT2 backend).
- All 27 clippy warnings fixed to zero.

---

## [2.0.4] — 2026-04-19

### Fixed
- `postinst` reads the installed version via `dpkg-query` instead of
  hardcoding it — no more stale banner after upgrades.

### Added
- First release to flow through the mandatory pbuilder chroot gate.
  Both noble and jammy chroots verified locally before any dput.

---

## [2.0.3] — 2026-04-19

### Fixed
- `debian/control` uses `libasound2t64 | libasound2` alternation so
  the package builds and installs on both jammy (22.04) and noble
  (24.04).

---

## [2.0.2] — 2026-04-19

### Changed
- `debian/rules` ships a pre-built `vtt-linux.prebuilt` binary instead
  of running `cargo build` in the Launchpad builder. Ubuntu Noble
  ships cargo 1.75 which cannot parse edition-2024 Cargo.toml
  manifests that modern crates use.

---

## [2.0.1] — 2026-04-19

### Fixed
- `Cargo.lock` pinned to format v3 for Launchpad Noble compatibility.

---

## [2.0.0] — 2026-04-16

### Changed
- **Rust rewrite**: replaced ~4,200 lines of C + 500 lines of Python
  with ~2,700 lines of Rust. The Whisper model now loads once
  in-process and remains resident for sub-second transcription
  regardless of clip length or model size — up from 3-8s per press
  on v1.x.

### Added
- Vulkan GPU acceleration (NVIDIA + AMD + Intel) — no CUDA Toolkit
  required.
- whisper-rs in-process inference (replaces Python + faster-whisper
  subprocess).
- GGML model download on demand from ggerganov/whisper.cpp HuggingFace.
- Default `ggml-small.en.bin` downloaded on first install via postinst.

### Removed
- Python runtime dependency (no `python3`, `pip install`,
  `faster-whisper`, `ctranslate2`).
- CT2 model caches at `~/.cache/huggingface/` no longer used — GGML
  models re-download on first selection.
- The `tiny`, `base`, and `distil-v3` models (replaced by
  `large-v3-turbo`, comparable speed, higher quality).

---

## Earlier

For releases before 2.0.0, see `debian/changelog`.
