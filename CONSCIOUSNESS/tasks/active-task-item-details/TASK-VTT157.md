# TASK-VTT157: Prepare 2.4.0 for release

## Context

Six operator-visible features sit under [Unreleased] while Cargo.toml and debian/changelog both read 2.3.11 — the last apt release, tagged 19 August. The unchanged version is also why apt refused two installs today and exited 0, which is what made a stale binary look like a successful install. Prepare only: bump Cargo.toml, Cargo.lock and debian/changelog, promote the CHANGELOG block, refresh the prebuilt so the .deb ships current code. Deliberately NOT tagging — pushing a v* tag fires release.yml and publishes to every PPA user, which is the operator's decision.

## Measured starting state, 2026-09-04

- `/usr/bin/vtt-linux` on the operator's machine: built 2026-08-18 10:13, `--version` says 2.3.11, `grep -c -a -- '--doctor'` returns 0. None of the six unreleased features are on the machine the operator dictates with.
- `git log -1 --format=%ci v2.3.11` → 2026-08-19. Cargo.toml, Cargo.lock, debian/changelog all still 2.3.11.
- `release-ppa.sh` refuses when Cargo.toml != debian/changelog, when the version is not newer than the last tag, or when the debian entry has zero bullets. `release-local.sh` refuses when the prebuilt's mtime is older than the newest commit touching src/, Cargo.toml or Cargo.lock.
- Operator approved the release 2026-09-04 01:48 ("All approved, go!"). Tag and dput still run through `release-ppa.sh`, which needs sudo (pbuilder) and the GPG passphrase — both interactive, so the operator runs that step.

## Acceptance criteria

- [ ] Cargo.toml, Cargo.lock and debian/changelog all read 2.4.0 — `release-ppa.sh` refuses on a mismatch
- [ ] CHANGELOG.md `[Unreleased]` promoted to `## [2.4.0] — 2026-09-04` with a fresh empty `[Unreleased]` heading above it; `scripts/gen-release-notes.sh 2.4.0 powell-clark/voice-to-text v2.4.0` extracts the section rather than the fallback stub
- [ ] debian/changelog 2.4.0 entry carries one bullet per user-facing change, matching the CHANGELOG block — `release-ppa.sh` refuses on zero bullets
- [ ] `packaging/linux/vtt-linux.prebuilt` rebuilt from the bumped source: `--version` prints 2.4.0, it contains `--doctor`, and its mtime is newer than the newest commit touching src/, Cargo.toml or Cargo.lock so the `release-local.sh` staleness gate passes
- [ ] `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` clean on the bumped tree
- [ ] Cargo.lock stays at lockfile version 3 after the build (Noble's cargo 1.75 cannot parse v4)
- [ ] `bash scripts/release-local.sh --install` builds and installs `voice-to-text_2.4.0_amd64.deb`; after `systemctl --user restart vtt.service`, `vtt-linux --doctor` reports 2.4.0 running — OPERATOR STEP, needs sudo
- [ ] No `v2.4.0` tag is created or pushed by this task — the tag and dput are `release-ppa.sh`, run by the operator

## Pre-mortem

### Failure modes

- The prebuilt ships old code again (TASK-VTT152, 2026-09-03: four green stages installed a two-week-old build). Mitigation: rebuild AFTER the bump commit and commit the prebuilt separately, so the staleness gate compares a fresh mtime against the bump commit and passes for the right reason; then prove it with `--version` and a `--doctor` string grep on the prebuilt itself.
- The staleness gate fails backwards: if the prebuilt is copied and then committed together with Cargo.toml, the commit time is newer than the file mtime and the gate refuses a binary that is in fact current. Mitigation: two commits — bump first, prebuilt second.
- Cargo.toml bumped but Cargo.lock not, so `cargo build --offline --locked` in `release-local.sh` fails. Mitigation: build once without `--locked` first, which rewrites the root package entry.
- The build rewrites Cargo.lock to v4 and Launchpad's Noble cargo cannot parse it. Mitigation: check `^version = 3$` after the build; the release scripts also downgrade defensively.
- 7 GB free of 31 GB at start; a full whisper.cpp rebuild could swap. Mitigation: target/release already holds the compiled whisper.cpp from 00:46 today, so only the app crate recompiles on the version change; check `free -g` before building.

### Weak assumptions

- That the other live session (vtt-cc-4ce2a46e, idle 16 h) has nothing uncommitted here — `git status` shows only session-stream files, no src/ or packaging/ changes.
- That the six CHANGELOG bullets are the complete user-facing delta since v2.3.11 — `git log v2.3.11..HEAD --oneline` is the check; anything there without a bullet gets one.
- That an empty `[Unreleased]` section breaks nothing — `gen-release-notes.sh` reads only the versioned section, and README does not parse the changelog.

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT018
