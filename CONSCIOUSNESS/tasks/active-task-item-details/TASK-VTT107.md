# TASK-VTT107: Smooth release CI/CD — never hang or fail a publish

## Context

Requested by Emmanuel (2026-06-26): "please get it working with smooth ci/cd".
v2.3.6 sat as a draft release for an hour because its macOS Intel (`macos-13`)
build job was stuck in GitHub's runner queue — and `publish-release` depended on
it, so the whole release never un-drafted. Earlier, v2.3.4 and v2.3.5 never
published at all because a Linux `-D warnings` failure killed the job that
*creates* the GitHub release.

Two distinct fragilities in `.github/workflows/release.yml`:
1. Lint strictness (`-D warnings`) on the release build — a warning fails a publish.
2. The scarce `macos-13` Intel runner is a hard dependency of publish.

## Approach

- Drop `RUSTFLAGS: -D warnings` from the release Linux build. Lint stays a PR
  gate in `ci.yml` (clippy + `-D warnings`); the release build is compile-only.
- Add `timeout-minutes` to every release job so a hung/queued runner fails fast.
- Split the macOS matrix: Apple-Silicon (`macos-latest`) is the REQUIRED asset;
  Intel (`macos-13`) becomes a best-effort job, removed from `publish-release`'s
  `needs`, attaching its binary via `gh release upload --clobber` (which never
  alters draft/latest state) so a late finish quietly adds the asset.
- `publish-release` waits only on Linux + Windows + arm64 macOS.

## Acceptance criteria

- [x] Release build no longer runs with `-D warnings` (PR CI still enforces it)
- [x] Every release job has a `timeout-minutes`
- [x] `publish-release` does not depend on the macOS Intel job
- [x] Intel binary attaches best-effort without re-drafting a published release
- [ ] A real tagged release (v2.3.7) publishes without manual intervention

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
- Feature: FEAT-VTT035
