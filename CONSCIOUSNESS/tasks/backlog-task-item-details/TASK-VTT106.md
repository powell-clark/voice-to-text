# TASK-VTT106: cargo audit red — RUSTSEC-2026-0186 (memmap2 unsound)

CI's `cargo audit` job fails repo-wide (including on `main`) on a newly-published
advisory: **RUSTSEC-2026-0186 — "Unchecked pointer offset in crate `memmap2`"**
(class: `unsound`). The job runs with `--deny warnings`, so an unsound warning
is treated as a denied finding and exits 1. `memmap2` is a transitive dependency
(whisper-rs mmaps the model file), not a direct one.

This is the "ignore-list grows on every new advisory" fragility flagged in the
CI/CD audit — the audit job already ignores 10 RUSTSEC advisories (the gtk-rs
unmaintained cluster, rustls-webpki, etc.).

Two candidate fixes — **operator decision** (security posture):
- [ ] **(A) Ignore** — add `--ignore RUSTSEC-2026-0186` to the `audit` step in
      `.github/workflows/ci.yml` with an inline rationale, matching the existing
      unsound/unmaintained-class policy. One line; unblocks CI immediately.
- [ ] **(B) Upgrade** — if a patched `memmap2` exists, `cargo update -p memmap2`
      to the fixed release and re-run audit. Cleaner, but churns Cargo.lock and
      depends on a fix being published + compatible with the vendored dep tree.

Acceptance: `cargo audit` green on `main` again, with the chosen path documented.

- Surfaced during TASK-VTT105 (Claude workflow parity) PR review.
