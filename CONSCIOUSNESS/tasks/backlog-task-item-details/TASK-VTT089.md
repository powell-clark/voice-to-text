# TASK-VTT089: Windows tray app pops a console window — build windowed

## Context

Reported by Emmanuel (2026-06-26) after installing the v2.3.0 MSI: launching the
app opened a terminal window alongside the tray icon. The binary was compiled as
a console (CUI) program, so Windows attaches a console on launch — wrong for a
system-tray app.

## Fix

`src/main.rs` now carries `#![cfg_attr(target_os = "windows", windows_subsystem
= "windows")]`, building a GUI (windowed) binary — PE subsystem 2, verified. The
tray icon is the only UI; logs still write to `%APPDATA%\voice-to-text\logs\`.
`--version` / `--help` re-attach to the launching terminal (AttachConsole) so the
CLI flags still print when run from a shell. No-op on Linux/macOS.

## Acceptance criteria

- [x] Windows binary is PE subsystem 2 (Windows GUI) — no console on launch
- [x] `--version` / `--help` still print when run from a terminal
- [x] Linux/macOS builds unaffected (attribute is cfg-gated)

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Found-by: real install of v2.3.0 MSI
