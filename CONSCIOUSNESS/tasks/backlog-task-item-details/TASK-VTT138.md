# TASK-VTT138: Spike hotkey dialog on portable tray

## Context

Required by ADR-0005 acceptance (alternative b now, a later gated on this spike). Time-boxed prototype of the ONE hardest linux.rs-only surface on tray-icon/muda: an interactive press-a-key-now hotkey capture dialog on Linux, using tao (already a transitive dep of tray-icon/muda) or a native dialog shim — muda has no dialog toolkit. Compare UX fidelity against linux.rs show_hotkey_dialog (X11 keycode capture via connect_key_press_event, 8-255 validation, HotkeyCmd::SetKeycode push). Outcome gates ADR-0005: acceptable equivalent -> revisit ADR-0005 for full unify migration plan; not acceptable -> (b) keep-split becomes the standing decision permanently. Do NOT retire any GTK code in this spike.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT002
- Features: FEAT-VTT030
