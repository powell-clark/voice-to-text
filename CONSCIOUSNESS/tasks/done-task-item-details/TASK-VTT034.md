# TASK-VTT034: Build, deploy, verify end-to-end

## Context
After all code changes (TASK-VTT025 through TASK-VTT033) and cleanup (TASK-VTT031, TASK-VTT032), the branch `whisper-rs-migration` must produce a working binary that the user can use for daily dictation. This task compiles the release binary, installs it over the running copy, and verifies transcription quality and latency against expectations.

## Acceptance Criteria
1. `cargo build --release` completes successfully on branch `whisper-rs-migration` — warnings are acceptable, errors are not
2. The resulting `target/release/vtt-linux` links against `libvulkan.so.1` (verifiable with `ldd`)
3. The existing running VTT process is killed cleanly and the new binary is installed at `/usr/bin/vtt-linux` (via `sudo cp`)
4. VTT starts from the desktop launcher or systemd service; the tray icon appears; the initial log shows `Loading model...` then `Ready` within 10 seconds on the user's RTX 2060 SUPER
5. Ten consecutive push-to-talk recordings of varying lengths (2 s, 5 s, 10 s, 15 s, 30 s, then five random short clips) all transcribe successfully with end-to-end latency under 1 second per press
6. Transcription quality for the user's British English with programming vocabulary (Claude, ADR, PGPS, WezTerm, GitHub) is measurably comparable to the CT2 baseline — sample five recordings from `~/.local/share/voice-to-text/recordings/` transcribed by both the old binary (via manual `python3 transcribe.py`) and the new binary (by replaying through VTT), diff outputs
7. Model switching from the tray (Small → Medium → Large-v3-turbo) succeeds without restart; status transitions cleanly; the first press with each new model succeeds
8. Language toggle (English → Multilingual → English) triggers model reload for `small` and `medium`, stays on the same model for `large-v3-turbo` and `large-v3`, and produces correct transcriptions
9. No Python process appears in `ps aux | grep python3` during or after ten consecutive transcriptions
10. `~/.local/share/voice-to-text/vtt-$(date +%Y-%m-%d).log` shows the new log lines: `Model loaded: X in Y.Ys`, `Transcribed in Y.Ys`, no `python3` or `transcribe.py` references

## Technical Approach
```
cargo build --release
pkill -f vtt-linux
sudo cp target/release/vtt-linux /usr/bin/vtt-linux
# Launch VTT from desktop
```

Use the user's existing recordings at `~/.local/share/voice-to-text/recordings/` as the quality benchmark. For five representative clips, run both:
- Current installed binary (CT2 via Python, already tested)
- New binary (whisper-rs, via VTT directly)

Diff the transcripts; flag any meaningful quality regression for discussion. Minor wording differences (e.g., "I'm" vs "I am") are expected between model variants.

## Test Strategy
Manual end-to-end. The user is the primary tester. If quality is acceptable and latency is sub-second, STORY-VTT010 is complete.

## Files
- No new files — this is validation of the changes made in prior tasks
- `CONSCIOUSNESS/sessions.json` records the test session outcome
