# TASK-VTT090: Release pipeline — CHANGELOG-driven notes, macOS binary, download links

## Context

Requested by Emmanuel (2026-06-26): the GitHub release log was "awful" — every
release showed the same hardcoded boilerplate body (Ubuntu + Windows only), never
reflected what actually changed, omitted macOS entirely, and buried the installer
assets. He wants a release manager modelled on the consciousness repo.

## Changes

- `scripts/gen-release-notes.sh` — extracts the version's section from
  CHANGELOG.md and appends a downloads table with direct asset links for all
  three platforms. Tested locally; CI feeds its output to the release via
  `body_path`.
- `release.yml` — replaced the static `body:` with the generated notes; added a
  `build-macos` job that compiles and attaches `vtt-macos` (Apple Silicon) to
  every release.

## Acceptance criteria

- [x] Release notes are generated per-version from CHANGELOG.md (no boilerplate)
- [x] Notes include a downloads table with direct links for Windows/Linux/macOS
- [x] A macOS binary (`vtt-macos`) is built and attached to releases
- [x] Notes generator is a tested script, not inline YAML
- [ ] Verified end-to-end on the next tagged release (v2.3.1)

## Follow-ups (not in scope here)

- Signed macOS `.app` bundle — TASK-VTT040
- Report-by-default release-manager agent with explicit-confirmation gate and a
  releases.jsonl audit log, per the consciousness pattern

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
