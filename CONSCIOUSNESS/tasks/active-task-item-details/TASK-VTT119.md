# TASK-VTT119: cargo audit red — RUSTSEC-2026-0190 anyhow unsound

## Context

Discovered 2026-07-01 checking CI on main after an unrelated push. cargo audit fails: anyhow 1.0.102 (direct dependency, Cargo.toml: anyhow = "1") hit RUSTSEC-2026-0190 'Unsoundness in Error::downcast_mut()', advisory published 2026-06-25. Unrelated to TASK-VTT106 (memmap2/enigo, already fixed and merged in 0830237) — that fix is confirmed working, this is a separate newly-published advisory. Fix is likely a simple cargo update -p anyhow to a patched version once one exists upstream; verify via cargo audit locally before pushing.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
