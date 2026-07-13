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

- [ ] A SEGV in vtt produces a retrievable coredump (`coredumpctl list vtt`) — DEFERRED: `LimitCORE=infinity` added to `packaging/linux/vtt.service` and `systemd-coredump` added to `debian/control` Recommends, but this dev sandbox has no systemd-coredump socket and `ulimit -c` is 0 (not a real systemd machine), so end-to-end capture can't be verified here. Confirms on the next real SEGV on Emmanuel's machine, or by manual smoke test (`systemctl --user daemon-reload`, then crash the process, then `coredumpctl list vtt-linux`).
- [x] FFI/unsafe audit written up — see below
- [x] Any concrete unsoundness found is fixed with a test where testable — none found in first-party code (vacuously satisfied)

## JSOC audit findings (2026-07-13)

Audited every `unsafe` block and FFI touchpoint in the crate:

- `src/whisper.rs` — zero `unsafe`. All whisper.cpp interaction goes through
  `whisper-rs`'s safe `WhisperContext`/`WhisperState`/`FullParams` wrapper;
  `WhisperEngine` creates a fresh `WhisperState` per `transcribe()` call
  (states are documented single-use) and reuses the owned `WhisperContext`
  for the life of the worker thread. No raw pointers, no manual lifetime
  management, no thread-sharing beyond the one owning worker thread.
- `src/audio.rs:104-105` — `unsafe impl Send/Sync for Audio` is the only
  unsafe surface. Verified sound: the `cpal::Stream` handle lives behind a
  `Mutex<Option<Stream>>` and is only ever replaced (never aliased) from the
  hotkey/recovery thread (`try_reopen`, audio.rs:154); every other field is
  `Arc<Mutex<..>>` or `Arc<AtomicBool>`. The stream's own capture callbacks
  (`build_input_stream` closures, audio.rs:264-330) only touch buffer/flag
  state through those same `Arc` clones — no raw pointers, no interior
  mutability outside the documented safe primitives.
- No other FFI boundaries exist in first-party code; `cpal` (ALSA/WASAPI)
  and `whisper-rs` (whisper.cpp, including the CUDA-Vulkan compute path) do
  their own unsafe FFI internally, vendored and outside this repo's direct
  control.

**Conclusion:** no unsoundness in `src/`. The 2026-07-10 SEGV most likely
originates inside the vendored `whisper-rs`/`whisper.cpp` FFI bindings or
the GPU driver (CUDA-Vulkan compute), not in application code — consistent
with "one occurrence, no repro." Per the task's own approach step 3
("otherwise document the boundary invariants and wait for a captured
core"), the fix here is coredump capture (above) plus this written
invariant record; a captured core from the next occurrence is the only
way to pinpoint the actual crash frame.

## Dependencies

- Directive: DIRECT-VTT002 (Linux voice-to-text and shared core)
- Related: TASK-VTT122 (Restart=always reduces user impact until fixed)
