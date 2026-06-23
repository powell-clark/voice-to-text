---
id: FEAT-VTT006
status: maintained
kano: performance
---

# FEAT-VTT006: Multi-language support with auto-detection

## Description
Whisper supports 99 languages. VTT exposes language selection via `settings.conf`. When set to `auto`, Whisper detects the language from the audio. When set to a specific language code (e.g. `en`, `fr`, `ja`), Whisper constrains transcription to that language for faster and more accurate results.

## Acceptance Criteria
- [x] `language = auto` in `settings.conf` produces correct transcription for English, French, and Spanish recordings — verified subjectively
- [x] `language = en` produces English-only transcription even when foreign words are spoken — verified
- [x] The language setting is read from `settings.conf` at startup and passed to the WhisperEngine — verified in `src/settings.rs` and `src/whisper.rs`
- [x] Invalid language codes do not crash the process — they fall back to `auto` with a log warning — verify with `language = xyz`
- [x] No language UI is exposed in the tray (language is settings-only, not runtime-switchable without restart) — verified

## Linked Tasks
- TASK-VTT006

## Parent Story
- STORY-VTT001
