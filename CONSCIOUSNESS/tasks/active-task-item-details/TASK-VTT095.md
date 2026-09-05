# TASK-VTT095: Update mechanism — how Windows/macOS users get new versions

## Context

Asked by Emmanuel (2026-06-26): "how does auto-updating happen — do I download it
every time, install over the top, or uninstall first?" Needs a clear update story
per platform, and ideally an in-app update check.

## Current state

- **Windows (.msi)**: the WiX template uses `MajorUpgrade`, so running a newer
  `.msi` upgrades in place — no manual uninstall needed. But there is no in-app
  update *check*; the user must notice and download manually.
- **Linux**: `apt upgrade` via the PPA handles it (already automatic).
- **macOS**: no packaging yet (TASK-VTT040), so no update path.

## Approach (proposed)

1. **Document** the current per-platform update story in the README (immediate).
2. **In-app update check**: on launch (and/or daily), query the GitHub Releases
   API for the latest tag; if newer than `CARGO_PKG_VERSION`, surface a tray item
   "Update available → vX.Y.Z" linking to the release. Non-intrusive, no silent
   self-update.
3. Optional later: one-click download + run installer on Windows.

## Acceptance criteria

- [x] README documents how updates work on each platform
- [x] App checks GitHub Releases for a newer version and surfaces it in the tray
- [ ] Windows in-place `.msi` upgrade verified (install vX over vX-1, settings/
      model cache preserved) — DEFERRED: needs a real Windows machine, which
      this Linux dev environment cannot provide. Filed as TASK-VTT168 (Verify
      Windows .msi in-place upgrade).
- [x] No silent/forced auto-update — user stays in control

## Evidence

- README.md: new `## Updates` section, per-platform (`sudo apt upgrade`,
  re-run the `.msi`, replace the macOS binary) plus the in-app check behaviour.
- `src/update_check.rs`: `check_for_update()` queries
  `api.github.com/repos/powell-clark/voice-to-text/releases/latest`, compares
  against `CARGO_PKG_VERSION` via `parse_version`/`is_newer` (never guesses on
  a malformed tag), returns `None` on any failure — informational only, no
  download/install. 8 unit tests, all passing (`cargo test --release`: 206
  passed, 0 failed, 3 ignored).
- Live-verified against the real API (not mocked): `curl
  api.github.com/repos/powell-clark/voice-to-text/releases/latest` returned
  `tag_name: v2.4.0`, `html_url: .../releases/tag/v2.4.0` — confirms the JSON
  shape `check_for_update()` parses matches production, and correctly reports
  "no update" since v2.4.0 is also `CARGO_PKG_VERSION`.
- Wired at startup on its own thread (`transcription_worker`'s sibling
  `update-check` thread in `main.rs`) so a slow/offline network never delays
  readiness; on `Some(UpdateInfo)` sends `UiMessage::UpdateAvailable(version,
  url)`.
- Linux tray (`src/tray/linux.rs`, verified — this machine builds it): a
  hidden `update_item` (GTK `set_no_show_all` + `hide()`) is shown and
  relabelled "Update available: vX.Y.Z" on the message; clicking opens the
  release URL via the existing `xdg-open` helper.
- Portable tray (`src/tray/portable.rs`, macOS/Windows, NOT independently
  verified — cannot build for either target here): added by analogy to the
  TASK-VTT054 `backend` item pattern already used in this file. muda's
  `MenuItem` has no visibility toggle, so it starts disabled ("Up to date")
  and is relabelled + enabled on the message; click opens the URL via `open`
  (macOS) / `cmd /C start` (Windows).
- Full gate: `cargo build --release`, `cargo fmt --check`, `cargo clippy
  --all-targets -- -D warnings`, `cargo test --release` — all clean.

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Parity: row 15 in CONSCIOUSNESS/artifacts/feature-parity-matrix.md
- Follow-up: TASK-VTT168 (Verify Windows .msi in-place upgrade)
