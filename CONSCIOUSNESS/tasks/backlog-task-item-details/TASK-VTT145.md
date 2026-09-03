# TASK-VTT145: Suppress steady background noise before inference

## Context

Operator p00 steering 2026-08-15: noise reduction from fans. No audio-domain noise handling exists today — audio.rs has only a peak-amplitude floor (MIN_AMPLITUDE 500) and main.rs strips Whisper's hallucinated [noise]/[BLANK_AUDIO] filler tokens after the fact. Steady fan hum defeats both: loud enough to pass the gate, and it degrades inference rather than producing a strippable token. Wanted: capture-path suppression (high-pass + noise-profile subtraction estimated from the leading pre-speech frames), toggleable in settings.conf, default on.

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
