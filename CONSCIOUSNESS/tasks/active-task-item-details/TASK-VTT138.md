# TASK-VTT138: Spike hotkey dialog on portable tray

## Context

Required by ADR-0005 acceptance (alternative b now, a later gated on this spike). Time-boxed prototype of the ONE hardest linux.rs-only surface on tray-icon/muda: an interactive press-a-key-now hotkey capture dialog on Linux, using tao (already a transitive dep of tray-icon/muda) or a native dialog shim — muda has no dialog toolkit. Compare UX fidelity against linux.rs show_hotkey_dialog (X11 keycode capture via connect_key_press_event, 8-255 validation, HotkeyCmd::SetKeycode push). Outcome gates ADR-0005: acceptable equivalent -> revisit ADR-0005 for full unify migration plan; not acceptable -> (b) keep-split becomes the standing decision permanently. Do NOT retire any GTK code in this spike.

## Acceptance criteria

- [x] A `tao`-based prototype (`examples/hotkey_capture_spike.rs`, commit
      `327b756`) opened a window and captured a single "press-a-key-now"
      keyboard event, reporting the platform key code — `tao` was added as a
      `[dev-dependencies]` entry only, so the shipped Linux/Windows/macOS
      release binaries and their dependency graphs were unaffected throughout
      (build-isolation guarantee; confirmed via `cargo tree --edges
      normal,build` and a green `cargo build --release`).
- [x] Automated proof, not human judgement: `xdotool key <X>` synthesises a
      real keypress into the running example under this machine's X11
      session, and the captured code is confirmed against the expected key
      in real captured command output.
- [x] Written UX-fidelity comparison against `linux.rs`'s
      `show_hotkey_dialog` (GTK modal `connect_key_press_event` /
      `connect_key_release_event`, 8-255 X11 keycode validation,
      `HotkeyCmd::SetKeycode` push) covering: modal-vs-plain-window
      behaviour, keycode validation/range, and how directly the captured
      value maps onto `HotkeyCmd::SetKeycode`.
- [x] Verdict recorded on this card. This task does NOT itself flip
      ADR-0005's Status — that file is an active ADR and editing it requires
      operator approval per the safety precept; flip is a follow-up left to
      the operator once this card's evidence is reviewed.
- [x] No GTK code touched or retired; production `Cargo.toml`/binary
      dependency graph unchanged — verified via `cargo build --release`
      dependency diff.

## Findings (2026-09-05)

**Correction to this card's Context.** `tao` is NOT actually a pre-existing
transitive dependency of `tray-icon`/`muda` in this repo's `Cargo.lock` —
checked directly (`grep -n -A3 '^name = "tao"' Cargo.lock` returned nothing
before this spike). `tray-icon` 0.19.3 depends on `libappindicator` + `muda`
+ objc2/windows-sys; `muda` 0.15.3 depends on `gtk` directly for Linux menus,
not `tao`. So adding `tao` for this spike is a genuinely new dependency, not
a reuse of an existing one — added as `[dev-dependencies]` only, consumed
solely by `examples/hotkey_capture_spike.rs`, confirmed absent from
`cargo tree --edges normal,build` and from `cargo build --release`'s compile
list.

**Automated capture proof.** `examples/hotkey_capture_spike.rs` opens a
`tao` window and matches `WindowEvent::KeyboardInput`'s
`key_event.physical_key: tao::keyboard::KeyCode`. On this Linux/X11 session
(`DISPLAY=:1`), a named key (e.g. `KeyCode::KeyB`) round-trips through
`KeyCode::to_scancode()` to the exact X11 keycode. Deliberate, reproducible
proof via `xdotool key --clearmodifiers b` (no human involved):
```
SPIKE RESULT: matched a named KeyCode::KeyB — to_scancode() = Some(56)
SPIKE RESULT: captured raw keycode 56 (valid 8-255 range, ready for HotkeyCmd::SetKeycode)
```
X11 keycode 56 = evdev `KEY_B` (48) + 8 — correct. An earlier, undirected run
also captured an incidental real keypress (`KeyT` → scancode 28 = evdev
`KEY_T` (20) + 8) confirming the mechanism works on ordinary ambient input
too, not just synthetic events.

**UX-fidelity comparison vs. `linux.rs::show_hotkey_dialog`:**

| Aspect | `linux.rs` (GTK) | `tao` prototype |
|---|---|---|
| Modality | Modal `gtk::Dialog`, blocks the rest of the tray until closed/cancelled | Plain top-level window; no modal/parent relationship to the tray icon without extra plumbing (`tao` has no dialog widget, only windows) |
| Capture mechanism | `connect_key_press_event`/`connect_key_release_event` signal handlers on the dialog | `WindowEvent::KeyboardInput` matched in the app event loop — same event-driven shape, different framework |
| Raw code exposed | X11 keycode directly (event.hardware_keycode) | `KeyCode::Unidentified(NativeKeyCode::Gtk(u16))` for unnamed keys, or `KeyCode::<Named>.to_scancode()` for named ones — both land on the same X11 keycode space, confirmed above |
| Validation | 8-255 range check inline in the dialog | Identical 8-255 range check ported directly into the spike (`validate_x11_keycode`) — no logic change needed |
| Push to hotkey thread | `HotkeyCmd::SetKeycode` sent directly from the dialog's handler | Same shape: the captured, validated `u8` is immediately ready for `HotkeyCmd::SetKeycode` — no adapter needed |
| Linux windowing backend | GTK3 directly | **`tao`'s own Linux backend is ALSO GTK-based** (`NativeKeyCode::Gtk`, and `gdkx11-sys`/`gdkwayland-sys` pulled into the dev-dependency build) — using `tao` on Linux does not remove the GTK dependency, only moves which crate owns the GTK call |
| Cancel/escape | GTK dialog has a Cancel button + window-close = abort | Prototype does not implement Escape-to-cancel (out of scope for this time-boxed spike, but trivial to add — matching a `KeyCode::Escape` arm before the general case) |

**Verdict: acceptable equivalent for keycode capture, but with a load-bearing
caveat for the wider unify decision.** The event-capture mechanism itself
maps cleanly (same 8-255 X11 keycode space, same validation, same
`HotkeyCmd::SetKeycode` handoff) — alternative (a)'s stated risk ("no dialog
toolkit … needs prototyping before committing") is resolved for keycode
capture specifically. However this spike also produces new evidence ADR-0005
did not have: **`tao`'s Linux backend still links GTK** (`gdkx11-sys`,
`gdkwayland-sys` in this build), so unifying onto `tao`/`tray-icon`/`muda`
does not eliminate the `libappindicator`/`gtk` build dependency ADR-0005's
alternative (a) lists as a pro ("Removes the `libappindicator`/`gtk`/`glib`
build dependency from the Linux target entirely") — that pro does not hold.
Recorded here as evidence for the operator to weigh when ADR-0005 is
revisited; this task does not flip the ADR's Status itself.

**Cleanup: the prototype was removed after its evidence was captured above.**
`cargo audit` on commit `327b756` came back red: `tao` pulls in
`gdkwayland-sys`/`gdkx11-sys` (RUSTSEC-2024-0411, -0414, both unmaintained
gtk-rs GTK3 bindings) and `instant` (RUSTSEC-2024-0384, unmaintained) — none
of which are in ci.yml's existing GTK-advisory ignore list, and CI's
`--all-targets` step compiles every example on every platform (ubuntu,
windows-latest, windows-11-arm, macos-latest), so the new dependency was not
actually as contained as the "dev-dependency only" framing above implied —
it became a standing per-push CI cost on 4 platforms plus a permanent
security-gate widening, for a question this spike had already answered.
Rather than expand the audit ignore-list to accommodate a one-shot spike
(engineering-first-principles: delete first, don't let a temporary
prototype become a permanent maintenance line), `examples/hotkey_capture_spike.rs`
and the `tao` dev-dependency were removed in commit `1cb2d03` — confirmed via `grep` that `tao`,
`gdkx11-sys`, `gdkwayland-sys`, and `instant` are all absent from `Cargo.lock`
again. The evidence above (exact captured output, the scancode/keycode
match, the UX comparison table, the GTK-still-linked finding) is the
permanent record; the code was the means, not the deliverable. The full
prototype source remains recoverable from commit `327b756` if a future task
resumes this work.

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT002
- Features: FEAT-VTT030
