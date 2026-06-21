# TASK-VTT023: Batch file transcription via --file flag

## Acceptance Criteria
1. `vtt-linux --file audio.wav` transcribes the file and prints the result to stdout
2. Accepts `.wav`, `.mp3`, `.m4a`, `.flac` via format auto-detection
3. Long files (>5 min) are processed in chunks; progress is reported to stderr
4. Exit code 0 on success, non-zero on unsupported format or decode failure
