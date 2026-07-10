# TASK-VTT124: Guard the SIGSEGV at the whisper/FFI boundary — coredumps + JSOC audit

## Context

Follow-up filed from the TASK-VTT121 diagnosis session. Journal showed:

    vtt.service: Main process exited, code=killed, status=11/SEGV  (2026-07-10 01:22:15)

A segfault at the whisper.cpp/CUDA-Vulkan or ALSA FFI boundary. One occurrence,
no coredump captured (coredumps not enabled for the user unit), so the crash
site is unknown. With TASK-VTT122's `Restart=always` the user impact of a rare
SEGV becomes a 5-second gap, but the crash itself should be found and fixed.

## Approach

1. Enable coredump capture for the vtt user unit (`LimitCORE=infinity` drop-in
   or systemd-coredump verification) so the next SEGV is debuggable.
2. JSOC-style audit of the `unsafe`/FFI surfaces: whisper-rs context/state
   calls in `src/whisper.rs`, the cpal stream callbacks in `src/audio.rs`
   (including the `unsafe impl Send/Sync for Audio`), and any raw pointer or
   lifetime assumptions at those boundaries.
3. Fix what the audit finds; otherwise document the boundary invariants and
   wait for a captured core.

## Acceptance criteria

- [ ] A SEGV in vtt produces a retrievable coredump (`coredumpctl list vtt`)
- [ ] FFI/unsafe audit written up (findings in this card or a linked artifact)
- [ ] Any concrete unsoundness found is fixed with a test where testable

## Dependencies

- Directive: DIRECT-VTT002 (Linux voice-to-text and shared core)
- Related: TASK-VTT122 (Restart=always reduces user impact until fixed)
