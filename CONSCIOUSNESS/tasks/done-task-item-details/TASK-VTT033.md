# TASK-VTT033: Add #[cfg(unix)] guards for Windows compilation

## Context
`src/main.rs` contains two Unix-only helpers with no `#[cfg]` guards:
- `singleton_lock()` at lines 372-388 uses `std::os::unix::io::AsRawFd` and `libc::flock(LOCK_EX | LOCK_NB)`
- `ctrlc_handler()` at lines 417-436 uses `libc::sigemptyset`, `libc::sigaddset`, `libc::pthread_sigmask`, `libc::sigwait`

These fail to compile on Windows. STORY-VTT013 (Windows build) implements proper Windows equivalents; this task adds the compile-time guards so Linux/macOS builds continue to work while Windows builds degrade gracefully with a clear message until TASK-VTT044 and TASK-VTT045 fill in the real implementations.

## Acceptance Criteria
1. `singleton_lock` has `#[cfg(unix)]` directly above its `fn` declaration; call sites are wrapped in `#[cfg(unix)] let _lock_fd = singleton_lock(&config_dir)?;` and `#[cfg(windows)]` uses a placeholder returning `Ok(())` with a log `"Singleton check not implemented on Windows yet"`
2. `ctrlc_handler` has `#[cfg(unix)]` guard; call site is `#[cfg(unix)] ctrlc_handler(...)`; on Windows, a placeholder registers nothing with a log `"Signal handler not implemented on Windows yet (see TASK-VTT045)"`
3. `cargo check --target x86_64-unknown-linux-gnu` succeeds
4. `cargo check --target x86_64-pc-windows-gnu` succeeds (even without the real implementations) — the code compiles with placeholders
5. The Linux binary behaviour is unchanged — singleton and signal handling work exactly as before
6. A TODO comment above each placeholder references the specific task that will implement it: `// TODO(TASK-VTT044): CreateMutexW-based singleton`, `// TODO(TASK-VTT045): SetConsoleCtrlHandler`

## Technical Approach
```rust
#[cfg(unix)]
fn singleton_lock(config_dir: &std::path::Path) -> anyhow::Result<std::fs::File> {
    // existing implementation
}

#[cfg(windows)]
fn singleton_lock(_config_dir: &std::path::Path) -> anyhow::Result<()> {
    vtt_log!("Singleton check not implemented on Windows yet (see TASK-VTT044)");
    Ok(())
}
```

Do the same for `ctrlc_handler`.

## Test Strategy
`cargo check --target x86_64-pc-windows-gnu` (install the target with `rustup target add x86_64-pc-windows-gnu`). Verify no compile errors. Linux build is verified by TASK-VTT034 end-to-end smoke test.

## Files
- `src/main.rs` (modify — add cfg guards and Windows placeholders)
