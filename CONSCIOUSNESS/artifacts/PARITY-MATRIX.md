# Voice-to-Text — Cross-Platform Parity Matrix

**Directive:** DIRECT-VTT005 (Cross-platform feature parity as a testable spec)
**Built:** 2026-06-26 from a code-grounded audit of each platform (Linux / macOS / Windows).
**Legend:** ✅ works · 🟡 partial (skeleton/buggy/incomplete) · ❌ missing · ➖ not applicable

This matrix IS the parity scorecard. Each row is a user-facing capability; the cell is its
true state per platform from the actual code/packaging — not the feature-card claim.

| # | Capability | 🐧 Linux | 🍎 macOS | 🪟 Windows | Tracking |
|---|------------|:------:|:------:|:------:|----------|
| 1 | Push-to-talk recording | ✅ | ✅ | ✅ | FEAT-VTT001 |
| 2 | In-process transcription (whisper-rs) | ✅ | ✅ | ✅ | FEAT-VTT022, FEAT-VTT023 |
| 3 | Text injection / typing | ✅ xdotool | 🟡 enigo, untuned | ✅ SendInput | FEAT-VTT005 |
| 4 | Clipboard paste fallback | ✅ | 🟡 Cmd+V bug | ✅ | FEAT-VTT012 |
| 5 | System tray / menu | ✅ GTK full | 🟡 no .app ctx | 🟡 no Logs submenu | FEAT-VTT004 / VTT030 |
| 6 | Configurable hotkey | ✅ tray picker | 🟡 config-only | 🟡 config-only | FEAT-VTT010, VTT013 |
| 7 | Voice prefix / initial prompt | ✅ | ✅ | ✅ | FEAT-VTT010, VTT011 |
| 8 | Model download + progress | ✅ | ✅ | ✅ | FEAT-VTT026 |
| 9 | GPU acceleration | ✅ Vulkan | ✅ Metal | ✅ Vulkan | FEAT-VTT024, VTT025 |
| 10 | Multi-language / auto-detect | ✅ | ✅ | ✅ | FEAT-VTT006 |
| 11 | Max-recording cap (5 min) | ✅ | ✅ | ✅ | FEAT-VTT014 |
| 11b | VAD silence auto-stop | ❌ | ❌ | ❌ | FEAT-VTT032 (not started) |
| 12 | Autostart / start-at-login | ✅ systemd unit | ❌ stub | ✅ HKCU Run | TASK-VTT094, VTT109 |
| 13 | Single-instance lock | ✅ flock | ✅ flock | ✅ named mutex | TASK-VTT044 |
| 14 | Packaging / real install path | ✅ apt PPA | 🟡 raw binary only | 🟡 MSI (draft, unsigned) | FEAT-VTT008/029/030 |
| 15 | Background service / daemon | ✅ systemd | ❌ none | ➖ by design (Run key) | FEAT-VTT015 |

## Read of the matrix

- **Linux is the reference platform** — full at every row except the two cross-cutting gaps (VAD, in-app hash verify).
- **Windows is close** — core loop works; remaining gaps are polish/packaging (Logs submenu, hotkey picker, signed MSI).
- **macOS is the weakest** — the code largely compiles cross-platform, but without a shipped `.app` several capabilities are skeleton/untested, and there are real bugs (Cmd+V).

## Cross-cutting gaps (all platforms)

- **VAD auto-stop** — push-to-talk only; no silence detection. FEAT-VTT032 / TASK-VTT050, not started.
- **In-app model checksum** — `models.rs` computes & logs sha256 on download but does NOT verify against an expected digest. TASK-VTT112. (Only the Linux `postinst` pre-download hard-verifies.)

## Per-platform gaps

### 🍎 macOS (biggest parity debt)
- **No real install path** — only a raw, unsigned `vtt-macos-arm64` binary; no `.app`, no DMG, Homebrew tap broken. → FEAT-VTT029 (.app), FEAT-VTT036 (Homebrew).
- **Clipboard paste broken** — auto-paste sends Ctrl+V; macOS needs Cmd+V. → **UNTRACKED → TASK-VTT114.**
- **Text injection untuned** — always falls to enigo char-by-char (xdotool absent); needs Accessibility-permission prompt flow (tied to .app). → FEAT-VTT005 / FEAT-VTT029.
- **Autostart missing** — `src/autostart.rs` macOS impl is a no-op stub. → TASK-VTT094 (LaunchAgent).
- **Menu bar untested** — `tray/portable.rs` compiles but a macOS menu-bar item needs NSApplication/.app activation. → FEAT-VTT029.
- **Metal card stale** — FEAT-VTT025 says "blocked", but Metal is compiled and ships in the binary. → card-accuracy fix.

### 🪟 Windows
- **MSI unsigned & draft** — no Authenticode signing. → TASK-VTT047; FEAT-VTT030 still backlog.
- **Tray Logs submenu missing** (Linux parity). → TASK-VTT098.
- **No hotkey picker in tray** (config-file only). → **UNTRACKED** (shared with macOS).
- Branding/icon (VTT108), model pre-provision (VTT101), binary rename (VTT102), update mechanism (VTT095), real-hardware smoke test (VTT082) — all backlog.

### 🐧 Linux
- **arm64** — amd64 only; no arm64 `.deb`.
- Autostart is packaging-driven (systemd unit), not the in-app toggle — intentional, but a parity asymmetry worth noting.

## Newly surfaced, untracked → to file
- **TASK-VTT114** — macOS clipboard auto-paste uses Ctrl+V; must be Cmd+V (capability 4).
- Hotkey picker absent from the portable tray (macOS + Windows) — capability 6.
- FEAT-VTT025 card says Metal "blocked" but it ships — card-accuracy fix.

## Next steps toward the testable spec (DIRECT-VTT005)
1. ✅ Step 1 — this matrix (audit current state).
2. 🟡 Step 2 (in progress) — per-platform acceptance criteria added to the variance-capability cards: FEAT-VTT005 (typing), FEAT-VTT012 (clipboard), FEAT-VTT004 (tray), FEAT-VTT013 (hotkey), FEAT-VTT026 (model download). Uniform-✅ capabilities (recording, transcription, prefix/prompt, multi-language, single-instance) keep a single shared criterion. Inherently per-platform capabilities already have their own cards (FEAT-VTT008 apt / FEAT-VTT029 .app / FEAT-VTT030 .msi packaging; FEAT-VTT015 + autostart).
3. ⬜ Step 3 — wire each acceptance criterion to an automated test (TASK-VTT080, per-feature test-status), so the matrix stays green-by-test, not by assertion.
