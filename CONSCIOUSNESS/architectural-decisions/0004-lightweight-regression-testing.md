# ADR-0004: Lightweight regression testing — pure helpers + GitHub Actions CI + pre-push hook

**Status:** Active
**Date:** 2026-04-20
**Context:** voice-to-text

## Context

Through v2.0.0 → v2.0.5 the project accumulated a regression pattern: every
release shipped a bug that only manifested when real users installed it from
the PPA. Examples: Cargo.lock v3 downgrade (v2.0.1), edition-2024 parse failure
on Noble cargo (v2.0.2), postinst hardcoded version (v2.0.4), silent non-ASCII
typing truncation in TUIs that don't bind Ctrl+V to paste (v2.0.5).

Until this ADR there was **zero automated testing**: no `tests/` directory, no
`#[test]` in `src/`, `.github/workflows/release.yml` was disabled, nothing ran
on push. The Launchpad PPA was effectively the first and only integration test.

Packaging regressions are now caught by the pbuilder hard gate (ADR-0003's
successor work in scripts/release-ppa.sh). Runtime regressions had nothing.

## Decision

A three-layer safety net — each cheap, each independent, each reinforcing:

### Layer 1: Cargo unit tests on pure helpers

Extract pure functions from modules that otherwise depend on GTK, enigo, or
cpal, and put unit tests inline via `#[cfg(test)] mod tests`:

- `settings::{strip_quotes, escape, unescape}` and `Settings::{load, save}`
  round-trip
- `main::compose_final_text(is_truncated, prefix, trimmed)`
- `tray::linux::format_log_label(filename, today, yesterday)`
- `audio::compute_append(current_len, incoming, max)`

Total: 28 tests at introduction. Execution time under 10 ms.

We deliberately do **not** test GTK widget behaviour, the cpal stream
callback itself, or whisper-rs inference. Those need a display, an audio
device, or a model file — all brittle in CI and low ROI relative to the
pure-logic coverage.

### Layer 2: GitHub Actions CI on every push

`.github/workflows/ci.yml` runs on every push to `main` and every PR
targeting `main`, on `ubuntu-24.04` (matching Launchpad Noble builders). Four
sequential steps:

1. `cargo fmt --all -- --check`
2. `cargo clippy --release --all-targets -- -D warnings`
3. `cargo test --release`
4. `cargo build --release`

Warnings-as-errors is the key discipline. No merging a branch that would fail
Launchpad's toolchain version.

### Layer 3: Local pre-push git hook

`scripts/git-hooks/pre-push` runs the exact same four checks before `git push`
hands the refs to the remote. Installed via `scripts/git-hooks/install.sh`.
This stops the CI feedback loop round-trip for the most common failure mode
(author forgot to format, or broke a test).

Bypassable via `git push --no-verify` for genuine emergencies only.

## Consequences

**Positive:**

- Three independent guards stop regressions from reaching users.
- CI badge on README.md gives contributors instant signal.
- Pure helpers (compose_final_text, format_log_label, compute_append) are
  reusable and the tests serve as executable documentation.
- Refactoring is safer — `cargo test` catches accidental semantic drift.

**Negative:**

- `#![allow(dead_code)]` at crate level (main.rs) permits legitimately
  retained API that isn't yet called (Settings::config_dir, logging::is_enabled,
  transcribe::load_wav). Future contributors may add genuine dead code and
  not notice. Mitigation: periodic audit, or pin the allow per-item if the
  crate-level becomes abused.
- CI runs cost GitHub Actions minutes. Current usage is negligible — one
  small repo, ~3 min per run, mostly free tier. If build time balloons
  (whisper-rs rebuild on every change), revisit with caching or split.
- Tests only cover the pure subset. Integration bugs (X11 Ctrl+V binding,
  GTK menu timing) still need manual reproduction. The v2.0.5 typing bug
  that motivated this ADR would not have been caught by unit tests; it
  required running the app in a real TUI.

## Alternatives Considered

1. **No tests, rely on release-manager manual QA.** Rejected — this is the
   status quo that produced the v2.0.x regression string.

2. **GTK widget tests via Xvfb + dogtail.** Rejected — brittle, CI-only,
   would catch maybe one bug class per year for weeks of setup work.

3. **Full integration tests with real model inference.** Rejected — requires
   committing or downloading a ~488 MB model in CI, slow (5+ min) and
   adds marginal confidence beyond what the unit tests plus daily use
   already provide.

4. **Pre-commit hook instead of pre-push.** Rejected — commits should be
   cheap so you can checkpoint mid-task. Push is the gate that matters.

5. **Require every PR to add a test.** Deferred — too rigid for a solo
   project. Revisit when team grows.

## References

- Commit `f08b06c` — initial 20 tests + 27→0 warning cleanup.
- Commit `bb88a66` — 8 more tests for audio truncation boundary.
- Commit `926a618` — GitHub Actions CI + pre-push hook.
- Commit `afebc00` — glslc dep for whisper-rs vulkan shader compilation.
- Story: STORY-VTT018 "Automated regression tests and release hygiene".
- Feature: FEAT-VTT035 "Automated regression testing".
- Tasks: TASK-VTT057, TASK-VTT058, TASK-VTT059.
