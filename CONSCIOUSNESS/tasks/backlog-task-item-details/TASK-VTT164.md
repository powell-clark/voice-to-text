# TASK-VTT164: accuracy-compare.sh --corpus override is neutralised by default-equality check

## Context

Discovered in TASK-VTT162: scripts/accuracy-compare.sh sets CORPUS_DEFAULT="$DATA_DIR/recordings" and only applies its archive-preference override when [ "$CORPUS" = "$CORPUS_DEFAULT" ] is true — so an operator who explicitly passes --corpus "$DATA_DIR/recordings" (the intended debug ring) gets silently redirected to the archive anyway, because the override value happens to equal the default value. The 'was --corpus explicitly passed' state needs its own flag rather than being inferred from string equality to the default.

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
