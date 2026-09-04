# TASK-VTT162: Measure large-v3 against large-v3-turbo with the accuracy harness

## Context

Operator 2026-09-04: 'Can we get a model bump as well?' and 'accuracy seems down'. Facts: whisper-rs 0.16.0 is the latest crate (crates.io, updated 2026-03-12), so no engine bump exists; ggerganov/whisper.cpp on HuggingFace ships large-v1/v2/v3, large-v3-turbo, and q5_0/q8_0 quantised variants — no large-v4. large-v3 (2963 MB) is already in the src/models.rs catalogue and selectable from the tray. TASK-VTT158 measured turbo vs small.en (turbo better 6 of 7) and left large-v3 untested. Run scripts/accuracy-compare.sh over the archive corpus with large-v3 as the variant; also try ggml-large-v3-turbo-q8_0 (needs a catalogue entry + sha256) as the speed candidate. Ship a default change only if the harness says so.

## Acceptance criteria

- [x] `ggml-large-v3.bin` downloaded via the app's normal `models::ensure()` path (already cataloged with sha256 in `src/models.rs`) — no code change needed to fetch it
- [x] `scripts/accuracy-compare.sh --settings-a <large-v3-turbo conf> --settings-b <large-v3 conf>` run over the same binary against the debug-ring corpus (archive is empty; ring holds 20, harness takes the most recent 10)
- [x] Every DIFF line read and judged (which side is closer to what was actually said), not just counted
- [x] Result recorded on this card (moved to done) with the per-recording diffs and an explicit RECOMMENDATION: keep `large-v3-turbo` or switch default to `large-v3`, following the TASK-VTT158 precedent — no default change ships unless the harness supports it
- [x] `cargo test --workspace` passes if any code changed (no code changed — measurement only)

## Two harness defects found while running this, both silent

**The archive corpus is 48 kHz; `--file` batch mode hard-requires 16 kHz.** By the time
this task ran, the archive had accumulated real recordings (TASK-VTT150's archiving
feature), so `accuracy-compare.sh`'s archive-preference logic kicked in for the first
time in practice. Every archive file failed identically on both settings sides with
`Error: ... is 48000 Hz — --file needs 16 kHz mono audio`, and since both sides produced
the same (empty) stdout, the harness reported a false `10/10 identical` — the exact
silent-null-result shape TASK-VTT158's own postmortem warned about. Filed as
TASK-VTT163.

**`--corpus` cannot override the archive preference when the override equals the
script's own default value.** `CORPUS_DEFAULT="$DATA_DIR/recordings"`, and the
preference check is `[ "$CORPUS" = "$CORPUS_DEFAULT" ]` — so passing
`--corpus "$DATA_DIR/recordings"` explicitly is indistinguishable from not passing
`--corpus` at all, and archive wins anyway. Worked around by pointing `--corpus` at a
copy of the same files under a differently-named directory. Filed as TASK-VTT164.

Separately (not a harness bug, an environment condition): this host was under heavy
memory pressure for most of the run (many concurrent Claude/VS Code/Chrome processes),
and the harness's `timeout 300`-per-call default was too tight under that load —
7 of the first 10 large-v3 calls timed out and returned empty, which would have been a
third silent false-"SAME" if taken at face value. Re-ran with a longer per-call timeout
and a resumable, append-only runner (`grep`-checked per file, only re-running files
whose prior result was empty) rather than trust the batch script's truncate-on-restart
output file.

## Result, 2026-09-04

10 recordings, same binary, `model=large-v3-turbo` vs `model=large-v3`, otherwise
identical settings:

| # | File | Verdict | Note |
|---|------|---------|------|
| 1 | vtt_recording_pwqJxW.wav | SAME | — |
| 2 | vtt_recording_QGGmNy.wav | SAME | — |
| 3 | vtt_recording_TWZG2Z.wav | DIFF | turbo: "...noise/error **there**." / large-v3: "...noise / error **then**" — turbo reads closer to what was meant |
| 4 | vtt_recording_Ce82Q1.wav | DIFF | turbo: "...right now. **Like 30,**" / large-v3 drops that trailing fragment entirely — unclear which is closer without the audio |
| 5 | vtt_recording_DA4es0.wav | SAME | — |
| 6 | vtt_recording_GcAtb6.wav | SAME | — |
| 7 | vtt_recording_ICqL9R.wav | DIFF | turbo: "vs code" / large-v3: "**VS** code" — large-v3 correct |
| 8 | vtt_recording_j3kZsb.wav | DIFF | turbo: "I speak **phonetic**." / large-v3: "I speak **phonetics**." — large-v3 correct (real word) |
| 9 | vtt_recording_la5ka0.wav | SAME | — |
| 10 | vtt_recording_lFErtZ.wav | DIFF | turbo: "**post-hog** projects... post-hog is alive" / large-v3: "**post-hoc** projects... post-hoc is alive" — turbo correct, large-v3 wrong twice in one sentence |

4/10 changed, 6/10 identical. large-v3 wins two minor points (VS Code capitalisation,
"phonetics" grammar). Turbo wins one minor point ("there" vs "then") and one that
matters: the operator's own product name, **PostHog**, comes out right on turbo
(`post-hog`) and wrong on large-v3 (`post-hoc` — a real but unrelated word). One diff
(Ce82Q1) is unresolvable from text alone without the source audio.

This is exactly the failure mode TASK-VTT158 flagged for `small.en` — "project names
and acronyms are exactly where the small model collapses" — except here the *larger*
model is the one that collapses on it. Bigger is not a proxy for closer to this
operator's actual vocabulary.

**RECOMMENDATION: stay on `large-v3-turbo`.** large-v3 does not clearly outperform
turbo on this operator's real dictation, gets his own most-used product name wrong
where turbo gets it right, and costs ~2x the disk (2963 MB vs 1536 MB) and
meaningfully slower per-call inference (contributed to repeated timeouts under host
load during this measurement). No default change ships.

`ggml-large-v3-turbo-q8_0` (the quantised speed candidate mentioned in the original
report) was not tried — out of scope for this pass since the harness result doesn't
support moving off turbo in the first place; a q8_0 turbo variant would need its own
catalogue entry + sha256 and is only worth doing if a *speed* problem with turbo
itself is reported.

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
