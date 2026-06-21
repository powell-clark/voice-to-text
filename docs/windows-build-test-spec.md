# Windows 11 Build and Test Specification

**Issue**: #2 — Request build and testing on Windows 11  
**Status**: Draft  
**Last updated**: 2026-06-21

---

## Overview

This document specifies the requirements, gaps, and acceptance criteria for building and
testing voice-to-text on Windows 11. The project already has partial Windows support
(portable tray, hotkey, and audio dependencies in `Cargo.toml`) but has never been
built or tested on Windows. This spec closes the gap.

---

## Current State

### What is already wired for Windows

| Component | Implementation | Location |
|-----------|---------------|----------|
| System tray | `tray-icon` + `muda` crates | `src/tray/portable.rs` |
| Global hotkey | `rdev` crate | `src/hotkey/portable.rs` |
| Text injection | `enigo` crate | `src/typing.rs` |
| Audio capture | `cpal` crate (cross-platform) | `src/audio.rs` |
| Clipboard | `arboard` crate (cross-platform) | — |
| Whisper inference | `whisper-rs` with `vulkan` feature | `Cargo.toml` |
| Config/data dir | `dirs::data_local_dir()` → `%LOCALAPPDATA%` on Windows | `src/main.rs` |

### Known gaps (blocking a Windows build)

| Gap | Location | Severity | Status |
|-----|----------|----------|--------|
| Singleton lock is `#[cfg(unix)]` only; Windows skips it | `src/main.rs` | Medium — multiple instances can run | **Fixed** — `singleton_lock_windows()` via `CreateMutexW` |
| Ctrl+C / signal handler is `#[cfg(unix)]` only | `src/main.rs` | Medium — graceful shutdown missing | **Fixed** — `setup_ctrl_handler()` via `SetConsoleCtrlHandler` |
| `cleanup_old_wavs` hardcodes `/tmp` | `src/main.rs` | Low — temp files accumulate in wrong dir | **Fixed** — uses `std::env::temp_dir()` |
| Help text references Linux paths and binary name | `src/main.rs` | Low — cosmetic, wrong paths shown | **Fixed** — `#[cfg(target_os = "windows")]` conditional paths |
| Binary target name is `vtt-linux` in `Cargo.toml` | `Cargo.toml:10` | High — must be renamed or made conditional | Open — rename or post-build step |
| Buffer-full notification is a no-op on non-Linux | `src/main.rs` | Low — feature parity | Open — tracked as STORY-VTT013 |
| No Windows CI job | `.github/workflows/ci.yml` | High — nothing is verified | Open — see CI Job Specification below |

---

## Build Requirements

### Developer workstation

- Windows 11 (22H2 or later)
- [Rust stable toolchain](https://rustup.rs/) with `x86_64-pc-windows-msvc` target
- [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/downloads/) with
  "Desktop development with C++" workload (provides MSVC linker + CRT)
- [CMake 3.20+](https://cmake.org/download/) — required by `whisper-rs` build script
- [LunarG Vulkan SDK](https://vulkan.lunarg.com/sdk/home#windows) — required by
  `whisper-rs` vulkan feature; installs `glslc` and sets `VULKAN_SDK` env var
- Git for Windows

### GitHub Actions CI runner

Use `windows-2022` (GitHub-hosted). It ships with:
- MSVC Build Tools and MSVC linker
- CMake
- Git

The Vulkan SDK must be installed explicitly via the
[`humbletim/setup-vulkan-sdk`](https://github.com/humbletim/setup-vulkan-sdk) action
or a manual download + env-var step.

### Environment variables required at build time

```
VULKAN_SDK=C:\VulkanSDK\<version>        # set by LunarG installer / setup action
PATH includes %VULKAN_SDK%\Bin           # for glslc.exe
```

---

## Binary Naming

The `Cargo.toml` `[[bin]]` section names the binary `vtt-linux`. This must be
conditional or renamed before Windows builds produce a usable artifact.

**Recommended approach** — use a build script or a second `[[bin]]` section:

```toml
[[bin]]
name = "vtt-linux"
path = "src/main.rs"
required-features = []   # keep existing entry for Linux/macOS

# Cargo does not support OS-conditional bin names natively.
# Workaround: rename the produced artifact in the CI step:
#   Move-Item target\release\vtt-linux.exe vtt-windows.exe
```

Until a cleaner solution is implemented, the CI step can rename the artifact
post-build. The binary will compile as `vtt-linux.exe` on Windows — functionally
correct but cosmetically wrong.

---

## Code Changes Required Before CI Can Pass

### 1. Singleton lock — `src/main.rs` ✅ Implemented

`singleton_lock_windows()` uses `CreateMutexW` with a named mutex
(`Global\VoiceToTextSingleton`). Returns an error if the mutex already exists
(another instance running). The handle is stored in a static `AtomicUsize` so it
lives for the process lifetime — Windows releases it automatically on exit.

No new crate dependencies; implemented via raw `extern "system"` FFI.

### 2. Signal handler — `src/main.rs` ✅ Implemented

`setup_ctrl_handler()` calls `SetConsoleCtrlHandler` with a handler that sets the
shared `running` flag to `false` on `CTRL_C_EVENT` (0), `CTRL_BREAK_EVENT` (1), or
`CTRL_CLOSE_EVENT` (2). The main loop's existing `loop { sleep(100ms); if !running
{ break; } }` then exits cleanly. No new crate dependencies; raw `extern "system"`
FFI.

### 3. Temp directory for WAV cleanup ✅ Implemented

Was: `cleanup_old_wavs_in(std::path::Path::new("/tmp"), cutoff)` — hardcoded `/tmp`, which does not exist on Windows. Fixed to:

```rust
fn cleanup_old_wavs() {
    let cutoff = std::time::SystemTime::now() - Duration::from_secs(3600);
    let tmp = std::env::temp_dir();
    let cleaned = cleanup_old_wavs_in(&tmp, cutoff);
    if cleaned > 0 {
        vtt_log!("Cleaned up {} old WAV files from {}", cleaned, tmp.display());
    }
}
```

This was the only **code change required** before `cargo build --release` can succeed
on Windows (assuming Vulkan SDK is present). All other gaps are either already
compile-gated or are runtime-only issues.

### 4. Help text paths ✅ Implemented

Help text now shows Windows paths conditionally:

```rust
#[cfg(target_os = "windows")]
println!("Config:   %LOCALAPPDATA%\\voice-to-text\\settings.conf\n...");
#[cfg(not(target_os = "windows"))]
println!("Config:   ~/.local/share/voice-to-text/settings.conf\n...");
```

This is cosmetic and not a blocker.

---

## CI Job Specification

Add a `windows` job to `.github/workflows/ci.yml` in parallel with the existing
`check` (Ubuntu) job.

```yaml
windows:
  name: build + test (windows-2022)
  runs-on: windows-2022

  steps:
    - name: Checkout
      uses: actions/checkout@v4
      with:
        submodules: recursive

    - name: Install Vulkan SDK
      uses: humbletim/setup-vulkan-sdk@v1.2.0
      with:
        vulkan-query-version: latest
        vulkan-components: Vulkan-Headers, Vulkan-Loader, glslc
        vulkan-use-cache: true

    - name: Install Rust toolchain (stable, MSVC)
      uses: dtolnay/rust-toolchain@stable
      with:
        targets: x86_64-pc-windows-msvc
        components: rustfmt, clippy

    - name: Cache cargo registry and target
      uses: Swatinem/rust-cache@v2
      with:
        cache-on-failure: true

    - name: cargo fmt --check
      run: cargo fmt --all -- --check

    - name: cargo clippy
      run: cargo clippy --release --all-targets -- -D warnings

    - name: cargo test
      run: cargo test --release

    - name: cargo build --release
      run: cargo build --release

    - name: Verify binary exists and reports version
      run: |
        $bin = "target\release\vtt-linux.exe"
        if (-not (Test-Path $bin)) { Write-Error "Binary not found"; exit 1 }
        & $bin --version
      shell: pwsh
```

**Note**: `rdev::listen` requires the Accessibility permission on macOS; on Windows
it may require running as administrator or granting `UIPI` bypass for low-integrity
processes. CI runners run as administrator, so this is not a blocker for CI, but
end-user installs need documentation.

---

## Testing Scope

### Automated (cargo test — all platforms)

All existing unit tests in `src/main.rs` are platform-agnostic (pure functions). They
must pass on Windows without modification:

- `compose_final_text_*` (8 tests)
- `is_whisper_filler_*` (3 tests)
- `migrate_legacy_model_name_*` (6 tests)
- `prune_recordings_*` (4 tests)
- `cleanup_old_wavs_in_*` (4 tests)
- `keycode_to_rdev_*` tests in `src/hotkey/portable.rs` (4 tests)

### Manual smoke test (first successful Windows build)

| Test | Expected result |
|------|----------------|
| Launch `vtt-linux.exe --version` | Prints `voice-to-text 2.x.x` |
| Launch `vtt-linux.exe` | System tray icon appears (green dot) |
| Right-click tray icon | Menu shows Status, Language, Model, Logging, About, Quit |
| Hold Scroll Lock and speak | Tray shows "Recording..." |
| Release Scroll Lock | Tray shows "Transcribing..." then "Ready", text typed at cursor |
| Select a model from tray menu | Model downloads, engine reloads |
| Quit from tray menu | Process exits cleanly |

### Known manual test limitations

- `rdev::listen` may show a UAC prompt or fail silently if accessibility is not
  granted. Document the requirement.
- The tray icon is a plain coloured dot (22×22 RGBA) — not a proper Windows `.ico`.
  Acceptable for initial support; tracked separately.
- Notifications are logged only (no Windows toast). Tracked as STORY-VTT013.

---

## Packaging (out of scope for this spec)

Windows packaging (`.msi`, `.exe` installer, winget manifest) is a follow-on story.
This spec covers only build and test. The `scripts/release-local.sh` and
`scripts/release-ppa.sh` are Linux/Ubuntu-specific and do not apply.

---

## Acceptance Criteria

- [ ] `cargo build --release` succeeds on `windows-2022` GitHub Actions runner
- [ ] `cargo test --release` passes with zero failures on `windows-2022`
- [ ] `cargo clippy --release -- -D warnings` produces no errors on `windows-2022`
- [ ] The produced binary prints the correct version via `--version`
- [ ] The Windows CI job runs in parallel with the existing Ubuntu job on every PR to `main`
- [x] The `cleanup_old_wavs` function uses `std::env::temp_dir()` instead of `/tmp`
- [x] Help text shows platform-correct paths (`%LOCALAPPDATA%` on Windows)
- [x] Singleton lock prevents multiple instances on Windows (`CreateMutexW`)
- [x] Graceful shutdown on Ctrl+C/Ctrl+Break on Windows (`SetConsoleCtrlHandler`)

---

## Implementation Order

1. ✅ Fix `cleanup_old_wavs` to use `std::env::temp_dir()` — unblocks the build
2. ✅ Fix help text paths — cosmetic
3. ✅ Implement singleton lock via `CreateMutexW` (no new deps, raw FFI)
4. ✅ Implement `SetConsoleCtrlHandler` (no new deps, raw FFI)
5. Add the `windows-2022` CI job — catches any remaining issues automatically
6. Windows toast notifications — STORY-VTT013
