# TASK-VTT172: Extend the platform-spec generator and retire docs/PLATFORM-PARITY.md

## Context

Split from TASK-VTT166 (2026-09-05) — see that card for the card-migration
work already done. `scripts/generate-platform-spec.sh` aggregates each
maintained feature card's own "Cross-platform acceptance criteria" section
cleanly, but `docs/PLATFORM-PARITY.md` also carries two views the generator
has no equivalent for:

- **§0, platform path conventions** — a small table of Linux/Windows path
  mappings (settings, model cache, logs, recordings) that isn't tied to any
  single feature card.
- **The consolidated "Parity gap register"** — a priority-sorted table of
  every open/closed gap with its task link, read at a glance rather than
  scattered across 12 cards' individual bullet lists.

Retiring the hand-maintained doc before the generator can reproduce (or an
operator explicitly accepts losing) these two views would be a silent
regression in how this project tracks parity.

## Acceptance criteria

- [ ] _(to be scoped — likely either: (a) extend the generator script to
      also emit a synthesised path-conventions section and a gap-register
      table built from every `- [ ]` bullet across the 12 migrated cards, or
      (b) an explicit operator decision that these two views are acceptable
      to drop, recorded here before retirement)_

## Dependencies

- Directive: DIRECT-VTT005
- Story: STORY-VTT018
- Doc: `docs/PLATFORM-PARITY.md`, `scripts/generate-platform-spec.sh`
