# TASK-VTT152: Fail the build when the packaged binary is stale

## Context

release-local.sh ran four green stages and shipped a binary two weeks older than HEAD, after a sudo prompt, with no warning. debian/rules deliberately does not compile — it installs packaging/linux/vtt-linux.prebuilt, because Launchpad builds on Noble whose Cargo 1.75 cannot parse this crate's edition-2024 manifest. That constraint is real. The gap is that the pre-flight cargo build --offline --locked proves the source compiles and proves nothing about what gets packaged. Add a gate: fail when the prebuilt's mtime predates the newest commit touching src/, naming both timestamps and the refresh command.

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
