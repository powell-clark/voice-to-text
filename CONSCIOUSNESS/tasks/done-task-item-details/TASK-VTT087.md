# TASK-VTT087: Windows automated test suite — E2E transcription + expanded unit tests

## Context

Requested by Emmanuel (2026-06-25): "full e2e tests and unit tests". Serves
STORY-VTT018 (automated regression tests and CI gates). The repo already has solid
inline unit tests in settings.rs, models.rs, hotkey/portable.rs, logging.rs,
audio.rs, main.rs, tray/linux.rs. This task fills the gaps and adds a genuine
end-to-end transcription test that needs no microphone.

## Approach

**E2E (no mic, fully automated):** a Windows SAPI voice generates a known phrase to
a 16 kHz mono 16-bit WAV fixture (`tests/fixtures/testing-one-two-three.wav`,
committed). The test decodes the WAV via `hound`, feeds the f32 PCM to a real
`WhisperEngine` loaded with `tiny.en`, and asserts the transcription contains the
expected words ("testing", "one", "two", "three"). This exercises the true
audio->text path through whisper-rs / whisper.cpp on Windows. Gated behind
`#[ignore]` (run with `cargo test -- --ignored`) so the default `cargo test` stays
fast and offline; the ignored test downloads tiny.en (~75 MB) on first run.

**Unit tests to add:**
- transcribe.rs: empty/whitespace transcription -> None
- models.rs: portable-tray label/catalogue consistency guard (supports TASK-VTT086)
- typing.rs: newline-segment splitting helper (extract pure fn)
- hotkey: default keycode 0 maps to Scroll Lock (regression lock for the
  Scroll-Lock activation-key requirement)

## Acceptance criteria

- [ ] `cargo test` (default) passes on Windows with new unit tests, no network
- [ ] `cargo test -- --ignored` runs the E2E and asserts whisper transcribes the
      SAPI fixture to the expected words on Windows
- [ ] Scroll Lock default-activation-key regression test present and green
- [ ] WAV fixture committed; generator script documented
- [ ] CI invocation documented (default fast suite in CI; E2E opt-in)

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
- Feature: FEAT-VTT035
- Relates-to: TASK-VTT086 (tray menu guard test)
