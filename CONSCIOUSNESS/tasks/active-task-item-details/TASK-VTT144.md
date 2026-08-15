# TASK-VTT141: Correction dictionary editable from the tray

## Context

Operator p00 steering 2026-08-15: common mispronounced phrases. corrections.rs + settings.rs shipped (TASK-VTT118) but src/tray/ has zero correction handling — grep 'correction' src/tray/ returns nothing — so adding a misheard->correct pair requires hand-editing settings.conf. Add a tray editor beside the existing initial_prompt textarea.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT019
- Features: FEAT-VTT037

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
