# TASK-VTT138: Spike hotkey dialog on portable tray

## Context

Required by ADR-0005 acceptance (alternative b now, a later gated on this spike). Time-boxed prototype of the ONE hardest linux.rs-only surface on tray-icon/muda: an interactive press-a-key-now hotkey capture dialog on Linux, using tao (already a transitive dep of tray-icon/muda) or a native dialog shim — muda has no dialog toolkit. Compare UX fidelity against linux.rs show_hotkey_dialog (X11 keycode capture via connect_key_press_event, 8-255 validation, HotkeyCmd::SetKeycode push). Outcome gates ADR-0005: acceptable equivalent -> revisit ADR-0005 for full unify migration plan; not acceptable -> (b) keep-split becomes the standing decision permanently. Do NOT retire any GTK code in this spike.

## Acceptance criteria

- [ ] A `tao`-based prototype (`examples/hotkey_capture_spike.rs`) opens a
      window and captures a single "press-a-key-now" keyboard event,
      reporting the platform key code — `tao` added as a `[dev-dependencies]`
      entry only, so the shipped Linux/Windows/macOS release binaries and
      their dependency graphs are unaffected (build-isolation guarantee).
- [ ] Automated proof, not human judgement: `xdotool key <X>` synthesises a
      real keypress into the running example under this machine's X11
      session, and the captured code is confirmed against the expected key
      in real captured command output.
- [ ] Written UX-fidelity comparison against `linux.rs`'s
      `show_hotkey_dialog` (GTK modal `connect_key_press_event` /
      `connect_key_release_event`, 8-255 X11 keycode validation,
      `HotkeyCmd::SetKeycode` push) covering: modal-vs-plain-window
      behaviour, keycode validation/range, and how directly the captured
      value maps onto `HotkeyCmd::SetKeycode`.
- [ ] Verdict recorded on this card. This task does NOT itself flip
      ADR-0005's Status — that file is an active ADR and editing it requires
      operator approval per the safety precept; flip is a follow-up left to
      the operator once this card's evidence is reviewed.
- [ ] No GTK code touched or retired; production `Cargo.toml`/binary
      dependency graph unchanged — verified via `cargo build --release`
      dependency diff.

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT002
- Features: FEAT-VTT030
