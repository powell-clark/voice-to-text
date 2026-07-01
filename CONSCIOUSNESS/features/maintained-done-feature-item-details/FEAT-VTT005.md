---
id: FEAT-VTT005
status: maintained
kano: must-have
---

# FEAT-VTT005: Text injection into focused application (Linux via XTest)

## Description
After transcription completes, the text is typed into the currently focused application using the X11 XTest extension (`enigo` crate on Linux). The text appears at the cursor position as if the user typed it. Clipboard is also set so the user can paste if the injection does not land correctly.

**Note:** macOS text injection via Accessibility API is in the codebase skeleton but the macOS binary is not yet distributed. The maintained behaviour covered by this feature is the Linux XTest path.

## Acceptance Criteria
- [x] **AC-1** — Transcribed text appears at the cursor position in the focused application without requiring a paste gesture — verified in daily use in Claude Code TUI, terminals, Firefox, Chrome, VS Code
- [x] **AC-2** — Unicode characters including £, é, ñ, and emoji type correctly — verified in v2.0.5 (£/é typing fix in TASK-VTT055)
- [x] **AC-3** — Clipboard is set to the transcription simultaneously so ctrl+v works as fallback — verified
- [x] **AC-4** — Text injection does not produce duplicate characters under normal use — verified in daily use
- [x] **AC-5** — Injection works in X11 and XWayland sessions — verified on Ubuntu 24.04 with GNOME/XWayland
- [x] **AC-6** — `enigo` is used for keyboard simulation, not raw XTest calls — verified in source

## Cross-platform acceptance criteria (DIRECT-VTT005 parity spec)
Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 3 — text injection).

**🐧 Linux — ✅ works**
- [x] Text types at the cursor via `xdotool type` (primary) with `enigo`/x11rb fallback — `src/typing.rs:41-152`
- [x] £/é/ñ/emoji type correctly; no duplicate characters in X11/XWayland

**🍎 macOS — 🟡 partial**
- [ ] Text types at the cursor via `enigo` once Accessibility permission is granted — code path exists (`src/typing.rs` non-Windows branch) but is untuned char-by-char and `xdotool` is always absent
- [ ] Accessibility-permission prompt presented to the user — BLOCKED on the `.app` bundle (FEAT-VTT029); a raw binary cannot register the prompt
- [ ] Verified typing into Safari, Notes, Terminal on a shipped build

**🪟 Windows — ✅ works**
- [x] Text types at the cursor via batched `enigo.text()` SendInput per newline segment — `src/typing.rs:69-105`
- [x] No dropped or reordered characters (regression fixed in TASK-VTT092)

## Linked Tasks
- TASK-VTT005, TASK-VTT018

## Parent Story
- STORY-VTT001
