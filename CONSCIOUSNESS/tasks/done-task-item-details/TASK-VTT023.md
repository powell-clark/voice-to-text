# TASK-VTT023: Batch file transcription via --file flag

## Acceptance Criteria
1. [x] `vtt-linux --file audio.wav` transcribes the file and prints the result
   to stdout. **Actual proof:** run against `tests/fixtures/testing-one-two-three.wav`
   with the resident large-v3-turbo model produced `Testing 1234. The quick brown
   fox jumps over the lazy dog.` on stdout, and `2>/dev/null` confirmed stdout is
   the transcript alone (batch mode skips `logging::init` + silences whisper.cpp
   chatter so pipes stay clean).
2. [deferred → TASK-VTT130] Accepts `.mp3`, `.m4a`, `.flac` via format
   auto-detection. WAV (16 kHz) ships now; other formats need a decoder +
   resampler stack (symphonia + rubato) — a dependency/architecture decision.
   The 16 kHz-required error already suggests `ffmpeg -ar 16000 -ac 1` as the
   interim path.
3. [deferred → TASK-VTT130] Long files (>5 min) processed in chunks with stderr
   progress. Whisper handles the whole buffer today; explicit chunking for very
   long inputs is a refinement.
4. [x] Exit code 0 on success, non-zero on decode failure. **Verified:** missing
   file → `Error: open …: No such file` + exit 1; successful run → exit 0.
   A non-16 kHz WAV is rejected with an actionable message (unit-tested).

## Implementation notes
- `--file` / `-f <PATH>` handled in `main()` before any GTK/singleton/tray setup
  (`run_file_mode`), reusing the tray app's `models::resolve_variant/find/ensure`
  + `WhisperEngine`. Headless — composes in shell pipelines.
- `whisper::decode_wav_to_samples` (pure, unit-tested: mono decode, stereo
  down-mix, non-16 kHz rejection) does the WAV → f32 conversion.
