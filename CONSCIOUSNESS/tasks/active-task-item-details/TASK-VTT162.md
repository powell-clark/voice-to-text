# TASK-VTT162: Measure large-v3 against large-v3-turbo with the accuracy harness

## Context

Operator 2026-09-04: 'Can we get a model bump as well?' and 'accuracy seems down'. Facts: whisper-rs 0.16.0 is the latest crate (crates.io, updated 2026-03-12), so no engine bump exists; ggerganov/whisper.cpp on HuggingFace ships large-v1/v2/v3, large-v3-turbo, and q5_0/q8_0 quantised variants — no large-v4. large-v3 (2963 MB) is already in the src/models.rs catalogue and selectable from the tray. TASK-VTT158 measured turbo vs small.en (turbo better 6 of 7) and left large-v3 untested. Run scripts/accuracy-compare.sh over the archive corpus with large-v3 as the variant; also try ggml-large-v3-turbo-q8_0 (needs a catalogue entry + sha256) as the speed candidate. Ship a default change only if the harness says so.

## Acceptance criteria

- [ ] `ggml-large-v3.bin` downloaded via the app's normal `models::ensure()` path (already cataloged with sha256 in `src/models.rs`) — no code change needed to fetch it
- [ ] `scripts/accuracy-compare.sh --settings-a <large-v3-turbo conf> --settings-b <large-v3 conf>` run over the same binary against the debug-ring corpus (archive is empty; ring holds 20, harness takes the most recent 10)
- [ ] Every DIFF line read and judged (which side is closer to what was actually said), not just counted
- [ ] Result recorded on this card (moved to done) with the per-recording diffs and an explicit RECOMMENDATION: keep `large-v3-turbo` or switch default to `large-v3`, following the TASK-VTT158 precedent — no default change ships unless the harness supports it
- [ ] `cargo test --workspace` passes if any code changed (expected: none — this is a measurement task, not an implementation task)

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT018

## Pre-mortem

### Failure modes

- Both configs read the operator's real `settings.conf` instead of the harness's temp copy (the exact silent-failure TASK-VTT158 found and fixed) — mitigated by re-running the French-canary falsification check from VTT158 first if any doubt exists
- large-v3 is 2963 MB; a truncated or interrupted download could leave a corrupt file, but `ensure()`'s sha256 check should catch and refuse it — the failure mode is "wastes bandwidth and retries", not "runs inference on a corrupt model"
- Only 10 recordings is a small sample; a result here is directional evidence for this operator's voice and vocabulary, not a general benchmark claim

### Weak assumptions

- Assumes the current debug-ring recordings (rotates, most recent 10) are representative of normal dictation and not dominated by one unusual session
- Assumes large-v3's extra ~1.4 GB and slower inference is worth measuring at all given large-v3-turbo already tested well in TASK-VTT158 — the operator explicitly asked for this comparison ("Can we get a model bump as well?"), so the assumption is operator-directed, not invented
- Assumes no code changes are needed — if the harness or `ensure()` misbehaves on this larger model, scope may need to expand; not expected based on the catalogue already containing large-v3
