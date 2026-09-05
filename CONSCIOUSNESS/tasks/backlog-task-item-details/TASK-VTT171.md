# TASK-VTT171: Package ct2-daemon into the Windows/macOS installs

## Context

Split from TASK-VTT167 (2026-09-05) — see that card for the Linux `.deb`
slice, already shipped. `resolve_daemon_script()` in `src/ct2_client.rs` now
has a `system_daemon_script()` fallback, but it is `#[cfg(target_os =
"linux")]`-gated and returns `None` everywhere else, with a comment pointing
here.

Windows: no WiX fragment in `wix/main.wxs` currently harvests an arbitrary
directory into the MSI — every `<File>` there is one explicit binary/icon,
not a `heat`-generated component group. Adding `ct2-daemon/` needs that
authoring plus a `#[cfg(target_os = "windows")]` counterpart to
`system_daemon_script()` pointing at wherever the MSI installs it (likely
next to the `.exe`, unlike Linux's FHS split, since Windows has no
`/usr/share` equivalent convention this repo already uses).

macOS: no `.app` bundling pipeline exists in this repo at all yet — blocked
on TASK-VTT040 (cargo bundle macOS `.app`). There is nowhere to place
`ct2-daemon/` inside a bundle that does not exist.

Neither installer can be built or its contents verified from this Linux
machine — real verification needs a human on the respective OS, same class
of gate as TASK-VTT108/TASK-VTT139/TASK-VTT144.

## Acceptance criteria

- [ ] _(to be scoped once TASK-VTT040 ships the macOS `.app` bundle — the
      macOS half of this card cannot be scoped against a bundle that does
      not exist yet)_

## Dependencies

- TASK-VTT040 (macOS `.app` bundle) — hard blocker for the macOS half only;
  the Windows half is independently scoped once someone picks this up.
- Directive: DIRECT-VTT002
- Story: STORY-VTT017
- Features: FEAT-VTT034
