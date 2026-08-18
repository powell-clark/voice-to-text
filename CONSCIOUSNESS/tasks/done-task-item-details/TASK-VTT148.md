# TASK-VTT148: Fast tap misreported as a dead microphone

## Context

Reported 2026-08-18: "If you tap the record button but don't say anything, it
holds recording and doesn't stop. Or at least the icon holds as red."

The icon is red because the tap is being diagnosed as a dead microphone. A tap
can end before cpal's first callback fires, so zero samples arrive.
`classify_capture` treated any zero-sample capture as a dead stream regardless
of how long the key was held, returning `NoAudioCaptured` — which sets a sticky
red error icon, fires a notify-send, and re-opens the capture stream for
nothing.

The old 150ms release guard hid this by discarding fast taps before they ever
reached `stop_recording`. Honouring the release (TASK-VTT146) exposed the
misclassification underneath. The premise was even written into the code:
"Zero samples is a dead stream, never a hasty user" — true only while the guard
shielded it.

Reproduced on the installed 2.3.10 build:

    10:08:07 Key released - stopping recording (14ms)
    10:08:07 No audio captured (0 samples) — re-opening capture stream
    10:08:07 No audio captured — mic changed/suspended; stream re-opened

## Acceptance criteria

- [x] `Audio` stamps the hold start in `start_recording` and measures the real
      hold duration in `stop_recording`, rather than inferring duration from
      the sample count — which is zero in exactly the case under test
- [x] `classify_capture` returns `TooShort` for a zero-sample capture whose
      hold was under `MIN_DURATION_SECS`, and `Empty` at or beyond it
- [x] A genuinely dead stream is still detected — the TASK-VTT121 zero-frame
      watchdog keeps firing for a long hold that captures nothing
- [x] The tray resets to Ready after a fast tap instead of holding the error
      icon, and the capture stream is no longer re-opened on every quick tap
- [x] Regression test proven genuine: reverting the classification fails
      `classify_zero_samples_splits_on_hold_length` with "a hasty tap must not
      be reported as a dead microphone" (left Empty, right TooShort)
- [x] Boundary covered — at exactly `MIN_DURATION_SECS` a silent stream is
      still `Empty`
- [x] Full suite green — 127 passed, 0 failed
- [x] The stale comments asserting the old premise are corrected in the same
      change, on `CaptureClass::Empty`, `RecordingResult::NoAudioCaptured`,
      the `stop_recording` doc, and the inline `Empty` branch

## Notes

This is the second defect the 150ms guard was concealing. Both were latent from
the Rust rewrite; neither could fire while fast taps were being discarded
before they reached the audio layer.

## Dependencies

- Story: STORY-VTT015
- Directive: DIRECT-VTT002
