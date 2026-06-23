# TASK-VTT063: Windows x86-64 compile-green and CI build job

## Context
The repo has a full `cfg(target_os = "windows")` dependency block and
`portable.rs` hotkey/tray modules, but **nothing has ever compiled them**.
CI only builds `ubuntu-24.04`. This task gets the first Windows build to
compile green in CI — the missing prerequisite that TASK-VTT044/045/046/047
all silently assume.

This is gensho: until a Windows compiler runs over the code, "Windows-ready"
is theory. CI is the only practical verifier — local cross-compilation is
infeasible (host is Linux with `cargo vendor` offline config, and whisper-rs
builds C++ whisper.cpp which can't cross to MSVC).

## Known compile blockers (found by inspection)
1. **`crossbeam-channel` not a direct dependency.** `src/tray/portable.rs`
   names `crossbeam_channel::Receiver<MenuEvent>` in `handle_menu_events`,
   but `crossbeam-channel` is only a *transitive* dep (via muda/tray-icon).
   The bare path won't resolve on Windows/mac. Fix: add `crossbeam-channel`
   as a direct dep under `cfg(any(target_os = "macos", target_os = "windows"))`.
2. **whisper-rs Vulkan on Windows = heavy CI setup.** First cut switches the
   Windows target to CPU-only whisper (`default-features = false`, no
   acceleration feature) so CI needs no Vulkan SDK. Also de-risks the ARM
   target (TASK-VTT064) where Adreno/Vulkan is unproven.

`singleton_lock` (flock) and `ctrlc_handler` (sigwait) are `#[cfg(unix)]`
with no required Windows counterpart, so they do NOT block compilation —
Windows simply runs without a singleton or Ctrl-C handler for now. Adding
those is TASK-VTT044 / TASK-VTT045, sequenced after green.

## Acceptance Criteria
1. `Cargo.toml` adds `crossbeam-channel` as a direct dep for macOS+Windows.
2. `Cargo.toml` Windows whisper-rs is CPU-only (no `vulkan` feature) with a
   comment pointing at the GPU-acceleration follow-up.
3. Linux and macOS dependency blocks are unchanged (no build skew).
4. `.github/workflows/ci.yml` gains a `build-windows` job on `windows-latest`
   that runs `cargo build --all-targets` (the compile gate).
5. The Windows job does NOT inherit `-D warnings` (override `RUSTFLAGS: ""`) —
   the gate is "does it compile", not lint-clean.
6. Work lands on a branch + PR so main never goes red; CI on the PR is the
   proof. The job either passes or surfaces the next concrete compile error.

## Test Strategy
Push branch, open PR, read the `build-windows` job log. Green = compiles.
Red = the log names the next blocker; iterate on the branch until green.

## Files
- `Cargo.toml`
- `.github/workflows/ci.yml`
