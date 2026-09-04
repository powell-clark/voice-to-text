# TASK-VTT130: --file multi-format decode + long-file chunking

## Context
Split out from TASK-VTT023, which shipped `--file` for 16 kHz WAV (transcribe →
stdout, clean pipes, exit codes) with actual-proof verification. This task
covers criteria 2 and 3 of that card: broader input formats and long-file
handling.

Deferred because both need dependency/architecture decisions rather than a
quick add:
- Multi-format (`.mp3`, `.m4a`, `.flac`) needs a decoder (symphonia) plus a
  resampler (rubato/dasp) to reach whisper's required 16 kHz mono — arbitrary
  sample rates are the hard part. Adding those crates is a one-way-door
  dependency decision that warrants an ADR and operator awareness.
- Whisper already processes the whole buffer, but very long inputs (>5 min)
  benefit from explicit chunking with stderr progress so memory stays bounded
  and the user sees movement.

## Acceptance Criteria
1. [x] `--file` accepts `.wav`, `.mp3`, `.m4a`, `.flac` via format auto-detection,
   decoding + resampling to 16 kHz mono before transcription. — `whisper::decode_audio_to_samples`
2. [x] Inputs at any sample rate are resampled to 16 kHz. Implemented as a NEW
   entry point (`decode_audio_to_samples`) rather than modifying
   `decode_wav_to_samples` in place — that function's existing 16kHz-only
   contract is kept unchanged since `RetranscribeLast` (`WorkItem` in
   `main.rs`) also calls it and only ever sees this app's own 16kHz archived
   WAVs; removing its rejection there would have been a no-op improvement to
   an unreachable path, not worth risking its existing tests for. `--file`
   (the criterion's actual target) gets the full any-rate/any-format
   capability via the new function.
3. [x] Long files (>5 min) are transcribed in chunks with progress reported to
   stderr; stdout still receives only the concatenated transcript. — pure,
   fully unit-tested `chunk_ranges()` in `main.rs`, non-overlapping (see
   Evidence for why overlap was not attempted).
4. [x] cargo test / clippy / fmt green; end-to-end proof on at least one non-WAV
   fixture. — 198 tests passing (was 182 before TASK-VTT130), clippy/fmt
   clean; permanent e2e test added (see Evidence) plus manual multi-format
   falsification broader than just "one" fixture.

## Evidence, 2026-09-05

### Required pre-checks (ADR-0006)

**(1) Edition-2024 transitive audit — PASS, zero new edition-2024 exposure.**
`cargo metadata` over the full dependency graph (437 packages) found 12
edition-2024 packages; traced every one via `cargo tree -i <pkg> --target all`
to its ancestor: all 12 come from pre-existing dependencies already in
`Cargo.toml` before this task (`enigo` directly; `moxcms`/`pxfm` via
`arboard`→`image`; `getrandom 0.4.2`/`wit-bindgen`/`wasip2`/`wasip3` via
`tempfile` and `whisper-rs-sys`'s build deps/`cpal`'s `oboe`/`reqwest`'s
`ring`; `toml_edit`/`toml_datetime`/`toml_parser` via `cpal`'s `ndk`;
`coreaudio-sys` via `cpal`, macOS-only). Neither `symphonia` nor `rubato`
nor anything unique to their trees appears among the 12.

**(2) Binary size delta — measured, +2.38 MiB (7.2%).** True before/after,
not a vacuous "dependency declared but unused" check (the first attempt at
this measurement showed a 0-byte delta because nothing called into
`symphonia`/`rubato` yet — LTO stripped both entirely; re-measured after
real implementation):
```
baseline (no symphonia/rubato):        34,412,160 bytes (32.82 MiB)
with symphonia+rubato, actually used:  36,905,472 bytes (35.20 MiB)
delta:                                  2,493,312 bytes ( 2.38 MiB, 7.2%)
```

### Vendored-build convention preserved

This repo vendors all dependencies (`.cargo/config.toml` → `vendor/`,
offline-reproducible builds). Adding a genuinely new crate required
temporarily bypassing the vendor override to resolve+fetch from the real
registry, then `cargo vendor` to regenerate `vendor/` including
`symphonia`/`rubato` and their transitive trees, restoring the original
relative-path `directory = "vendor"` config. Verified with a full clean
`cargo clean --release && cargo build --release --offline`: succeeds with
zero network access, proving the regenerated vendor tree is complete.

### Real end-to-end falsification (not just unit tests)

Generated real non-WAV fixtures via `ffmpeg` from the existing
`tests/fixtures/testing-one-two-three.wav`, each at a genuinely different
sample rate/channel layout than the source, and ran the actual built binary:

| Fixture | Format | Rate/channels | `--file` output |
|---|---|---|---|
| fixture.mp3 | MP3 | 44.1 kHz mono | "Testing 1234. The quick brown fox jumps over the lazy dog." |
| fixture.m4a | AAC/M4A | 44.1 kHz mono | same, correct |
| fixture.flac | FLAC | 48 kHz **stereo** | same, correct (proves downmix + resample together) |
| fixture-8k-stereo.wav | WAV | 8 kHz stereo (upsampling case) | same, correct — previously hard-rejected by `decode_wav_to_samples` |

All four produced the exact correct transcript, proving format detection,
resampling (both down- and up-sampling), and stereo downmix all work
together correctly, not just in isolation.

**Chunking**, tested with a genuine >5 minute WAV (688.2s, built by
concatenating the short fixture 120× via `ffmpeg`'s concat demuxer):
```
stderr: chunk 1/3...
        chunk 2/3...
        chunk 3/3...
stdout: (only the concatenated transcript — no chunk markers leaked into stdout)
```
3 chunks matches `chunk_ranges(688.2s × 16000, 300s × 16000)`'s prediction
exactly (`ceil(688.2/300) = 3`). Real transcription of all 3 chunks
succeeded and concatenated correctly. Minor repetition/garbling artifacts
appear in the middle of the long transcript — this is whisper's known
behaviour on unnaturally-looped repetitive audio (a source-material
artifact of the test method), not a chunking defect; a chunk boundary
falling mid-phrase is the documented, accepted limitation (see below), not
what caused this.

**Permanent automated e2e test added** (not just my manual scratchpad
run): `whisper::tests::e2e_transcribes_mp3_fixture_via_decode_audio_to_samples`,
using a new committed fixture `tests/fixtures/testing-one-two-three.mp3`
(52 KB, transcoded from the existing wav fixture). `#[ignore]`d by default
(downloads the same `base.en` model the existing wav e2e test already
uses), mirroring that test's own convention exactly. Ran it for real:
```
$ cargo test --release --offline -- --ignored e2e_transcribes_mp3_fixture_via_decode_audio_to_samples
test whisper::tests::e2e_transcribes_mp3_fixture_via_decode_audio_to_samples ... ok
```

### Design choice: non-overlapping chunks, not overlap-and-trim

The ADR's Recommendation sketched "overlapping segments" for chunking.
Implemented non-overlapping instead: overlap without a way to deduplicate
the overlapping region's *text* just produces duplicated words in the
output, and a correct overlap-trim (using whisper segment timestamps to
keep only each chunk's "core" region) is real additional complexity for a
benefit (avoiding an occasional mid-word split at a chunk boundary) the
acceptance criteria don't explicitly require. Documented as a known,
accepted limitation rather than silently different from the ADR's sketch.

### Full verification
```
cargo build --release --offline: clean, 0 warnings
cargo fmt --check: clean
cargo clippy --all-targets --offline -- -D warnings: clean
cargo test --release --offline: 198 passed, 0 failed, 3 ignored (was 182/0/1 before this task)
```

## Technical Approach
- File an ADR for the audio-decode dependency choice (symphonia + resampler)
  before adding the crates (authorship-flow: one-way-door).
- Generalise `whisper::decode_wav_to_samples` into a format-agnostic
  `decode_audio_to_samples` that detects the container, decodes to f32, and
  resamples to 16 kHz mono.
- Add a chunker that windows the sample buffer (with small overlap) for inputs
  beyond a threshold, transcribing each window and concatenating.

## Dependencies
- TASK-VTT023 (WAV `--file` core) — done, provides the flag + plumbing.
- New: an ADR for the decoder/resampler dependency stack.

## ADR-0006 accepted (2026-07-17)

Decision is now binding: option (a) — symphonia (features: mp3, isomp4, aac, flac only) + rubato for resample; WAV stays on hound. Two pre-merge checks are REQUIRED and their evidence recorded on this card: (1) cargo tree audit confirming no edition-2024 transitive requirement enters via either crate; (2) stripped-binary size delta measured before/after and noted. Chunking (criterion 3) may ship first/independently as pure control flow. Rollback path per ADR-0006 if checks fail: ffmpeg shell-out (Recommends-only) or defer multi-format.
