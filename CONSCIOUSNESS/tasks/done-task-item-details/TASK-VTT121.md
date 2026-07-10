# TASK-VTT121: Audio capture must recover when the input device changes or suspends

## Context

Reported by Emmanuel (2026-07-10 ~01:24): "vtt is dropping our recording
transcriptions." Live diagnosis on Emmanuel's Linux machine:

- Every recording since ~01:20 logged `Recording too short (0.00s)` despite
  20-second holds — not short, **zero samples captured**.
- Root cause: vtt opens **one** cpal/ALSA capture stream at process startup
  (`src/audio.rs:206`, stored as `_stream`) and never re-opens it. Its stream
  error callback (`src/audio.rs:110`) is `|err| eprintln!(...)` — logs and does
  nothing.
- Trigger today: the RØDE VideoMic GO II (USB) **re-enumerated at 01:07:11**
  (kernel log). vtt's stream stayed bound to the now-dead node; the new node had
  no client and PipeWire idle-suspended it. Every capture returned 0 frames.
- Confirmed the mic is healthy: fresh `parecord` and `arecord -D default` both
  captured live audio (peak ~4800). Only vtt's held-open stream was dead.
- The message `Recording too short (0.00s)` conflates "empty" with "short",
  hiding the real failure from the user.

**Second recurrence (2026-07-10 03:50):** worked until 02:07, then broke again
after Emmanuel logged out and back in — no USB change. Logout restarts the
PipeWire/WirePlumber session; vtt (same PID, held stream) stayed bound to the
dead old session and captured 0 frames. Same root cause, third distinct trigger
(idle-suspend, USB re-enumeration, session restart) — proving the fix must be
device/session-agnostic stream recovery, not a fix per trigger.

Immediate machine-level mitigations applied (not code fixes):
1. WirePlumber drop-in `~/.config/wireplumber/main.lua.d/51-disable-idle-suspend.lua`
   sets `session.suspend-timeout-seconds = 0` (kills idle-suspend).
2. systemd drop-in `~/.config/systemd/user/vtt.service.d/restart-on-session.conf`
   adds `PartOf=graphical-session.target` so logout/login restarts vtt and
   rebinds the stream (covers the session-restart trigger).
Neither covers USB re-enumeration mid-session — that still needs the code fix.

## Approach

Make capture self-healing and honest, portably (cpal 0.15, all platforms):

1. Treat the stream error callback as a signal, not a print — flag the stream
   dead and trigger a re-open against the current default input device.
2. Add a zero-frame watchdog: when a recording ends with 0 samples, attempt one
   stream re-open before reporting, so a device that changed under us recovers
   without a manual restart.
3. Classify the outcome honestly: 0 captured frames → a distinct
   `NoAudioCaptured` result (log + tray/notification "no audio — check
   microphone"), never `TooShort`. Keep `TooShort` for genuine <0.5s real audio.

## Acceptance criteria

- [ ] After the default input device re-enumerates or suspends while vtt runs,
      the next recording captures audio with no manual restart (reproduce by
      re-plugging the USB mic / toggling the default source)
      *(deferred: live-hardware verification — mechanism shipped 2026-07-10;
      verify on next real USB re-plug after installing the release)*
- [ ] After the PipeWire session restarts under vtt (e.g. logout/login) the next
      recording captures audio with no manual restart
      *(deferred: live verification on next logout/login with the release
      installed; the systemd PartOf drop-in independently covers this trigger)*
- [x] A zero-frame capture is reported as `NoAudioCaptured` (distinct log line +
      user-visible tray/notification), not `Recording too short (0.00s)`
- [x] The stream error callback (`src/audio.rs:110`) triggers a re-open rather
      than only printing to stderr (flags the stream dead; next
      `start_recording` re-opens)
- [x] A genuine <0.5s recording of real audio still reports `TooShort`
- [x] Unit test covers empty-capture vs too-short classification (7 tests on
      the pure `classify_capture`)

## Resolution (2026-07-10, commit bb326c7)

Three-layer recovery in `src/audio.rs`, device/session-agnostic:
1. Stream error callback sets a `stream_dead` flag (was print-only).
2. `start_recording` re-opens a dead stream before capturing.
3. Zero-sample capture watchdog: `stop_recording` re-opens the stream and
   returns the new `NoAudioCaptured` result; `main.rs` shows "No audio — check
   microphone" (tray status + error icon + notify-send on Linux).

Stream construction extracted to `open_capture_stream` (same 16 kHz-direct →
native-format fallback chain); `Audio._stream` became
`Mutex<Option<cpal::Stream>>` so recovery can replace it at runtime.
95 tests, clippy `-D warnings`, fmt all green.

## Follow-ups (separate tasks, filed when picked up)

- Service restart semantics: unit is `Restart=on-failure` but the tray Quit
  calls `std::process::exit(0)` (`src/tray/portable.rs:246`), a clean exit that
  systemd will not restart — reads as "service won't restart". Fix: `Restart=always`
  + tray Quit runs `systemctl --user stop vtt` so intentional quit and
  crash-recovery both work.
- SIGSEGV: `vtt.service: Main process exited, code=killed, status=11/SEGV` at
  01:22:15 — a crash at the whisper/CUDA or ALSA FFI boundary. Enable coredumps
  and guard the boundary (JSOC reliability).

## Dependencies

- Directive: DIRECT-VTT005 (cross-platform feature parity) — capture reliability
  is a parity-critical behaviour across Linux/macOS/Windows
