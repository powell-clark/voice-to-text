# TASK-VTT114: macOS clipboard auto-paste sends Ctrl+V — must be Cmd+V

## Context

`src/typing.rs`'s clipboard-fallback paste path (`paste_text` /
`simulate_ctrl_v`) was `#[cfg(not(target_os = "windows"))]` — one code path
shared by Linux and macOS — and hardcoded `Key::Control` as the paste
modifier. Correct on Linux, wrong on macOS: the system paste shortcut there
is Cmd+V, and sending Ctrl+V does nothing useful (or triggers an unrelated
shortcut) instead of pasting the transcribed text.

## Fix

`src/typing.rs`: renamed `simulate_ctrl_v` to `simulate_paste_shortcut` and
introduced a per-platform `PASTE_MODIFIER` constant — `Key::Command` on
macOS (verified against `vendor/enigo/src/macos/macos_impl.rs:1012`, which
maps `Key::Command` to `KeyCode::COMMAND`), `Key::Control` everywhere else
this path compiles (Linux; Windows has its own separate `paste_text`).

## Acceptance criteria

- [x] macOS paste path sends Cmd+V, not Ctrl+V — code fix in place, Linux
      build/tests unaffected (verified: `cargo build --release`, 98/98 tests
      green)
- [ ] Verified pasting actually works on real macOS hardware — DEFERRED.
      Full cross-compile isn't possible from this Linux sandbox (`ring`'s C
      build requires an Apple clang toolchain, not available here); confidence
      rests on static inspection of the vendored `enigo` macOS backend rather
      than a live run. Needs confirmation on Emmanuel's Mac (FEAT-VTT029/
      DIRECT-VTT003 hardware).

## Dependencies

- Story: STORY-VTT012
- Directive: DIRECT-VTT003
- Feature: FEAT-VTT012 (typing/paste parity)
