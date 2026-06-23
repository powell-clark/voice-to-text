---
id: FEAT-VTT035
status: maintained
kano: must-have
---

# FEAT-VTT035: Automated regression testing

## Description
A lightweight regression safety net: cargo unit tests on pure logic, GitHub Actions CI on every push, and a local pre-push git hook matching CI. Together they make main green-by-default and give Emmanuel real signal before shipping to PPA.

## Why (Kano: must-have)
Voice-to-text ships weekly. Every release of v2.0.x has had a regression that reached the PPA because there is no pre-ship check. The pbuilder hard gate now catches packaging regressions. Unit tests + CI catch the code-level regressions that pbuilder cannot see.

This is a must-have, not a delighter — it underpins the release-cadence claim made in the vision statement ("outperforms paid alternatives"). You cannot outperform anyone if you regress every Friday.

## Scope
**In:**
- `cargo test` on pure logic (settings, log labels, audio bounds, text composition)
- GitHub Actions CI on `ubuntu-24.04` running fmt + clippy + test + build on every push
- Local pre-push hook matching CI
- README badge advertising the green state

**Out:**
- GTK widget tests (require Xvfb, brittle)
- Full transcription integration tests (need model, slow)
- Cross-platform CI matrix (TASK-VTT048 covers macOS + Windows separately)
- Snapshot testing (premature abstraction)

## Linked Story
STORY-VTT018 — Automated regression tests and release hygiene.

## Linked Tasks
- TASK-VTT055 release v2.0.5 (the typing + logs fixes that motivated this)
- TASK-VTT056 monitor in production
- TASK-VTT057 unit tests
- TASK-VTT058 GitHub Actions CI
- TASK-VTT059 local pre-push hook

## Measurable Outcome
Before: 0 tests, 0 CI runs per push, regression caught by Emmanuel at the PPA stage.
After: ~8 tests across 4 modules, CI runs on every push, pre-push hook catches failures before the push leaves the machine. Target: two consecutive releases without a post-release hotfix commit.

## Acceptance Criteria
- [x] `cargo test` passes on main with at least 20 tests covering settings, log labels, audio bounds, and text composition — verified: 67 tests as of v2.1.1 (expanded from original 20)
- [x] `.github/workflows/ci.yml` exists and runs fmt + clippy + test + build on every push to main — verified in repo
- [x] `scripts/git-hooks/pre-push` exists and mirrors CI checks locally — verified in repo, installed via `scripts/install-dev.sh`
- [x] README CI badge links to the GitHub Actions run and shows green — verify in README.md
- [x] Two consecutive releases (v2.0.5 → v2.1.0 → v2.1.1) shipped without a post-release hotfix commit needed for a regression — verified in git log
