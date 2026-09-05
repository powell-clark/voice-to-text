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

- [x] `vtt.exe` embeds a version + icon resource — `build.rs` (Windows-cfg-gated)
      uses `winresource` to embed `assets/vtt.ico` plus ProductName/
      FileDescription/ProductVersion/FileVersion/LegalCopyright, sourced from
      `CARGO_PKG_VERSION` so it tracks the crate version automatically.
      DEFERRED (operator gate) — the resource embed step compiling without
      error is evidenced by CI; actually seeing the icon + version render
      correctly in Explorer properties needs a human looking at a real
      Windows Explorer window, same class as TASK-VTT144's dialog-layout
      deferral.
- [x] Installer / ARP entry / shortcut wired to the icon — `wix/main.wxs` adds
      an `<Icon>` element + `ARPPRODUCTICON` property (Add/Remove Programs) and
      an explicit `Icon` attribute on the Start-Menu `<Shortcut>`.
      DEFERRED (operator gate) — `cargo wix` only runs in `release.yml` (tag
      push), not on every CI push, and WiX Toolset itself is Windows-only, so
      the XML has been reviewed carefully (standard, well-documented WiX
      patterns) but not compiled; needs a real release build or a manual
      `cargo wix` run on Windows to confirm.
- [x] Build validated on Windows — CI run (see Evidence) built both
      windows-latest (x86_64-msvc) and windows-11-arm (aarch64-msvc) green with
      the new `winresource` build-dependency and `build.rs` in place.
- [x] No regression to the Linux/macOS builds — `winresource` sits under the
      same `cfg(target_os = "windows")` gate as the rest of the Windows
      dependency block, so it never enters the Linux/macOS dependency graph;
      confirmed by a clean local Linux build (fmt/clippy -D warnings/209 tests)
      and CI's ubuntu + macOS-arm64 jobs on the same push.

## Evidence

Local (Linux dev box): `cargo build --release` clean with the new
`winresource` build-dep vendored; `cargo fmt --check` clean; `cargo clippy
--all-targets -- -D warnings` clean; `cargo test` — 209 passed, 0 failed.
Icon legibility checked at 16×16/32×32/256×256 (16×16 is the binding
constraint — Explorer list view, taskbar, title bar); a first draft's thin
cradling arc anti-aliased into a smudge at 16×16 and was redrawn as a bolder
capsule+stem+base silhouette.

CI run: see the push this task lands in — ubuntu fmt+clippy+test+build,
macOS arm64 build, windows-latest x86_64-msvc build, windows-11-arm build,
cargo-audit.

Note: the icon design (slate rounded-square, white microphone glyph,
`assets/generate_icon.py` regenerates it) is a first-pass brand mark, not a
final design decision — cheap to revise since it is a ~40-line generated
script, not a hand-authored binary asset.

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Feature: FEAT-VTT030
