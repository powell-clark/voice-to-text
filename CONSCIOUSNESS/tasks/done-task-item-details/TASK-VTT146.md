# TASK-VTT146: Fix hotkey release lost during typing wait

## Context

Reported 2026-08-18: VTT "kept typing and I couldn't stop it, and it kept
transcribing the same thing again and again, and it's been capturing without
the button pressed down".

All three symptoms are one defect. The key-down handler waited up to 30s for
the previous transcription to finish typing, and did the waiting inside the
X11 event callback — the single thread that also delivers KeyRelease. The
user's real release queued behind that wait and was delivered the instant it
ended, ~0ms after the recording start, where a guard intended for auto-repeat
artefacts discarded it. The recording flag stayed true and no later key could
clear it, because key-down returns early while recording is true.

The mic then stayed open, the long capture clipped, and Whisper returned the
same hallucinated token from every saturated buffer — each one typed.

Field evidence from journalctl, 2026-08-18 04:38-04:39:

    04:38:55 Key pressed - starting recording
    04:38:55 Ignoring stale KeyRelease (0ms)     <- genuine release, dropped
    04:39:00 Key released - stopping recording   <- 5s later, a LATER tap
    ...
    04:39:08 Key pressed - starting recording
    04:39:08 Ignoring stale KeyRelease (0ms)
    [12s of open mic until systemd stopped the unit]

Three holds ran 4.6s, 6.2s and 12s with the key already up, all at amplitude
32767 (clipping), and all three produced the identical transcription
`*throwing*` from buffers of 15019, 73728 and 98987 samples.

## Acceptance criteria

- [x] A release is never discarded — `PushToTalk::release` always stops a
      running recording, regardless of how briefly the key was held
- [x] The typing wait no longer runs on the X11 event thread; it moves to a
      worker thread so KeyRelease delivery is never delayed
- [x] The starter thread holds the gate across the capture start, so a release
      arriving mid-start waits rather than racing past it
- [x] A watchdog force-stops any hold outliving `MAX_HOLD`, so a release lost
      below this layer still cannot strand the microphone
- [x] Press/release decisions are unit-testable without an X server, and the
      wedge is covered by a named regression test
- [x] Regression test proven genuine: reverting `release` to the old 150ms
      guard fails `release_arriving_instantly_still_stops_the_recording` and
      `a_wedged_recording_can_never_block_the_next_one`, with the fix green
- [x] Full suite green — 127 passed, 0 failed
- [x] Verified live against a real X server: a 39ms tap logs
      `Key released - stopping recording (39ms)` then `Recording too short
      (0.04s)`, where the old build would have logged `Ignoring stale
      KeyRelease (39ms)` and left the mic open
- [x] Repeated fast taps (15ms, 14ms) each stop cleanly with no wedge
      accumulating across presses

## Notes

A short hold is still discarded — that behaviour was never the problem. It is
now discarded by `audio.rs`'s `MIN_DURATION_SECS` check, which closes the
microphone, instead of by a guard that left it open.

Separately observed while diagnosing: this machine's `settings.conf` carries
`hotkey=65`, which is the space bar. Every space keypress therefore opens a
recording, which is why this race was hit so often. Raised with the operator;
tracked as TASK-VTT147 (Warn when the hotkey is an ordinary typing key).

## Dependencies

- Story: STORY-VTT015
- Directive: DIRECT-VTT002
