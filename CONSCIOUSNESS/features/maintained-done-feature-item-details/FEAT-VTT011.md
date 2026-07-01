---
id: FEAT-VTT011
status: maintained
kano: performance
---

# FEAT-VTT011: User-configurable initial_prompt passthrough to Whisper

## Description
The `initial_prompt` field in `settings.conf` is passed directly to the Whisper inference call as the `initial_prompt` parameter. This primes the model with domain-specific vocabulary, proper nouns, and expected formatting so that repeated terms (names, product names, technical jargon) are transcribed more accurately.

**Note:** The original feature description referenced "both backends" (Python CT2 + whisper.cpp). The Python backend was retired in ADR-0003 (v2.0.0). This feature now covers only the whisper-rs backend.

## Acceptance Criteria
- [x] **AC-1** — `initial_prompt` value from `settings.conf` is passed to the whisper-rs `full_params` `initial_prompt` field — verified in `src/whisper.rs`
- [x] **AC-2** — A prompt of `Emmanuel Powell-Clark` causes "Emmanuel Powell-Clark" to be transcribed correctly when spoken — verified in daily use
- [x] **AC-3** — Setting `initial_prompt` to empty string is a no-op (no Whisper errors) — verified
- [x] **AC-4** — The prompt is not prepended to the transcription output — it is only passed as inference context — verify by checking output

## Linked Tasks
- TASK-VTT009

## Parent Story
- STORY-VTT003
