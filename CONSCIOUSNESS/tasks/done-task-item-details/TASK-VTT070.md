# TASK-VTT070: Vendor refresh — rustls-webpki security upgrade

## Acceptance Criteria
1. [x] `rustls-webpki` is at `>=0.103.13` in `Cargo.lock` — now 0.103.13
2. [x] `cargo audit` exits 0 with no RUSTSEC-2026-0104 finding — cleared, verified on Linux
3. [ ] CI build passes on Linux and Windows after the upgrade — Linux verified locally (build + 98 tests pass) and via ci.yml on push. DEFERRED (Windows leg): `.github/workflows/ci.yml` only runs `ubuntu-24.04` today — no Windows CI matrix exists yet (that's TASK-VTT048/FEAT-VTT031, still backlog). Nothing platform-specific in this change (pure dependency bump behind existing rustls usage), so no Windows-specific risk is expected, but the criterion can't be marked done until TASK-VTT048 ships a Windows CI leg to actually run it.
