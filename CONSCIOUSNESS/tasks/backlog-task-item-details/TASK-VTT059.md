# TASK-VTT059: Local git pre-push hook matching CI

## Context
CI catches regressions on GitHub — but only after you push. A local pre-push hook catches the same regressions on YOUR machine before you send them anywhere. The friction is a few seconds of waiting at push time in exchange for not turning main red.

This hook runs the exact four checks from TASK-VTT058 locally. If all four pass, push proceeds. If any fail, push is aborted with a clear message.

## Acceptance Criteria
1. `hooks/pre-push` script exists in the repo (NOT `.git/hooks/pre-push` — that is gitignored; the actual hook is tracked in a repo directory and installed by a setup script)
2. `scripts/install-hooks.sh` installs `hooks/pre-push` by symlinking or copying into `.git/hooks/pre-push` (symlink for live updates preferred; confirm with Emmanuel before creating — his CLAUDE.md says "Don't create symlinks without asking")
3. The hook runs: `cargo fmt --all -- --check`, `cargo clippy --release --all-targets -- -D warnings`, `cargo test --release`, `cargo build --release`
4. Hook exits non-zero on any failure and prints which step failed and the command to reproduce
5. Hook can be bypassed with `git push --no-verify` for emergencies (this is standard and expected; global CLAUDE.md notes `--no-verify` should not be used by Claude)
6. `README.md` has a "Development setup" section with a single line: `bash scripts/install-hooks.sh`

## Technical Approach
Plain bash. No dependencies beyond cargo and the system deps that are already installed on this machine.

```bash
#!/usr/bin/env bash
# hooks/pre-push
set -e
echo "→ cargo fmt --check"
cargo fmt --all -- --check
echo "→ cargo clippy"
cargo clippy --release --all-targets -- -D warnings
echo "→ cargo test"
cargo test --release
echo "→ cargo build --release"
cargo build --release
echo "✓ all pre-push checks passed"
```

Ask Emmanuel before using symlinks (per global CLAUDE.md). Plain `cp` on install is safe and obvious; downside is the tracked copy can drift from the installed copy if someone edits in place. A `make install-hooks` target that re-copies on every invocation is a defensive middle ground.

## Test Strategy
1. Run `bash scripts/install-hooks.sh`, confirm `.git/hooks/pre-push` exists and is executable
2. `git push` on a clean working tree → hook runs, all pass, push succeeds
3. Introduce a formatting error, `git commit && git push` → hook fails at fmt step, push aborted
4. Fix format, push succeeds

## Files
- `hooks/pre-push` — new, tracked
- `scripts/install-hooks.sh` — new, tracked
- `README.md` — one line referencing the install command
