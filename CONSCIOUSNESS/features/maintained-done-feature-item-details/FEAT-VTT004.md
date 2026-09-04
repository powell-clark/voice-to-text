---
id: FEAT-VTT004
status: maintained
kano: must-have
---

# FEAT-VTT004: Linux system tray with GTK3 and AppIndicator

## Description
On Linux, VTT displays a system tray icon using GTK3 and the `libappindicator3` library. The tray provides status display (Recording / Transcribing / Ready / Loading model), a model selection submenu, a logs submenu, and settings/quit actions. This is the primary user interface on Linux.

## Acceptance Criteria
- [x] **AC-1** — Tray icon appears in the system tray on Ubuntu 24.04 (GNOME + AppIndicator extension) after `systemctl --user start vtt` — verified in daily use
- [x] **AC-2** — Status text updates correctly through the lifecycle: Ready → Recording... → Transcribing... → Ready — verified
- [x] **AC-3** — Model submenu shows available models with the current model checked — verified
- [x] **AC-4** — Logs submenu shows the last N transcription log entries on hover — verified in v2.0.5 (fixed in TASK-VTT055)
- [x] **AC-5** — Quit action stops the service cleanly — verified
- [x] **AC-6** — No GTK warnings on launch (`G_MESSAGES_DEBUG=all`) under normal use — verify after fresh install

## Cross-platform acceptance criteria (DIRECT-VTT005 parity spec)
Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 5 — system tray/menu). The tray capability spans this card (Linux/GTK) and the portable tray (`src/tray/portable.rs`) used on macOS + Windows.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux — ✅ works** (this card)
- [x] GTK3 + libappindicator tray: status, model picker, Logs submenu, hotkey picker, quit — `src/tray/linux.rs`

**🍎 macOS — 🟡 partial**
- [ ] `tray/portable.rs` (tray-icon + muda) shows status, model picker, quit — compiles and ships, but a menu-bar item needs NSApplication/`.app` activation; untested without the `.app` (FEAT-VTT029)
- [ ] No Logs submenu and no hotkey picker (parity gap vs Linux)

**🪟 Windows — 🟡 partial**
- [x] `tray/portable.rs` shows status, model picker, logging + autostart toggles, quit; Win32 message pump drives it — `src/main.rs:399-419` (TASK-VTT091)
- [ ] Logs submenu missing (parity gap vs Linux) — TASK-VTT098
- [ ] No hotkey picker (config-file only)

## Linked Tasks
- TASK-VTT004, TASK-VTT019

## Parent Story
- STORY-VTT001
