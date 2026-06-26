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

- [ ] README documents how updates work on each platform
- [ ] App checks GitHub Releases for a newer version and surfaces it in the tray
- [ ] Windows in-place `.msi` upgrade verified (install vX over vX-1, settings/
      model cache preserved)
- [ ] No silent/forced auto-update — user stays in control

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Parity: row 15 in CONSCIOUSNESS/artifacts/feature-parity-matrix.md
