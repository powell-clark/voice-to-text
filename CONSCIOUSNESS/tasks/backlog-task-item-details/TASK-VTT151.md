# TASK-VTT151: Spectral subtraction if rumble filtering proves insufficient

## Context

Follow-up to TASK-VTT145, to be justified by measurement rather than assumed. Baseline: a 4th-order high-pass at 90 Hz removes the band holding 79.5 percent of measured noise energy. Spectral subtraction needs an FFT dependency and introduces musical noise, which Whisper tolerates far worse than steady rumble. Only worth building if transcription accuracy on real recordings is still limited by noise after the high-pass.

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
