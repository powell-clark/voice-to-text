---
id: FEAT-VTT010
status: maintained
kano: delighter
---

# FEAT-VTT010: Configurable hotkey, voice prefix, and initial prompt

## Description
Three key behaviours are configurable via `~/.config/voice-to-text/settings.conf`:
- `hotkey`: the push-to-talk key (default: F4)
- `voice_prefix`: text prepended to every transcription (e.g. a wake word or context)
- `initial_prompt`: seed text passed to Whisper to prime it with domain vocabulary

Settings are read at startup. Changing them requires a service restart.

## Acceptance Criteria
- [x] **AC-1** — `hotkey = XF86AudioMicMute` reassigns the trigger key; F4 no longer activates VTT — verified by testing with alt hotkeys
- [x] **AC-2** — `voice_prefix = Note:` prepends "Note: " to every transcription output — verify by setting and dictating
- [x] **AC-3** — `initial_prompt = Emmanuel Powell-Clark Powell-Clark` primes Whisper to recognise those names — verified in TASK-VTT009
- [x] **AC-4** — Settings file is created with sane defaults on first run if it does not exist — verify on fresh install
- [x] **AC-5** — Malformed settings file produces a log warning and falls back to defaults; does not crash — verify by corrupting settings.conf
- [x] **AC-6** — All three settings are documented in the README — verify in README.md

## Linked Tasks
- TASK-VTT009

## Parent Story
- STORY-VTT001
