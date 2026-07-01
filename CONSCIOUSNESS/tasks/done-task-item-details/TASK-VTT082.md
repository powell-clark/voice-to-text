# TASK-VTT082: Build and smoke-test VTT on Windows x86-64 hardware

## Context

This is the entry point for the Windows handoff. Currently all development is on
Linux; a Windows session will pick this card up cold on a real Windows x86-64
machine. TASK-VTT063 already proved the code is compile-green and builds in CI —
this task is the first **on-hardware run-and-verify**: does VTT actually record,
transcribe, and inject text on Windows in front of a human?

Read STORY-VTT013 (Windows builds) before starting.

## Verified from Linux (2026-06-25, TASK-VTT084)

- Windows CI job is **green** on current main (`windows-latest, x86_64-msvc, CPU
  whisper`) — the code compiles on Windows now.
- No `todo!`/`unimplemented!` stubs in any Windows/shared path.
- Text injection rides on `enigo` (cross-platform), audio on `cpal` (WASAPI),
  tray on `tray-icon`, hotkey on `rdev` — all present for Windows.
- Binary name is `vtt-linux.exe` ([[bin]] name = vtt-linux).
- CI's red `cargo audit` job is unrelated (RUSTSEC-2026-0185 / quinn-proto,
  tracked as TASK-VTT085) — not a build blocker.

## Quick path

From the repo root in PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\smoke-test-windows.ps1
```

It pulls, builds release, confirms the binary, and prints the manual checklist.
Prerequisites (install once): Rust (rustup), VS Build Tools C++ workload (MSVC
linker), LLVM/Clang for bindgen (set LIBCLANG_PATH if needed).

## Runbook (manual, what the script automates + the human checks)

1. `git pull origin main`
2. `cargo build --release` — produces `target/release/vtt-linux.exe`
3. Run the binary; confirm the system-tray icon appears and the default model
   downloads / loads (FEAT-VTT026, FEAT-VTT028).
4. Hold the push-to-talk hotkey, speak, release — confirm recording starts/stops.
5. Confirm Whisper transcribes in-process (sub-second after warm load,
   FEAT-VTT022) with no Python dependency (FEAT-VTT023).
6. Confirm transcribed text injects into a focused app (Notepad, browser,
   terminal). Note which injection path Windows uses vs Linux XTest (FEAT-VTT005).
7. Note GPU path: does whisper-rs use Vulkan on Windows (FEAT-VTT024) or fall
   back to CPU? Record what was observed.
8. File a follow-up task for any Windows-specific defect found (do not fix
   silently — capture it).

## Acceptance criteria

- [x] `cargo build --release` succeeds on Windows x86-64 hardware (not just CI)
- [x] vtt-linux.exe launches — reaches "All systems initialized" (tray icon
      visibility is the one manual perceptual check below)
- [x] Default Whisper model downloads/loads on first run (download verified
      starting; small.en from HuggingFace)
- [ ] Push-to-talk records and stops cleanly — **manual, needs a mic + human**
- [x] Whisper transcribes speech to text on Windows — proven by the E2E test
      (SAPI speech fixture → base.en → correct transcript), TASK-VTT087
- [ ] Text injects into a focused Windows application — **manual, needs a mic +
      focused window**
- [x] GPU vs CPU inference path on Windows is recorded — **CPU** ("no GPU found";
      the Windows build is deliberately CPU-only, see Cargo.toml)
- [x] Any Windows-specific defects are captured/fixed — fixed two launch-blocking
      defects (audio stream-config, write_wav /tmp); captured tray model-menu
      mismatch as TASK-VTT086

## Verification (2026-06-25, on Windows 11 x86-64)

Toolchain bootstrapped on a cold machine: Rust via rustup, libclang via the
PyPI `libclang` wheel (LLVM not needed wholesale), cmake + ninja from the VS
2022 Build Tools bundle. Build green in ~55s → `vtt-linux.exe` (4.84 MB).
`cargo test` 67 unit tests green; E2E transcription test green. App launched,
audio capture started (48 kHz native → 16 kHz resample), Scroll Lock hotkey
monitor started, model download began. MSI built (2.11 MB) via cargo-wix + WiX
3.14. Two launch-blocking Windows defects found and fixed (see CHANGELOG 2.2.0).
Remaining open ACs are physical mic checks only — left for Emmanuel.

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Features: FEAT-VTT030
- Builds on: TASK-VTT063 (compile-green + CI, done)
- Blocks: TASK-VTT064 (ARM64 build) → TASK-VTT047 (Authenticode signing)
