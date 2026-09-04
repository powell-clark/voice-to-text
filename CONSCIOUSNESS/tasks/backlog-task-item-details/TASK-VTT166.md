# TASK-VTT166: Migrate remaining parity cards, retire docs/PLATFORM-PARITY.md

## Context

Filed alongside ADR-0008 and TASK-VTT080. docs/PLATFORM-PARITY.md aggregates 19 maintained feature cards (VTT001,004,005,006,008,010,011,012,013,014,015,016,017,022,023,026,027,028,035) with a detailed gap register and per-gap task links -- considerably richer than the 5-card 'Cross-platform acceptance criteria' convention ADR-0008 formalised. Migrate the remaining 14 cards to that same section+last_tested shape, verify scripts/generate-platform-spec.sh's output preserves the gap-register detail (or extend the generator to carry it), then retire the hand-maintained doc only once the generated view is confirmed not to have silently dropped anything.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT005
- Story: STORY-VTT018

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
