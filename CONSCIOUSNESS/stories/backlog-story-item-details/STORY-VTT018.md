# STORY-VTT018: Automated regression testing and release hygiene

## User Story
As Emmanuel I want automated regression tests and CI gates so that broken typing, broken tray menus, and broken releases stop reaching users.

## Why This Matters
Recent regression pattern (git log): v2.0.0 → v2.0.1 Cargo.lock v3 downgrade; v2.0.2 switched to prebuilt binary to bypass Noble cargo 1.75; v2.0.3 jammy alternation fix; v2.0.4 postinst version hardcoding; v2.0.4 pbuilder gate wasn't actually gating. Then v2.0.5 (being released now) fixes two runtime bugs that went undetected — typing stops at £/é because Ctrl+V paste is silently dropped in TUIs, and the tray Logs submenu rebuilds too late so first open is always stale.

Packaging regressions are now caught by the pbuilder hard gate (commit b10cee3). Runtime regressions have nothing catching them — zero `#[test]` in `src/`, no `tests/` directory, no CI that runs on push.

The lack is horrendous. This story closes that gap without overengineering — unit tests on the pure logic we can actually unit-test (pure functions, no GTK/enigo/X11), CI that runs the same on every push, and a local pre-push hook so you feel the friction before GitHub does.

## Acceptance Criteria
1. `cargo test` runs locally and passes on a clean clone
2. At least one test per pure-logic area: settings round-trip, log filename label, audio buffer truncation, transcription prefix logic
3. `.github/workflows/ci.yml` exists and runs fmt, clippy, test, and release build on ubuntu-24.04 on every push to main and every PR
4. A failing commit to main triggers a red check and a notification email from GitHub
5. `.git/hooks/pre-push` (or documented setup script that installs it) runs the same four checks locally and blocks the push on failure
6. Old Launchpad PPA versions (below v2.0.3) are deleted; PPA quota drops meaningfully
7. v2.0.5 is released via release-manager with pbuilder gate passing
8. Three days after release, typing monitoring confirms no £/é regressions

## Scope
- **In scope:** cargo unit tests on pure logic, GitHub Actions CI, local pre-push hook, PPA cleanup, v2.0.5 release, short monitoring window
- **Out of scope:** GTK widget tests (brittle, low ROI), full transcription integration tests (need model fixture, slow), cross-platform CI matrix (TASK-VTT048 tracks that separately), snapshot testing (premature)

## Tasks
- TASK-VTT055 — Release v2.0.5 (the two fixes)
- TASK-VTT056 — Monitor v2.0.5 in daily use
- TASK-VTT057 — Unit tests on pure logic
- TASK-VTT058 — GitHub Actions CI
- TASK-VTT059 — Local pre-push hook
- TASK-VTT060 — Launchpad PPA cleanup
- TASK-VTT061 — Local build-archives cleanup

## Definition of Done
Green CI badge on the README. Red CI blocks merges. Two consecutive releases ship without a post-release hotfix. v2.0.5 confirmed typing £/é end-to-end in three different target apps.
