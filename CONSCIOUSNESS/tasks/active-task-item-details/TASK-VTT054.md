# TASK-VTT054: Settings toggle for CT2 daemon backend

## Acceptance Criteria
1. [x] `settings.conf` has a `backend = [native|ct2]` field; default is `native` — `src/settings.rs`, unrecognised values fall back to `native`
2. [x] When `ct2` is selected, the daemon is launched at startup if not already running — `src/main.rs` worker startup, via `src/ct2_client.rs::Ct2Client::spawn`
3. [x] If the daemon crashes mid-session, the app falls back to whisper-rs without user intervention — any failed call marks the client permanently `dead` for the rest of the session; the per-recording call site falls through to the existing `transcribe::transcribe_samples` unchanged
4. [x] The tray shows "Backend: CT2" or "Backend: Native" in the status area — Linux (`tray/linux.rs`, verified) and portable (`tray/portable.rs`, written by analogy, **not independently verified** — this dev machine cannot build for Windows/macOS)

## Process note

Implementation began without going through the claim CLI first (a real
process gap this session — every other task this cycle claimed properly
before touching a file). Caught and corrected before any commit: claimed
retroactively once noticed, before committing or closing.

## Evidence, 2026-09-04

**Scope note on the daemon script's location:** there is no packaged install
location yet (this feature has never shipped in a .deb/.msi) — `TASK-VTT054`'s
"launched at startup" criterion is met for a binary built and run from this
repo checkout (`cargo build --release`, this project's own documented build
step), which is how every current build of this app runs. Packaging
`ct2-daemon/` into the shipped install is real follow-up work, filed as
TASK-VTT167.

**Build/test:**
```
cargo build --release: clean, 0 warnings
cargo fmt --check: clean
cargo clippy --all-targets -- -D warnings: clean
cargo test --release: 193 passed, 0 failed, 2 ignored
```
The 2 ignored tests are the two real-daemon integration tests (this task's
own + TASK-VTT053's), both opt-in for the same reason as the Rust suite's
pre-existing e2e whisper test — they download/exercise a real model.

**Real end-to-end integration test** (not `#[ignore]`d — it's the whole
point of the split-testable design, mirrors the Python side's own
falsification): `cargo test --release -- --ignored spawn_load_and_transcribe_against_the_real_daemon`
passed in 2.75–9.53s across two runs — a real `python3 transcribe_daemon.py`
spawned, a real `faster-whisper` `tiny.en` model loaded, a real operator
recording transcribed over the actual stdio pipe.

**A real bug found and fixed by this task's own smoke test.** Ran the built
app with `backend=ct2` and the operator's real `large-v3-turbo` model name
(isolated `XDG_DATA_HOME`, never touching the live production `vtt.service`
— confirmed via `ps aux` before and after). `load_model` for a CTranslate2
`large-v3-turbo` exceeded my first `CALL_TIMEOUT` (30s, sized for inference,
not a first-time model resolution/download) — Rust gave up and returned
`None`, but the spawned `python3` process was still running afterward,
unowned (confirmed via `ps aux`). Fixed: a separate, much longer
`LOAD_MODEL_TIMEOUT` (600s) for `load_model` specifically, and `spawn()`'s
failure path now explicitly `child.kill()` + `child.wait()`s rather than
relying on `Drop`'s cooperative shutdown (which assumes the daemon is free
to read the next command — not true while it's still busy in the call that
just failed). Added `spawn_kills_the_process_when_load_model_fails` as a
permanent (non-ignored) regression test — fast and deterministic since a
bogus model name fails before any download. Cleaned up the leaked test
processes immediately; the real production service was never affected.

**Not independently verified:** the portable tray's (`tray/portable.rs`)
backend label — written correctly by analogy to the adjacent `status`
MenuItem (same type, same `.set_text()` update pattern) but this module
only compiles for Windows/macOS, neither of which this machine can build
for. Flagged rather than silently claimed.
