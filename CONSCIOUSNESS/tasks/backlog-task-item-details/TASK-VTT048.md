# TASK-VTT048: GitHub Actions matrix workflow — all platforms

## Acceptance Criteria
1. A single workflow file builds on `ubuntu-latest`, `macos-latest`, `macos-14` (ARM), and `windows-latest`
2. Each platform produces a release binary in the correct format (.bin, .app, .exe)
3. All four jobs run in parallel on every push to main and on every tag
4. The matrix is gated on the existing Linux CI checks (fmt, clippy, test) passing first


## Reality-check note (2026-07-17)

`.github/workflows/release.yml` already exists and is tag-triggered
(`on: push tags: v*`). It builds ubuntu-24.04 + windows-latest (.msi via
cargo-wix) + macos-latest (arm64) + macos-13 (Intel), drafts a GitHub
release, attaches each asset, and un-drafts when all succeed. The core of
this card is therefore ALREADY BUILT and live. Re-scope this card to the
actual remaining delta (verify the workflow runs green end-to-end; confirm
runner labels vs this card's original ubuntu-latest/macos-14 wording) or
close it against the live workflow — do NOT re-implement from scratch. See
ADR-0007 Context reconciliation note.
