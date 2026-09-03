# TASK-VTT159: Guard the high-pass against degenerate sample rates

## Context

<<<<<<< HEAD
denoise.rs computes biquad coefficients as 2*PI*cutoff/sample_rate without guarding the rate. At sample_rate 0 that is infinity, sin_cos of infinity is NaN, the coefficients are NaN and every output sample is NaN — Whisper then receives NaN audio rather than a clean error. At any rate where the 90 Hz corner sits at or above Nyquist the filter has gain above 1 and amplifies instead of attenuating; measured b0=1.98 at fs=100. capture_rate is read from the device on the native-format fallback path and nothing guarantees it is sane. The existing tests missed both because they only ever pass 16 kHz and 48 kHz.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT015

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
=======
`denoise.rs` computed biquad coefficients as `2*PI*cutoff/sample_rate` with no
guard on the rate. Two failures follow, both silent:

- **Rate 0.** The division is infinity, `sin_cos` of infinity is NaN, the
  coefficients are NaN, and every output sample becomes NaN. Whisper then
  receives NaN audio rather than a clean error.
- **Corner at or above Nyquist.** The coefficients come out with gain above 1
  and the filter amplifies rather than attenuates. Measured `b0 = 1.98` at
  `fs = 100`, where the 90 Hz corner sits above the 50 Hz Nyquist.

Neither arises from the direct capture path, which requests 48 kHz. But
`capture_rate` is read from the device on the native-format fallback
(`native.sample_rate().0`) and nothing guarantees that value is sane.

The existing eight tests missed both because every one of them passes 16 kHz or
48 kHz. A test suite that only exercises the values you expect cannot find the
values you do not.

## Acceptance criteria

- [x] A filter whose corner cannot be realised at the given rate passes audio
      through unchanged rather than producing NaN or amplifying
- [x] Rate 0 is handled, and the test asserts the output is finite AND equal to
      the input — not merely finite
- [x] A corner at or above Nyquist is handled, tested at 100 Hz and 180 Hz
- [x] The lowest realisable rate still filters, so the guard has not quietly
      disabled the feature at ordinary rates
- [x] `cargo test --workspace` passes; clippy and fmt clean

## Evidence

The defect, reproduced from the coefficient maths before any fix:

```
fs=16000: b0=0.9681 a1=-1.9356     <- correct
fs=48000: b0=0.9892 a1=-1.9783     <- correct
fs=0:     NaN (sin/cos of inf)     <- every sample becomes NaN
fs=100:   b0=1.9794 a1=-3.5409     <- gain above 1, amplifies
fs=180:   b0=0.0000 a1=2.0000      <- degenerate at exactly Nyquist
```

After the guard, 176 tests pass including three new ones:
`a_zero_sample_rate_passes_audio_through_rather_than_nan`,
`a_corner_above_nyquist_passes_through_rather_than_amplifying`, and
`the_lowest_realisable_rate_still_filters`.

The third exists because a guard that is too eager is the obvious way to fix
this badly — disabling the filter at every rate would pass the first two tests
and silently remove the feature. 8 kHz is just above twice the corner and must
still filter.

## Why passthrough rather than an error

`suppress_rumble` sits on the transcription path. Returning an error would mean
either dropping a recording the operator has already spoken, or adding a failure
branch to a path that currently cannot fail. Unfiltered audio transcribes fine —
it is what shipped before TASK-VTT145 and what ships today by default. Passing
through is the degradation that costs nothing.

## Dependencies

- Story: STORY-VTT015
- Directive: DIRECT-VTT002
>>>>>>> ed3288d (fix(denoise): pass audio through when the filter cannot be built TASK-VTT159)
