# TASK-VTT058: GitHub Actions CI — fmt, clippy, test, build on push

## Context
`.github/workflows/release.yml` exists but is disabled (commented-out triggers) and only runs on tag push for macOS release artefacts. There is no CI that validates main on every push. The Launchpad PPA is effectively our first and only integration test — which is why every packaging regression (Cargo.lock v3, edition 2024, postinst) has been discovered by real users first.

This task adds a lightweight CI that runs on `ubuntu-24.04` (matches Launchpad Noble builders) on every push to main and every PR, so broken code never gets to PPA.

## Acceptance Criteria
1. `.github/workflows/ci.yml` exists and is triggered by `push` to main, `pull_request` targeting main, and `workflow_dispatch`
2. Four jobs (or four steps in one job, preferred) on `ubuntu-24.04`:
   - `cargo fmt --all -- --check` — zero drift allowed
   - `cargo clippy --release --all-targets -- -D warnings` — warnings are errors in CI (existing 27 warnings need fixing or allow-listing first — see note below)
   - `cargo test --release` — all tests from TASK-VTT057 pass
   - `cargo build --release` — binary compiles for Noble's toolchain
3. System deps installed via `apt-get install` before cargo: `libgtk-3-dev libayatana-appindicator3-dev libasound2-dev libvulkan-dev libxtst-dev` (match the Launchpad Build-Depends)
4. Rust toolchain pinned via `rust-toolchain.toml` or `dtolnay/rust-toolchain@stable` — whatever matches what the prebuilt binary uses
5. Cargo cache configured via `Swatinem/rust-cache@v2` so repeat runs take under 3 minutes
6. CI fails on any of the four steps; CI status badge added to `README.md`

## Technical Approach
Single workflow file, single job, four sequential steps. Example skeleton:
```yaml
name: CI
on:
  push: { branches: [main] }
  pull_request: { branches: [main] }
  workflow_dispatch:
jobs:
  check:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt,clippy }
      - uses: Swatinem/rust-cache@v2
      - run: sudo apt-get update && sudo apt-get install -y libgtk-3-dev libayatana-appindicator3-dev libasound2-dev libvulkan-dev libxtst-dev
      - run: cargo fmt --all -- --check
      - run: cargo clippy --release --all-targets -- -D warnings
      - run: cargo test --release
      - run: cargo build --release
```

**Existing warning debt:** the release build emits 27 warnings (mostly X11 constant naming `XK_End` etc.). `cargo clippy -D warnings` will fail until those are either fixed or allow-listed with `#[allow(non_upper_case_globals)]` at the pattern site. Fix them in the same PR or precede this task with a cleanup PR.

## Test Strategy
1. Merge the workflow on a branch, open a PR, watch the check go green
2. Break formatting on a second commit, watch the check go red
3. Revert the break, confirm green again

## Files
- `.github/workflows/ci.yml` — new
- `README.md` — add CI status badge
- Possibly `rust-toolchain.toml` — pin the stable version
- Warning cleanup across `src/hotkey/linux.rs` and any other `XK_*` pattern sites
