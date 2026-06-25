# TASK-VTT082: Build and smoke-test VTT on Windows x86-64 hardware

## Context

This is the entry point for the Windows handoff. Currently all development is on
Linux; a Windows session will pick this card up cold on a real Windows x86-64
machine. TASK-VTT063 already proved the code is compile-green and builds in CI —
this task is the first **on-hardware run-and-verify**: does VTT actually record,
transcribe, and inject text on Windows in front of a human?

Read STORY-VTT013 (Windows builds) before starting.

## Runbook (for the Windows session)

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

- [ ] `cargo build --release` succeeds on Windows x86-64 hardware (not just CI)
- [ ] vtt-linux.exe launches and the tray icon appears
- [ ] Default Whisper model downloads/loads on first run
- [ ] Push-to-talk records and stops cleanly
- [ ] Whisper transcribes speech to text on Windows
- [ ] Text injects into a focused Windows application
- [ ] GPU vs CPU inference path on Windows is recorded
- [ ] Any Windows-specific defects are captured as new tasks

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
- Features: FEAT-VTT030
- Builds on: TASK-VTT063 (compile-green + CI, done)
- Blocks: TASK-VTT064 (ARM64 build) → TASK-VTT047 (Authenticode signing)
