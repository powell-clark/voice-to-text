# TASK-VTT163: Archive corpus is 48kHz; accuracy-compare.sh needs 16kHz

## Context

Discovered in TASK-VTT162: recordings under $DATA_DIR/archive are captured at 48kHz (TASK-VTT150's archiving feature), but scripts/accuracy-compare.sh's --file mode hard-requires 16kHz mono and fails every archive file identically on both settings sides, producing a false 'N/N identical' result instead of an error. Either --file needs to accept/resample 48kHz input, or the harness needs to detect and skip/resample archive files before comparing, or the archive-preference logic needs a sample-rate probe before trusting the corpus.

## Acceptance criteria

- [x] `scripts/accuracy-compare.sh` probes each candidate WAV's sample rate before transcribing it
- [x] Any file that isn't 16 kHz mono is resampled to a 16 kHz mono temp copy (via `ffmpeg`, already the app's own suggested fix in its `--file` error message) before being passed to the binary, so archive material (48 kHz, TASK-VTT150) works transparently — the archive itself stays 48 kHz (training-grade audio is the intended design, not a bug; TASK-VTT150) and is not touched
- [x] Resampling happens once per source file per run and the temp copy is deleted after use — no leftover files in the corpus dir or elsewhere
- [x] Falsification: re-run the harness against the archive corpus (now populated, per TASK-VTT162) and confirm it produces real DIFF/SAME verdicts from actual transcriptions, not the false all-"identical" TASK-VTT162 hit
- [x] Missing `ffmpeg` fails loudly with a clear message naming the file and required tool — never silently skips or falls back to a false result

## Evidence, 2026-09-04

Directly verified the resample path on a real 48 kHz archive recording:
```
source: .../archive/2026-09-04/vtt_20260904T220754_414.wav
ffprobe (source):     48000
ffprobe (resampled):  16000
--file output: "Are you requesting me to run those lines? I'm happy to do
anything. I'd like a president's update."
```
Non-empty, real transcription — previously this file hard-failed with
`Error: ... is 48000 Hz`. Then ran the harness itself end-to-end against the
now-preferred archive corpus (113 recordings):
```
$ scripts/accuracy-compare.sh --baseline ./target/release/vtt-linux --candidate ./target/release/vtt-linux -n 2
corpus: archive (113 recordings, paired with transcripts)
[1] SAME  vtt_20260904T220754_414.wav
[2] SAME  vtt_20260904T220717_134.wav
=== 2/2 identical, 0 changed ===
```
SAME is the correct control result here (baseline and candidate are the
identical binary with no settings override) — the point is that both sides
now produce real transcribed text via the resample path rather than
identically-empty output from a decode error. `bash -n` syntax check clean;
no shellcheck in this repo's toolchain or CI to run. No `.rs` files changed,
so `cargo test`/clippy are not applicable.

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT018

## Pre-mortem

### Failure modes

- `ffmpeg` may not be installed on every machine in the multi-machine workflow (macOS/Windows/Linux per CLAUDE.md) — must fail loud naming the missing tool, not silently degrade to another false-identical result
- Resampling changes the audio slightly versus a native 16 kHz capture, but both settings-A and settings-B get the identical resampled input, so the A/B comparison stays apples-to-apples even though it's not bit-identical to a real 16 kHz mic capture
- The archive's sidecar JSON (ground-truth transcript pairing) must not be disturbed by writing resampled temp copies into the archive directory itself — resample into a scratch/temp location, never in-place

### Weak assumptions

- Assumes `ffmpeg` is an acceptable new dev-only script dependency (not shipped to end users, not linked into the app itself) — reasonable since the app's own `--file` error message already recommends it as the fix
- Assumes probing the WAV header's frame rate (not deep content validation) is sufficient to decide whether resampling is needed
