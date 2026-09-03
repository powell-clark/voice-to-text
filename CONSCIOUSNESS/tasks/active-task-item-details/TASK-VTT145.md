# TASK-VTT145: Suppress steady background noise before inference

## Context

Operator p00 steering 2026-08-15: noise reduction from fans. No audio-domain
noise handling exists today — audio.rs has only a peak-amplitude floor
(MIN_AMPLITUDE 500) and main.rs strips Whisper's hallucinated [noise]/[BLANK_AUDIO]
filler tokens after the fact. Steady fan hum defeats both: loud enough to pass the
gate, and it degrades inference rather than producing a strippable token. Wanted:
capture-path suppression (high-pass + noise-profile subtraction estimated from the
leading pre-speech frames), toggleable in settings.conf, default on.

## Measured noise profile, 2026-09-03

Twelve of Emmanuel's own archived recordings, quietest 200 ms window of each taken
as the noise estimate, loudest as speech:

```
median noise floor:      -27.2 dBFS
median speech-to-noise:   17.5 dB

     band Hz   noise %  speech %
    0-  100      79.5       7.6
  100-  200      10.5      22.0
  200-  300       5.6      21.0
  300- 1000       3.0      46.9
 1000- 4000       0.9       2.4
 4000- 8000       0.4       0.0
```

Four fifths of the noise energy is below 100 Hz; less than a tenth of the speech
energy is. The interference is low-frequency rumble — desk and fan — not
broadband hiss.

## Scope decision: high-pass now, spectral subtraction only if measured

The card asks for a high-pass AND noise-profile subtraction. The measurement says
the first does nearly all the available work, and it argues against reaching for
the second by default:

- A first-order-corner high-pass at 90 Hz removes the band holding 79.5% of the
  noise while touching 7.6% of the speech, none of it in the 300-3400 Hz range
  that carries intelligibility.
- Spectral subtraction needs an FFT dependency and introduces musical noise —
  isolated time-frequency artefacts that sound like tones. Whisper is trained on
  noisy speech and is robust to steady rumble; it is markedly less robust to
  artefacts that resemble speech. Adding subtraction on the assumption it helps
  risks making transcription worse in exchange for audio that measures cleaner.

So this task ships the high-pass, measures transcription against real recordings,
and reports. Spectral subtraction is filed ahead as a follow-up to be justified by
that measurement, not assumed. If the measurement shows residual noise costing
accuracy, the follow-up is warranted and has a baseline to beat.

## The archive stays raw

The denoiser runs on the Whisper path only. Archived audio (FEAT-VTT040) is
training data for a voice clone, and baking a filter into a corpus is not
reversible — a future model would learn Emmanuel's voice as it sounds after a
high-pass. Filtering is a decision the corpus consumer makes later, from the
original, so `stop_recording` archives the unfiltered capture and denoises only
the samples bound for transcription.

## Acceptance criteria

- [ ] A pure, unit-testable biquad high-pass in its own module, correct at both
      the 16 kHz and 48 kHz paths — the coefficients derive from the sample rate
      rather than assuming one
- [ ] Sub-100 Hz content is attenuated by at least 12 dB while 300-3400 Hz passes
      within 1 dB, asserted on synthesised tones
- [ ] The filter is applied to the samples bound for Whisper and NOT to the
      archived audio
- [ ] `denoise` in settings.conf toggles it, default on; absent means on, and
      `denoise=0` reproduces today's behaviour exactly
- [ ] Measured on at least 10 of Emmanuel's real recordings: transcription is
      unchanged or improved, with the outputs recorded on this card. A regression
      on any recording blocks the default-on setting
- [ ] `cargo test --workspace` passes; clippy and fmt clean
- [ ] A follow-up task is filed for spectral subtraction, carrying this
      measurement as its baseline

## Test Strategy

The filter is pure arithmetic over a slice, so it tests directly: feed synthesised
sine tones at 50, 90, 300, 1000 and 3000 Hz, measure output RMS against input RMS,
and assert the attenuation profile. Cross-check the real-world effect by running
the before/after transcription harness over Emmanuel's archived recordings, which
is the same method that verified TASK-VTT150.

## Files

- Create: `src/denoise.rs`
- Modify: `src/audio.rs` (apply on the Whisper path in `stop_recording`),
  `src/settings.rs` (the `denoise` key), `src/main.rs` (module declaration),
  `CHANGELOG.md`

## Pre-mortem

### Failure modes

- The high-pass thins Emmanuel's voice because his fundamental sits partly below
  the corner. A male fundamental runs roughly 85-180 Hz, so a 90 Hz corner clips
  the bottom of it. Mitigation: the corner is at 90 rather than 120, the slope is
  gentle, and the transcription measurement is the gate — 7.6% of speech energy
  below 100 Hz is the budget being spent, and Whisper reads formants, not F0.
- The filter is applied to the archive as well and silently corrupts the
  voice-clone corpus. Mitigation: an explicit acceptance criterion, because this
  failure is invisible until a model trained on it sounds wrong.
- Biquad state carried across recordings leaks the tail of one into the next.
  Mitigation: the filter is constructed per recording, not held on `Audio`.

### Weak assumptions

- That the twelve measured recordings represent Emmanuel's usual environment.
  They are recent and from his normal desk, but a different room or a laptop fan
  would shift the profile. The setting exists so a bad fit is one line to
  disable.
- That 200 ms of the quietest window is noise rather than speech. At a median
  17.5 dB speech-to-noise the separation is clear, and the aggregate across
  twelve files washes out any single mis-picked window.

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT015
