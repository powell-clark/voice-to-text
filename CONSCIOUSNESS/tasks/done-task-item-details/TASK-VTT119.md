# TASK-VTT119: cargo audit red — RUSTSEC-2026-0190 anyhow unsound

## Context

Discovered 2026-07-01 checking CI on main after an unrelated push. cargo audit fails: anyhow 1.0.102 (direct dependency, Cargo.toml: anyhow = "1") hit RUSTSEC-2026-0190 'Unsoundness in Error::downcast_mut()', advisory published 2026-06-25. Unrelated to TASK-VTT106 (memmap2/enigo, already fixed and merged in 0830237) — that fix is confirmed working, this is a separate newly-published advisory. Fix is likely a simple cargo update -p anyhow to a patched version once one exists upstream; verify via cargo audit locally before pushing.

## Fix applied — `cargo update -p anyhow --precise 1.0.103`

The patch landed upstream the same day the advisory published: anyhow 1.0.103
(2026-06-25) fixes the `Error::downcast_mut()` unsoundness (advisory
`versions.patched: >=1.0.103`). `Cargo.toml` already declares `anyhow = "1"`,
so no manifest change was needed — only `Cargo.lock`.

The repo vendors dependencies (`.cargo/config.toml` replaces `crates-io` with
a local `vendor/` directory per `cargo vendor`), so a bare `cargo update`
resolves against the offline vendor snapshot and sees nothing to do. Fix
required temporarily removing `.cargo/config.toml` to reach the real
crates.io index, running `cargo update -p anyhow --precise 1.0.103`, then
`cargo vendor` to pull the patched crate into `vendor/anyhow` and restoring
`.cargo/config.toml` (content unchanged — the vendor-source block cargo
prints is static regardless of which crate versions it vendors).

- [x] `anyhow` bumped 1.0.102 → 1.0.103 in `Cargo.lock`; `vendor/anyhow` carries the patched source
- [x] `cargo build --release --offline --locked` succeeds
- [x] `cargo test --release --offline --locked` — 88 passed, 0 failed, 1 ignored (unchanged from pre-fix baseline)
- [x] `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean
- [x] `cargo audit --deny warnings` with CI's exact `--ignore` list (the 9 known GTK/glib/proc-macro-error advisories plus RUSTSEC-2026-0104, tracked separately as TASK-VTT070) exits 0 — RUSTSEC-2026-0190 no longer present

## Dependencies

- Directive: DIRECT-VTT002
