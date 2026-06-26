# TASK-VTT108: Branded Windows app + installer icon and version info

## Context

The shipped `vtt.exe` carries **zero embedded resources** — version info reads
empty in Explorer, and there is no application icon, so the .exe, Alt-Tab,
installer, and Start-Menu shortcut are anonymous. (The tray notification-area
icon is separate and works; it is a runtime-generated coloured circle.)

Separately, on Windows 11 the tray icon is registered correctly but hidden in
the notification-area overflow (`^`) flyout by default — that is OS behaviour,
not a bug, and cannot be forced open programmatically. A recognisable branded
icon makes it easy to identify once surfaced.

## Approach

- Add `assets/vtt.ico` (multi-resolution: 16/32/48/256).
- Add a `build.rs` using `winresource` (Windows-only, cfg-gated) to embed the
  icon + version info (from `CARGO_PKG_VERSION`) into `vtt.exe`.
- Wire the icon into the WiX installer (`wix/main.wxs`) for the ARP entry and
  any Start-Menu shortcut.
- Consider loading the same `.ico` for the tray base icon while keeping the
  green/red/amber state tinting.

## Acceptance criteria

- [ ] `vtt.exe` shows version + icon in Explorer properties
- [ ] Installer / ARP entry / shortcut show the branded icon
- [ ] Build validated locally on Windows (release build is green)
- [ ] No regression to the Linux/macOS builds (build.rs is Windows-gated)

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Feature: FEAT-VTT030
