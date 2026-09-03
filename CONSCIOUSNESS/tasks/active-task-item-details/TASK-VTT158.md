# TASK-VTT158: Track transcription accuracy across models and settings

## Context

Operator reports 2026-09-03 that accuracy feels worse than it has been, and there is no way to check — nothing measures transcription quality over time. Two candidate causes found by inspection, neither verified: the initial_prompt contains correction-pair syntax (odd=ADR, odds=ADRs) which Whisper treats as decoder context rather than substitution, and zero correction pairs are configured despite corrections.rs existing since TASK-VTT118; separately the prompt is 214 of 240 chars of comma-jammed word list that reads nothing like speech. Wanted: a harness that re-transcribes a fixed set of the operator's own recordings across model and settings variants and reports where outputs differ, so a change to model, prompt or corrections is measured rather than felt. The archive from TASK-VTT150 pairs audio with transcripts permanently and is the corpus this reads.

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
