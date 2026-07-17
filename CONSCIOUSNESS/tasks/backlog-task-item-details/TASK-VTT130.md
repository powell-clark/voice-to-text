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
1. `--file` accepts `.wav`, `.mp3`, `.m4a`, `.flac` via format auto-detection,
   decoding + resampling to 16 kHz mono before transcription.
2. Inputs at any sample rate are resampled to 16 kHz (removing the current
   "must be 16 kHz" restriction from `decode_wav_to_samples`).
3. Long files (>5 min) are transcribed in chunks with progress reported to
   stderr; stdout still receives only the concatenated transcript.
4. cargo test / clippy / fmt green; end-to-end proof on at least one non-WAV
   fixture.

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
