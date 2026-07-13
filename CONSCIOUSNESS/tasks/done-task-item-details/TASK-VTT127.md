# TASK-VTT127: CI contract gate for regression and release workflow parity

## Context

The repository already runs four Cargo checks in GitHub Actions and the local
pre-push hook, but the configuration that defines those gates was not itself
tested. A future edit could silently remove a check, change the CI trigger, or
make the release workflow wait on the scarce macOS Intel runner again. Those
drifts recreate the broken typing, tray, and release regressions covered by
STORY-VTT018.

## Approach

- Add `scripts/ci/check-regression-gates.sh` as a dependency-free contract
  check for the CI workflow, pre-push hook, and release workflow.
- Require the same four Cargo commands in CI and pre-push: fmt, clippy, test,
  and release build.
- Require push/PR coverage for `main`, timeout coverage for every release job,
  and a publish job that does not depend on the best-effort Intel build.
- Run the contract check in both GitHub Actions and the local pre-push hook.

## Acceptance criteria

- [x] The contract checker passes against the current repository configuration.
- [x] The checker detects missing Cargo gates or CI push/PR triggers.
- [x] The checker detects missing release job timeouts and an Intel dependency
      in `publish-release`.
- [x] GitHub Actions and the local pre-push hook run the contract checker.
- [x] Existing Cargo formatting, lint, tests, and release build remain green.

## Verification

- `bash scripts/ci/check-regression-gates.sh`
- `cargo fmt --all -- --check`
- `cargo clippy --release --all-targets -- -D warnings`
- `cargo test --release`
- `cargo build --release`

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
- Feature: FEAT-VTT035
