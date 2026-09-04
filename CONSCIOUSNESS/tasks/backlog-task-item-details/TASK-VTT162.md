# TASK-VTT162: Measure large-v3 against large-v3-turbo with the accuracy harness

## Context

Operator 2026-09-04: 'Can we get a model bump as well?' and 'accuracy seems down'. Facts: whisper-rs 0.16.0 is the latest crate (crates.io, updated 2026-03-12), so no engine bump exists; ggerganov/whisper.cpp on HuggingFace ships large-v1/v2/v3, large-v3-turbo, and q5_0/q8_0 quantised variants — no large-v4. large-v3 (2963 MB) is already in the src/models.rs catalogue and selectable from the tray. TASK-VTT158 measured turbo vs small.en (turbo better 6 of 7) and left large-v3 untested. Run scripts/accuracy-compare.sh over the archive corpus with large-v3 as the variant; also try ggml-large-v3-turbo-q8_0 (needs a catalogue entry + sha256) as the speed candidate. Ship a default change only if the harness says so.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT018

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
