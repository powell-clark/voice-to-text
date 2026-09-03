# TASK-VTT159: Guard the high-pass against degenerate sample rates

## Context

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
