# Platform parity specification — Linux ↔ Windows (↔ macOS)

**Purpose.** Define, capability by capability, what voice-to-text must do on every
platform, so Windows can be driven to full parity with Linux and neither platform
drifts. This spec is **aggregated from the maintained feature cards**
(`CONSCIOUSNESS/features/`, status `[maintained]`) — those cards are the source of
truth for *what the capability is*; this document adds *how each platform realises
it and whether it does yet*.

**Method.** Each section names the governing `FEAT-VTT*` card, states the canonical
behaviour, then gives the per-platform mechanism and status. Status: ✅ done ·
🟡 partial/unverified · ❌ missing. Every ❌/🟡 maps to a `TASK-VTT*`.

**Source cards (maintained):** VTT001, 004, 005, 006, 008, 010, 011, 012, 013, 014,
015, 016, 017, 022, 023, 026, 027, 028, 035.

---

## 0. Platform path conventions (cross-cutting)

The Linux cards assume XDG paths; Windows needs a defined equivalent. Canonical:

| Data | Linux | Windows | Status |
|------|-------|---------|--------|
| Settings | `~/.config/voice-to-text/settings.conf` | `%APPDATA%\voice-to-text\settings.conf` (via `dirs::config_dir`) | 🟡 verify |
| Model cache (user) | `~/.cache/voice-to-text/models/` | `%LOCALAPPDATA%\voice-to-text\models\` (via `dirs::cache_dir`) | ✅ |
| Model cache (shared) | `/usr/share/voice-to-text/models/` | n/a (per-user only) | n/a |
| Logs | `~/.local/share/voice-to-text/` | `%APPDATA%\voice-to-text\logs\` | ✅ |
| Recordings archive | `~/.local/share/voice-to-text/recordings/` | `%LOCALAPPDATA%\voice-to-text\recordings\` | ✅ |

> `models.rs::system_cache()` returns the Linux `/usr/share` path unconditionally;
> on Windows it simply never exists so the user cache is used. Harmless but worth a
> `cfg` guard for cleanliness. **Gap → TASK-VTT097.**

---

## 1. Capture & transcription core (cross-platform)

### 1.1 Push-to-talk recording — FEAT-VTT001
Canonical: hold hotkey → record 16 kHz mono f32 via cpal → release → transcribe;
silence/noise filtered before send.
- **Linux:** cpal (ALSA), 16 kHz requested directly. ✅
- **Windows:** cpal (WASAPI). WASAPI shared mode rejects 16 kHz, so capture opens at
  the device's native rate and **resamples to 16 kHz** (audio.rs). ✅ (v2.2.0)
- Parity: ✅.

### 1.2 In-process model worker — FEAT-VTT022 · Pure Rust — FEAT-VTT023
Canonical: model loaded once per process, reused; no Python; sub-second warm latency.
- **Linux / Windows:** identical whisper-rs in-process worker. ✅ both.

### 1.3 GPU acceleration — FEAT-VTT024
Canonical: GPU inference, vendor-neutral, CPU fallback when absent.
- **Linux:** Vulkan. ✅  **Windows:** Vulkan (v2.3.0). ✅  **macOS:** Metal. ✅

### 1.4 Multi-language — FEAT-VTT006 · initial_prompt — FEAT-VTT011 · max duration — FEAT-VTT014
Canonical: language via settings (`auto`/code); initial_prompt biases inference;
auto-stop at 300 s.
- All settings-driven in shared code (`settings.rs`, `whisper.rs`, `audio.rs`). ✅ all platforms.

### 1.5 Model download + verify — FEAT-VTT026
Canonical: download missing GGML from HF over HTTPS, SHA-256 verify, atomic rename,
progress callback ≥ every 256 KB.
- **Linux / Windows:** shared `models.rs`. ✅ download/verify.
- **Windows gap:** progress is emitted as `UiMessage::SetStatus` but the portable tray
  only *logs* it — the user sees no tray progress. **Gap → TASK-VTT093** (status wiring).

### 1.6 Model catalogue (turbo/distil) — FEAT-VTT017
Canonical: `large-v3-turbo` etc. selectable from the tray; selection downloads+loads.
- **Linux:** GTK model submenu from catalogue. ✅
- **Windows:** portable model submenu now generated from `models::MODELS` (v2.3.2). ✅
  (Selection requires the tray menu to open — fixed v2.3.2.)

---

## 2. Tray UI — FEAT-VTT004 (LINUX-MECHANISM → Windows equivalent)

Canonical: a tray icon that (a) shows status Ready→Recording→Transcribing→Ready,
(b) reflects state visually, (c) offers a model submenu (current checked),
(d) a language submenu, (e) a Logs submenu (last N entries), (f) Quit.

| Sub-capability | Linux (GTK/AppIndicator) | Windows (tray-icon/muda) | Status |
|---|---|---|---|
| Icon visible | ✅ | ✅ | ✅ |
| Menu opens on click | ✅ | ✅ Win32 msg pump (v2.3.2) | ✅ |
| Status text | ✅ | ❌ logged only, not shown | TASK-VTT093 |
| Icon colour reflects state | ✅ | ❌ never calls set_icon | TASK-VTT093 |
| Model submenu | ✅ | ✅ (v2.3.2) | ✅ |
| Language submenu | ✅ | ✅ present | ✅ |
| Logs submenu | ✅ last-N on hover | ❌ absent | TASK-VTT098 |
| Quit | ✅ | ✅ | ✅ |

**Windows tray parity gaps:** icon/status state (TASK-VTT093), Logs submenu (TASK-VTT098).

---

## 3. Text injection — FEAT-VTT005 · Clipboard — FEAT-VTT012 (LINUX-MECHANISM)

Canonical: type transcription into the focused app at the cursor (no paste gesture),
Unicode-correct (£ é ñ emoji), AND set the clipboard simultaneously as a Ctrl+V
fallback; no duplicate characters.

- **Linux:** `xdotool type` (primary) → enigo fallback; clipboard via `xclip`. ✅
- **Windows:** `enigo.text()` whole-string batch (v2.3.2 — fixes dropped/reordered
  chars). 🟡 **Unicode not yet verified on hardware**; **clipboard is NOT set** as a
  fallback (the Linux `xclip` path is cfg-gated out). **Gaps:**
  - Unicode (£ é ñ emoji) verification on Windows → TASK-VTT092 acceptance (open).
  - Set the Windows clipboard after typing (arboard) for Ctrl+V parity → TASK-VTT099.

---

## 4. Hotkey robustness — FEAT-VTT013 (LINUX-MECHANISM)

Canonical: holding the key yields exactly one recording — auto-repeat keypresses are
discarded; release stops at the right moment; no perceptible delay.

- **Linux:** `XkbSetDetectableAutoRepeat` + manual `XPeekEvent` pairing filter
  (`hotkey/linux.rs`). ✅
- **Windows:** `rdev::listen` fires `Down` on **every** `KeyPress`. Scroll Lock (a
  toggle key) does not auto-repeat, so the default works — but an F-key or letter
  hotkey *will* auto-repeat and re-fire `Down`, restarting state. **Gap → TASK-VTT100**
  (suppress rdev key-repeat: ignore a `Down` while already pressed).

---

## 5. Lifecycle: autostart & background — FEAT-VTT015 (LINUX-MECHANISM)

Canonical: starts with the desktop session and stays resident with input/tray access.
- **Linux:** systemd `--user` service inheriting DISPLAY/XAUTHORITY. ✅
- **Windows:** ❌ no autostart. **Gap → TASK-VTT094** (HKCU Run / Startup shortcut + tray toggle).
- **macOS:** ❌ LaunchAgent. (parity backlog)

---

## 6. Packaging, distribution, updates — FEAT-VTT008/016/027/028

Canonical: install via a native channel; default model present for offline first-run;
one-command release; updates picked up automatically.

| Aspect | Linux | Windows | Status |
|---|---|---|---|
| Installer | `.deb` + Launchpad PPA (VTT008) | `.msi` via cargo-wix (VTT027 analogue) | ✅ both |
| Cargo-built binary in package | ✅ (VTT027) | ✅ (MSI wraps `vtt-linux.exe`) | ✅ |
| Default model pre-provisioned | ✅ postinst (VTT028) | ❌ first-run download only | TASK-VTT101 |
| One-command release | `release-ppa.sh` (VTT016) | CI on tag (release.yml) | ✅ both |
| Update mechanism | `apt upgrade` | `.msi` in-place (MajorUpgrade); no in-app check | TASK-VTT095 |

---

## 7. Testing & CI — FEAT-VTT035

Canonical: cargo tests gate every change; CI on every push.
- **Tests:** pure-logic, cross-platform; Windows-specific tests added (audio resample,
  E2E transcription). ✅
- **CI:** ubuntu (fmt+clippy+test+build), windows (build, Vulkan), macos (build). 🟡
  Windows/macOS are build-only (no test run) — full cross-platform test matrix is
  deferred (TASK-VTT048).

---

## Parity gap register (Windows → Linux)

| Gap | Task | Priority | Status |
|---|---|---|---|
| Tray icon colour + status text not shown | TASK-VTT093 | p1 | ✅ v2.3.3 |
| Clipboard not set as paste fallback | TASK-VTT099 | p2 | ✅ v2.3.3 |
| Hotkey auto-repeat not suppressed (non-toggle keys) | TASK-VTT100 | p2 | ✅ v2.3.3 |
| Tray Logs submenu absent | TASK-VTT098 | p2 | ❌ |
| Unicode typing unverified (£ é ñ emoji) | TASK-VTT092 (AC) | p1 | 🟡 needs hardware check |
| Autostart on login | TASK-VTT094 | p2 | ❌ |
| Default model not pre-provisioned by installer | TASK-VTT101 | p3 | ❌ |
| In-app update check | TASK-VTT095 | p2 | ❌ |
| `system_cache()` returns Linux path on Windows | TASK-VTT097 | p3 | ❌ |
| Cross-platform test matrix (run tests on win/mac) | TASK-VTT048 | p3 | ❌ |

**Parity is reached when every row above is ✅.** This register is the definition of
done for "Windows feature parity with Linux". Update this spec whenever a maintained
feature card changes or a new capability lands on one platform.
