# TASK-VTT106: cargo audit red — RUSTSEC-2026-0186 (memmap2 unsound)

CI's `cargo audit` job failed repo-wide (including on `main`) on **RUSTSEC-2026-0186
— "Unchecked pointer offset in crate `memmap2`"** (class: `unsound`), fixed in
memmap2 **>= 0.9.11**. The job runs `--deny warnings`, so the unsound warning
denies the build.

## Root cause (not fixable by `cargo update` alone)

`memmap2 0.8.0` came in transitively, Linux-only:

```
voice-to-text → enigo 0.2.1 → xkbcommon 0.7.0 → memmap2 0.8.0
```

`xkbcommon 0.7.0` hard-pins `memmap2 = "^0.8.0"` and `enigo 0.2.1` caps
`xkbcommon` at `^0.7`, so `cargo update -p memmap2 --precise 0.9.11` is rejected.
The fix must come from bumping our direct dep `enigo`.

## Fix applied — upgrade enigo 0.2 → 0.6 (all three target blocks)

enigo 0.6.1 → xkbcommon 0.9.0 → **memmap2 0.9.11** (patched). The `x11rb`
feature still exists in 0.6 (resolution confirmed).

All three `enigo` declarations were bumped, not just the Linux one: the Cargo.lock
is **target-agnostic**, so leaving Windows/macOS on `enigo 0.2.1` would keep its
`xkbcommon 0.7.0 → memmap2 0.8.0` subtree in the lock and `cargo audit` (which
reads the lock, not the per-target build) would still fail. Bumping all three
evicts the 0.2.1 subtree entirely — lock now carries a single `enigo 0.6.1`,
single `memmap2 0.9.11`, single `xkbcommon 0.9.0`.

All 37 enigo call sites live in `src/typing.rs` and use the stable
`Enigo`/`Settings`/`Key`/`Direction`/`Keyboard` API (`.key()`, `.text()`),
unchanged across 0.2 → 0.6.

- [x] `memmap2` 0.8.0 evicted from `Cargo.lock`; only patched 0.9.11 remains — confirmed in commit `0830237`
- [x] CI matrix compiles with enigo 0.6 on Linux, Windows, macOS — confirmed green on run 28492074555 (2026-07-01)
- [ ] [deferred → daily use / next dedicated smoke-test task] Runtime text-injection smoke test on Windows + macOS
      via `scripts/smoke-test-windows.ps1` — not run this pass; build-green is not proof of runtime typing behaviour

## Closing note (2026-07-01)
`cargo audit` on `main` is still red, but not from this task's target vulnerability —
RUSTSEC-2026-0186 (memmap2) is gone. A newly-published, unrelated advisory
(RUSTSEC-2026-0190, `anyhow` unsoundness) started failing the same job the same day;
split off as TASK-VTT119 rather than block this task's closure on an unrelated bug.
Closed on operator instruction with the runtime smoke-test AC explicitly deferred,
not silently dropped.

Surfaced during TASK-VTT105 (Claude workflow parity) PR review.
