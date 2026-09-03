# TASK-VTT157: Prepare 2.4.0 for release

## Context

Six operator-visible features sit under [Unreleased] while Cargo.toml and debian/changelog both read 2.3.11 — the last apt release, tagged 19 August. The unchanged version is also why apt refused two installs today and exited 0, which is what made a stale binary look like a successful install. Prepare only: bump Cargo.toml, Cargo.lock and debian/changelog, promote the CHANGELOG block, refresh the prebuilt so the .deb ships current code. Deliberately NOT tagging — pushing a v* tag fires release.yml and publishes to every PPA user, which is the operator's decision.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT018

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
