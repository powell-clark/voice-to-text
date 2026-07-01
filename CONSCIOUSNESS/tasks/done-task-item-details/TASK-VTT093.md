# TASK-VTT093: Windows/macOS tray icon never changes state (recording/processing)

## Context

Reported by Emmanuel (2026-06-26): the taskbar/tray icon doesn't change colour or
show a loading indicator while recording or transcribing.

## Root cause

`src/tray/portable.rs` spawns a `ui-updates` thread that receives `UiMessage`
(SetStatus / SetIcon) and only **logs** them (`[UI] Icon: recording`) — it never
calls `tray_icon.set_icon()` / `set_tooltip()`. The `TrayIcon` lives on the main
thread and muda/tray-icon handles are !Send, so the update must happen on the main
thread (in `poll_menu`), not the worker thread.

## Fix (proposed)

Route `UiMessage` to the main thread: store the `ui_rx` in the `Tray` struct and
drain it in `poll_menu()`, mapping state → icon colour (idle=green, recording=red,
processing=amber) via the existing `create_icon()`, and set the tooltip to the
status text. Drop the log-only thread.

## Acceptance criteria

- [ ] Tray icon colour changes: idle → recording → processing → idle
- [ ] Tray tooltip reflects current status
- [ ] Linux (GTK) path unchanged
- [ ] No cross-thread access to !Send tray handles

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Found-by: real use of v2.3.0 on Windows
- Parity: row 7 in CONSCIOUSNESS/artifacts/feature-parity-matrix.md
