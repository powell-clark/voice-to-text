# TASK-VTT070: Vendor refresh — rustls-webpki security upgrade

## Acceptance Criteria
1. `rustls-webpki` is at `>=0.103.13` in `Cargo.lock`
2. `cargo audit` exits 0 with no RUSTSEC-2026-0104 finding
3. CI build passes on Linux and Windows after the upgrade
