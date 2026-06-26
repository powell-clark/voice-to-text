# TASK-VTT091: Windows tray icon has no menu — pump the Win32 message loop

## Context

Reported by Emmanuel (2026-06-26) testing v2.3.0 on Windows: the tray icon
appears but left/right-clicking it shows no menu, so there's no way to quit or
change model/language from the tray.

## Root cause

`src/main.rs` runs the macOS/Windows event loop (the `cfg(not(linux))` branch)
as a sleep-and-`poll_menu()` loop. `poll_menu()` drains muda's `MenuEvent`
channel but nothing **pumps the Win32 message queue**. On Windows, `tray-icon`
creates a hidden message-only window that only processes clicks (and pops its
context menu) when `PeekMessage`/`TranslateMessage`/`DispatchMessage` run on the
thread that created the icon. With no pump, the icon is inert.

## Fix

On Windows, pump pending messages each loop tick before `poll_menu()`
(`windows-sys` `PeekMessageW`/`TranslateMessage`/`DispatchMessageW`). Keep the
plain poll loop for macOS (separate run-loop mechanism, tracked separately if it
shows the same symptom).

## Acceptance criteria

- [ ] Right-click (and left-click where applicable) on the Windows tray icon
      opens the menu
- [ ] Menu actions work — Quit exits, model/language selection applies
- [ ] Linux (GTK) and macOS paths unchanged

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Found-by: real use of v2.3.0 on Windows
