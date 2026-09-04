# TASK-VTT163: Archive corpus is 48kHz; accuracy-compare.sh needs 16kHz

## Context

Discovered in TASK-VTT162: recordings under $DATA_DIR/archive are captured at 48kHz (TASK-VTT150's archiving feature), but scripts/accuracy-compare.sh's --file mode hard-requires 16kHz mono and fails every archive file identically on both settings sides, producing a false 'N/N identical' result instead of an error. Either --file needs to accept/resample 48kHz input, or the harness needs to detect and skip/resample archive files before comparing, or the archive-preference logic needs a sample-rate probe before trusting the corpus.

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
