# TASK-VTT131: Clipboard persists without a clipboard manager (FEAT-VTT038 X11 regression)

## Context
Emmanuel stopped running CopyQ (it was "breaking parts of the system") and the
tray "Copy last transcription" item (FEAT-VTT038) silently stopped working — he
now recovers text by opening the log by hand, which is the exact pain the
feature was built to remove (2026-07-17, session vtt-main-690a6246).

Root cause (confirmed by code read): `typing::set_clipboard_text` uses
`arboard::Clipboard::new()` → `set_text()`, then the `Clipboard` is dropped at
function return. On X11/Wayland the clipboard *selection* is owned by the live
client, so on drop the text is released — it survived only because CopyQ (a
clipboard manager) was grabbing the selection. Without a manager, the copy is a
silent no-op. The existing `paste_via_xclip` holder path (xclip forks a daemon
that serves the selection until the next paste) is only reached on arboard
*error*, never on the success-then-drop path.

## Acceptance Criteria
1. On Linux, `set_clipboard_text` routes through a holder tool (xclip / xsel,
   plus wl-copy when `WAYLAND_DISPLAY` is set) that keeps the selection alive
   after the function returns — the copied text is pasteable with no clipboard
   manager running.
2. macOS / Windows keep using arboard (the OS clipboard persists natively there)
   — no behaviour change off Linux.
3. Actual proof on X11 without CopyQ: copy text, read it back via `xclip -o`
   after the process that set it has returned, and confirm it matches.
4. cargo fmt / clippy -D warnings / cargo test green.

## Technical Approach
- Rewrite `set_clipboard_text` so the Linux branch calls the holder path
  directly (not only on arboard error); keep the arboard branch for
  non-Linux.
- Extend the holder helper to try `wl-copy` first on Wayland, then xclip, then
  xsel — each forks a daemon that owns the selection.
- Regression-guard: a unit test that the Wayland/X11 tool-ordering selector
  picks the right first candidate per session type (pure function, no GUI).

## Dependencies
- FEAT-VTT038 (Copy last transcription) — this repairs its silent breakage.
