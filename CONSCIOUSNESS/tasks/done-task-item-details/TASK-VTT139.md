# TASK-VTT139: Portable tray About dialog parity

## Context

ADR-0005 accepted alternative (b): mirror parity gaps onto portable.rs as small
independent tasks. This card originally bundled two gaps together — the About
window and the full Customize Transcription Settings dialog — but reading the
current `linux.rs` reference during implementation (2026-09-05) found the
settings dialog has grown far beyond what this card described: it is now 10+
sections (hotkey, language, model, device, voice prefix, initial prompt,
corrections, newline, archive, denoise), several of which have no muda/tray-icon
equivalent at all because that toolkit ships no dialog/text-input widgets.
Mirroring it blind would mean picking a GUI-toolkit strategy for Windows/macOS
before TASK-VTT138 (the spike whose entire purpose is to answer exactly that
question) has reported. Split: this card narrows to just the About window,
which needs no new toolkit (a native `MessageBoxW` on Windows); the settings
dialog moves to TASK-VTT170, explicitly blocked on TASK-VTT138.

Logs submenu parity is already shipped (TASK-VTT098). Windows is the
regression-testable target (operator dual-boots); macOS parked, same as the
rest of this story's portable-tray work.

## Acceptance criteria

- [x] `MenuCmd::About` opens a real, visible window on Windows (not just a log
      line) — `show_about_messagebox()` in `src/tray/portable.rs`, a native
      `MessageBoxW` via `windows-sys` (already a dependency; no new toolkit).
- [x] Content matches the Linux tray's `show_about_dialog` — app name, version
      (`CARGO_PKG_VERSION`), one-line description, GitHub URL, and the
      hold/release usage hint.
      NOTE: MessageBoxW text is selectable via right-click Copy / Ctrl+C, not
      a freely-selectable text widget — the closest native equivalent without
      adding a GUI toolkit dependency (that toolkit question is TASK-VTT138's).
- [x] macOS unaffected — still the existing log-line fallback (macOS parked,
      no dialog toolkit wired up; `#[cfg(target_os = "windows")]` gates the
      new code path).
- [ ] DEFERRED (operator gate) — actually seeing the About box render on a
      real Windows machine needs a human; CI validates it compiles and links
      on both windows-latest and windows-11-arm, not that the box looks right.

## Evidence

Reused the existing wide-string FFI pattern from `main.rs`'s
`singleton_lock` (`OsStr::encode_wide().chain(once(0))`), so the Win32 call
shape is already proven to compile in this codebase. `windows-sys`'s
`Win32_UI_WindowsAndMessaging` feature (already enabled in Cargo.toml) already
provides `MessageBoxW`/`MB_OK`/`MB_ICONINFORMATION` — no Cargo.toml change
needed. `cargo fmt` clean; this module only compiles on Windows/macOS targets,
so CI (windows-latest, windows-11-arm) is the actual compile verification —
see the push this task lands in.

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Features: FEAT-VTT030
