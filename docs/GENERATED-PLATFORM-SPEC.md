# Generated per-platform spec (ADR-0008)

**Generated file — do not hand-edit.** Re-run `scripts/generate-platform-spec.sh`
after editing a card's Cross-platform acceptance criteria section instead.

Aggregated from each maintained card's own "Cross-platform acceptance
criteria (DIRECT-VTT005 parity spec)" section — nothing here is
hand-written, so it cannot drift from what the cards say. Cards without
that section are not listed (see TASK-VTT166 for the exclusion
reasoning — a few cards are single-platform or `status: done` and
genuinely don't qualify).

Generated: 2026-09-05 07:12 UTC

---

# Platform path conventions

Moved from `docs/PLATFORM-PARITY.md` §0 (TASK-VTT172) — this table is
cross-cutting, not tied to any single feature card, so it needs a home that
survives `PLATFORM-PARITY.md`'s retirement.
`scripts/generate-platform-spec.sh` prepends this file verbatim.

The Linux cards assume XDG paths; Windows needs a defined equivalent. Canonical:

| Data | Linux | Windows | Status |
|------|-------|---------|--------|
| Settings | `~/.config/voice-to-text/settings.conf` | `%APPDATA%\voice-to-text\settings.conf` (via `dirs::config_dir`) | 🟡 verify |
| Model cache (user) | `~/.cache/voice-to-text/models/` | `%LOCALAPPDATA%\voice-to-text\models\` (via `dirs::cache_dir`) | ✅ |
| Model cache (shared) | `/usr/share/voice-to-text/models/` | n/a (per-user only) | n/a |
| Logs | `~/.local/share/voice-to-text/` | `%APPDATA%\voice-to-text\logs\` | ✅ |
| Recordings archive | `~/.local/share/voice-to-text/recordings/` | `%LOCALAPPDATA%\voice-to-text\recordings\` | ✅ |

`models.rs::system_cache()` is `#[cfg(target_os = "linux")]`-gated (returns
`None` on Windows/macOS, falling through to the user cache) — corrected
2026-09-05, TASK-VTT172; TASK-VTT097 (which tracked this exact gap) is done.

---

## FEAT-VTT001: Push-to-talk voice recording

Anchored to `docs/PLATFORM-PARITY.md` §1.1. Capture is shared `cpal`-based code (`src/audio.rs`) with one platform-specific branch for sample-rate negotiation.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates.

**🐧 Linux — ✅ works** (this card)
- [x] `cpal` (ALSA), 16 kHz requested directly from the device — `src/audio.rs`

**🪟 Windows — ✅ works**
- [x] `cpal` (WASAPI). WASAPI shared mode rejects a direct 16 kHz open, so capture opens at the device's native rate and resamples to 16 kHz in software — `src/audio.rs` (v2.2.0)

**🍎 macOS — ✅ works**
- [x] `cpal` (CoreAudio), same shared-code path as Linux/Windows; no macOS-specific branch exists in `src/audio.rs`


---

## FEAT-VTT004: Linux system tray with GTK3 and AppIndicator

Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 5 — system tray/menu). The tray capability spans this card (Linux/GTK) and the portable tray (`src/tray/portable.rs`) used on macOS + Windows.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux — ✅ works** (this card)
- [x] GTK3 + libappindicator tray: status, model picker, Logs submenu, hotkey picker, quit — `src/tray/linux.rs`

**🍎 macOS — 🟡 partial**
- [ ] `tray/portable.rs` (tray-icon + muda) shows status, model picker, quit — compiles and ships, but a menu-bar item needs NSApplication/`.app` activation; untested without the `.app` (FEAT-VTT029)
- [x] CORRECTED (2026-09-05, TASK-VTT166): Logs submenu IS present — `src/tray/portable.rs` is shared between Windows and macOS, so TASK-VTT098's fix applies here too (compiles and ships; untested without the `.app` bundle, same caveat as the line above)
- [ ] No hotkey picker (parity gap vs Linux) — TASK-VTT138/TASK-VTT170

**🪟 Windows — 🟡 partial**
- [x] `tray/portable.rs` shows status, model picker, logging + autostart toggles, quit; Win32 message pump drives it — `src/main.rs:399-419` (TASK-VTT091)
- [x] CORRECTED (2026-09-05, TASK-VTT166): Logs submenu shipped — TASK-VTT098 is done (`src/tray/portable.rs`'s `refresh_logs_submenu`)
- [ ] No hotkey picker (config-file only) — TASK-VTT138/TASK-VTT170 track the toolkit spike + follow-on work


---

## FEAT-VTT005: Text injection into focused application (Linux via XTest)

Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 3 — text injection).

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux — ✅ works**
- [x] Text types at the cursor via `xdotool type` (primary) with `enigo`/x11rb fallback — `src/typing.rs:41-152`
- [x] £/é/ñ/emoji type correctly; no duplicate characters in X11/XWayland

**🍎 macOS — 🟡 partial**
- [ ] Text types at the cursor via `enigo` once Accessibility permission is granted — code path exists (`src/typing.rs` non-Windows branch) but is untuned char-by-char and `xdotool` is always absent
- [ ] Accessibility-permission prompt presented to the user — BLOCKED on the `.app` bundle (FEAT-VTT029); a raw binary cannot register the prompt
- [ ] Verified typing into Safari, Notes, Terminal on a shipped build

**🪟 Windows — 🟡 partial**
- [x] Text types at the cursor via batched `enigo.text()` SendInput per newline segment — `src/typing.rs:69-105`
- [ ] CORRECTED (2026-09-05, TASK-VTT172): the batched-typing fix for dropped/reordered characters is implemented and TASK-VTT092 is closed, but that task's own acceptance criteria are still unchecked and its card says "Needs on-hardware verification" — this line previously claimed `[x]` done, overstating it. Unicode (£/é/ñ/emoji) on real Windows hardware has not been confirmed — TASK-VTT092 (AC unverified)


---

## FEAT-VTT006: Multi-language support with auto-detection

Anchored to `docs/PLATFORM-PARITY.md` §1.4 and §2 (tray language submenu). Language selection itself is fully shared (`settings.rs`, `whisper.rs`); the runtime English/Multilingual toggle lives in each platform's own tray module.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates.

**🐧 Linux — ✅ works** (this card)
- [x] `settings.conf` `language` field drives `WhisperEngine` — shared code
- [x] Tray language submenu (English / Multilingual) with auto `.en`-suffix switching on the model name — `src/tray/linux.rs`

**🪟 Windows — ✅ works**
- [x] Same shared `settings.rs`/`whisper.rs` language handling
- [x] Tray language submenu (English / Multilingual) — `src/tray/portable.rs`

**🍎 macOS — 🟡 partial**
- [x] Same shared `settings.rs`/`whisper.rs` language handling (identical binary code path to Windows)
- [ ] Tray language submenu compiles (`src/tray/portable.rs` is shared across Windows/macOS) but is untested without a `.app` bundle (FEAT-VTT029) to actually run the menu-bar item


---

## FEAT-VTT008: APT PPA distribution for Ubuntu (Noble + Jammy)

Anchored to `docs/PLATFORM-PARITY.md` §6 (packaging/distribution). Each platform's installer is a genuinely different mechanism (apt/dpkg vs `cargo wix` MSI vs none yet) achieving the same "one native-feeling install" outcome — same shape as FEAT-VTT004's tray section comparing GTK vs tray-icon/muda.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates.

**🐧 Linux — ✅ works** (this card)
- [x] `.deb` via Launchpad PPA, `debian/postinst` pre-provisions the default model (FEAT-VTT028)
- [x] Branded icon + version info embedded in the binary is N/A here (no Windows-style VERSIONINFO resource on ELF); `.desktop` file carries the app metadata instead

**🪟 Windows — 🟡 partial**
- [x] `.msi` via `cargo wix` (`wix/main.wxs`), built only from `release.yml` on a tag push — Start Menu shortcut, branded icon + ProductVersion/FileVersion resource (TASK-VTT108)
- [ ] Default model NOT pre-provisioned by the installer — first launch downloads ~465 MB with only a tray-tooltip progress indicator, no `.deb`-postinst equivalent — TASK-VTT101 (open)
- [ ] In-place upgrade (`MajorUpgrade`) not yet verified against a real prior-version install — TASK-VTT168 (open)

**🍎 macOS — ❌ missing**
- [ ] No `.app` bundle or `.dmg`/installer pipeline exists yet — blocked on TASK-VTT040


---

## FEAT-VTT012: Clipboard paste via xclip subprocess

Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 4 — clipboard paste fallback).

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux — ✅ works**
- [x] Transcription written to the clipboard via `xclip`/`arboard`; `xclip` in `debian/control` Depends — `src/typing.rs:187-252`, `debian/control:36` (TASK-VTT110)
- [x] Manual Ctrl+V pastes correctly; no crash across 50+ writes

**🍎 macOS — 🟡 partial (TASK-VTT114)**
- [x] Transcription reaches the clipboard via `arboard` (manual Cmd+V works) — `src/typing.rs:187-216`
- [ ] Auto-paste keystroke uses **Cmd+V** on macOS — currently sends Ctrl+V (no-op); tracked as TASK-VTT114

**🪟 Windows — ✅ works**
- [x] Transcription written to the clipboard via `arboard` for manual Ctrl+V (no auto-paste, parity with the Linux fallback) — `src/typing.rs:99-103` (TASK-VTT099)


---

## FEAT-VTT013: X11 key auto-repeat filtering

Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 6 — hotkey auto-repeat handling).

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux — ✅ works** (this card)
- [x] Held hotkey = exactly one recording — X11 `XkbSetDetectableAutoRepeat` + manual KeyPress/KeyRelease pairing — `src/hotkey/linux.rs`

**🍎 macOS — ✅ works**
- [x] `rdev` listener filters auto-repeat; held key = one recording — `src/hotkey/portable.rs` (TASK-VTT100)
- [ ] Requires Input-Monitoring permission (no prompt flow without `.app`, FEAT-VTT029)

**🪟 Windows — ✅ works**
- [x] `rdev` listener filters auto-repeat; live keycode remap via SetKeycode — `src/hotkey/portable.rs` (TASK-VTT100)


---

## FEAT-VTT017: large-v3-turbo and distil-large-v3 model support

Anchored to `docs/PLATFORM-PARITY.md` §1.6. The catalogue itself (`src/models.rs::MODELS`) is fully shared; only the tray menu construction around it differs.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates.

**🐧 Linux — ✅ works** (this card)
- [x] `rebuild_model_menu` filters `MODELS` to multilingual entries only, title-cases the label, and auto-strips `.en` when switching to multilingual — `src/tray/linux.rs`

**🪟 Windows — 🟡 partial**
- [x] Model submenu is generated from the real `MODELS` catalogue (v2.3.2), no hardcoded list
- [ ] Iterates ALL of `MODELS` unfiltered and shows the raw `info.name` — no multilingual-only filter, no title-casing, no `.en` auto-switch — so Windows/macOS can show both `.en` and non-`.en` variants as separate entries where Linux collapses them into one radio group (documented in ADR-0005, still true as of this migration — `git grep multilingual src/tray/portable.rs` returns nothing)

**🍎 macOS — 🟡 partial**
- [ ] Same `src/tray/portable.rs` code as Windows — same unfiltered-list gap, untested without a `.app` bundle (FEAT-VTT029)


---

## FEAT-VTT022: Whisper model loaded once in-process worker thread

Anchored to `docs/PLATFORM-PARITY.md` §1.2. The worker loop (`load_engine` in `src/main.rs`, `whisper-rs` engine) has no `target_os` branches at all — this is one code path shared byte-for-byte across every platform.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates.

**🐧 Linux — ✅ works** (this card)
- [x] Model loaded once at worker-thread startup, reused for every transcription — `src/main.rs::load_engine`

**🪟 Windows — ✅ works**
- [x] Identical shared code path — `main.rs` has several `target_os` branches elsewhere (tray/hotkey wiring), but none fall inside `load_engine` or the transcription worker loop itself (verified: no `target_os` hit between those functions' line range)

**🍎 macOS — ✅ works**
- [x] Same shared code path as Windows; no macOS-specific branch either


---

## FEAT-VTT026: Automatic GGML model download from HuggingFace

Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 8 — model download). Behaviour is identical on all platforms (one `src/models.rs` path); the only gap is cross-cutting.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux / 🍎 macOS / 🪟 Windows — ✅ works (uniform)**
- [x] Selecting an uncached model downloads from HuggingFace over HTTPS (rustls) with tray progress and atomic `.tmp`→rename — `src/models.rs:135-193`
- [x] Cache path resolves per-OS via `dirs::cache_dir`: `~/.cache` (Linux), `~/Library/Caches` (macOS), `%LOCALAPPDATA%` (Windows)

**Cross-cutting gap (all platforms) → TASK-VTT112**
- [ ] In-app download verifies the bytes against a stored expected SHA-256 — NOT implemented (only the Linux `debian/postinst` pre-download hard-verifies, TASK-VTT110)


---

## FEAT-VTT028: Default model pre-downloaded via postinst

Anchored to `docs/PLATFORM-PARITY.md` §6. The user-facing capability is "offline-ready immediately after install/first launch" — Linux reaches it at install time (root, network, postinst), Windows only at runtime (no installer-time hook exists).

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates.

**🐧 Linux — ✅ works** (this card)
- [x] `debian/postinst` downloads the default model during `apt install`, before the user ever opens the app

**🪟 Windows — 🟡 partial**
- [x] No surprise data loss: `src/main.rs::load_engine` runs at worker-thread startup (before any hotkey action reaches the transcribe step), so a recording made mid-download is queued, not dropped
- [x] Progress IS shown — `UiMessage::SetStatus("Downloading {model}... {pct}%")` sets the tray tooltip (`src/tray/portable.rs`) — same shared code Linux uses
- [ ] Unlike Linux, this is tooltip-only (hover-to-see), not a proactive install-time step — a first-time user who doesn't hover has no visible signal that a ~465 MB download is happening — TASK-VTT101 (open)

**🍎 macOS — ❌ missing**
- [ ] Same runtime download-with-tooltip mechanism as Windows would apply once a `.app` bundle exists (FEAT-VTT029, blocked on TASK-VTT040) — untested, no bundle to test it in


---

## FEAT-VTT035: Automated regression testing

Anchored to `docs/PLATFORM-PARITY.md` §7. This card's own Scope section already excludes the full cross-platform matrix ("Cross-platform CI matrix (TASK-VTT048 covers macOS + Windows separately)") — this table tracks the CURRENT state of that gap, not a claim that it's closed.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates.

**🐧 Linux — ✅ works** (this card)
- [x] `ubuntu-24.04` job runs `cargo fmt --check`, `cargo clippy --release --all-targets -- -D warnings`, `cargo test --release`, `cargo build --release` — `.github/workflows/ci.yml`

**🪟 Windows — 🟡 partial**
- [x] `windows-latest` (x86_64-msvc, Vulkan) and `windows-11-arm` (aarch64-msvc, CPU) jobs run `cargo build --all-targets` — compile-green gate
- [ ] Neither Windows job runs `cargo test` — the 209 tests never execute on real Windows CI, only compile — TASK-VTT048 (open)

**🍎 macOS — 🟡 partial**
- [x] `macos-latest` (arm64, Metal) job runs `cargo build --all-targets`
- [ ] Same gap as Windows: build-only, no `cargo test` execution — TASK-VTT048 (open)

---

## Open gaps

Every unchecked (`- [ ]`) bullet across the cards above that names a
task — extracted, not hand-maintained. A bullet with no task reference
is listed as-is; check the card itself for context.

**FEAT-VTT004: Linux system tray with GTK3 and AppIndicator**
`tray/portable.rs` (tray-icon + muda) shows status, model picker, quit — compiles and ships, but a menu-bar item needs NSApplication/`.app` activation; untested without the `.app` (FEAT-VTT029)
No hotkey picker (parity gap vs Linux) — TASK-VTT138/TASK-VTT170
No hotkey picker (config-file only) — TASK-VTT138/TASK-VTT170 track the toolkit spike + follow-on work

**FEAT-VTT005: Text injection into focused application (Linux via XTest)**
Text types at the cursor via `enigo` once Accessibility permission is granted — code path exists (`src/typing.rs` non-Windows branch) but is untuned char-by-char and `xdotool` is always absent
Accessibility-permission prompt presented to the user — BLOCKED on the `.app` bundle (FEAT-VTT029); a raw binary cannot register the prompt
Verified typing into Safari, Notes, Terminal on a shipped build
CORRECTED (2026-09-05, TASK-VTT172): the batched-typing fix for dropped/reordered characters is implemented and TASK-VTT092 is closed, but that task's own acceptance criteria are still unchecked and its card says "Needs on-hardware verification" — this line previously claimed `[x]` done, overstating it. Unicode (£/é/ñ/emoji) on real Windows hardware has not been confirmed — TASK-VTT092 (AC unverified)

**FEAT-VTT006: Multi-language support with auto-detection**
Tray language submenu compiles (`src/tray/portable.rs` is shared across Windows/macOS) but is untested without a `.app` bundle (FEAT-VTT029) to actually run the menu-bar item

**FEAT-VTT008: APT PPA distribution for Ubuntu (Noble + Jammy)**
Default model NOT pre-provisioned by the installer — first launch downloads ~465 MB with only a tray-tooltip progress indicator, no `.deb`-postinst equivalent — TASK-VTT101 (open)
In-place upgrade (`MajorUpgrade`) not yet verified against a real prior-version install — TASK-VTT168 (open)
No `.app` bundle or `.dmg`/installer pipeline exists yet — blocked on TASK-VTT040

**FEAT-VTT012: Clipboard paste via xclip subprocess**
Auto-paste keystroke uses **Cmd+V** on macOS — currently sends Ctrl+V (no-op); tracked as TASK-VTT114

**FEAT-VTT013: X11 key auto-repeat filtering**
Requires Input-Monitoring permission (no prompt flow without `.app`, FEAT-VTT029)

**FEAT-VTT017: large-v3-turbo and distil-large-v3 model support**
Iterates ALL of `MODELS` unfiltered and shows the raw `info.name` — no multilingual-only filter, no title-casing, no `.en` auto-switch — so Windows/macOS can show both `.en` and non-`.en` variants as separate entries where Linux collapses them into one radio group (documented in ADR-0005, still true as of this migration — `git grep multilingual src/tray/portable.rs` returns nothing)
Same `src/tray/portable.rs` code as Windows — same unfiltered-list gap, untested without a `.app` bundle (FEAT-VTT029)

**FEAT-VTT026: Automatic GGML model download from HuggingFace**
In-app download verifies the bytes against a stored expected SHA-256 — NOT implemented (only the Linux `debian/postinst` pre-download hard-verifies, TASK-VTT110)

**FEAT-VTT028: Default model pre-downloaded via postinst**
Unlike Linux, this is tooltip-only (hover-to-see), not a proactive install-time step — a first-time user who doesn't hover has no visible signal that a ~465 MB download is happening — TASK-VTT101 (open)
Same runtime download-with-tooltip mechanism as Windows would apply once a `.app` bundle exists (FEAT-VTT029, blocked on TASK-VTT040) — untested, no bundle to test it in

**FEAT-VTT035: Automated regression testing**
Neither Windows job runs `cargo test` — the 209 tests never execute on real Windows CI, only compile — TASK-VTT048 (open)
Same gap as Windows: build-only, no `cargo test` execution — TASK-VTT048 (open)

