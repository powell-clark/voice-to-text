# Changelog

All user-facing changes are documented here. Installed versions on
Debian/Ubuntu can also be inspected via `apt changelog voice-to-text`
or `vtt-linux --version`.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
versioning: [SemVer](https://semver.org/spec/v2.0.0.html).

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
