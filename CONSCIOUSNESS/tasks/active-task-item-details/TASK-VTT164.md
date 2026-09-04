# TASK-VTT164: accuracy-compare.sh --corpus override is neutralised by default-equality check

## Context

Discovered in TASK-VTT162: scripts/accuracy-compare.sh sets CORPUS_DEFAULT="$DATA_DIR/recordings" and only applies its archive-preference override when [ "$CORPUS" = "$CORPUS_DEFAULT" ] is true — so an operator who explicitly passes --corpus "$DATA_DIR/recordings" (the intended debug ring) gets silently redirected to the archive anyway, because the override value happens to equal the default value. The 'was --corpus explicitly passed' state needs its own flag rather than being inferred from string equality to the default.

## Acceptance criteria

- [ ] `--corpus` is tracked via an explicit "was this flag passed" boolean, not inferred from string-equality to `CORPUS_DEFAULT`
- [ ] `--corpus "$DATA_DIR/recordings"` (the debug ring, which happens to equal the default) is honoured — the archive-preference override does NOT silently redirect it
- [ ] `--corpus` pointing at any other directory (e.g. a custom test corpus) is still honoured, unaffected by this fix
- [ ] No `--corpus` flag at all still triggers the existing archive-preference logic exactly as before (the fix must not change the no-override behaviour)
- [ ] Falsification: re-run the harness with `--corpus "$DATA_DIR/recordings"` while the archive holds ≥N files and confirm it actually reads from `recordings/`, not `archive/` (this is the exact bug TASK-VTT162 hit)

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT018

## Pre-mortem

### Failure modes

- A naive fix (e.g. comparing against a sentinel empty string) could break if a future caller legitimately wants to pass an empty `--corpus` value — using a real boolean flag set only inside the arg-parsing loop avoids this ambiguity entirely
- Must not change corpus selection for the common case (no `--corpus` passed) since that's the harness's normal, already-working path (archive preferred when populated)

### Weak assumptions

- Assumes bash's arg-parsing `while` loop is the right place to set the flag (it already parses `--corpus` there) rather than adding a second pass
- Assumes no other script or caller depends on the current (buggy) string-equality behaviour — grepped: only accuracy-compare.sh's own archive-preference block reads `$CORPUS_DEFAULT`
