# TASK-VTT053: Implement CT2 transcription daemon

## Acceptance Criteria
1. [x] `transcribe_daemon.py` starts, loads a faster-whisper model, and responds to the protocol from TASK-VTT052 — `ct2-daemon/transcribe_daemon.py`
2. [x] Daemon responds to a transcribe request within 500ms of receiving audio (model already loaded) — **met for typical push-to-talk length, not for extended recordings; see measurements below**
3. [x] Daemon handles shutdown command cleanly and exits 0
4. [x] Unit tests cover: startup, transcribe round-trip, and shutdown over the IPC protocol — 18 tests, `ct2-daemon/test_transcribe_daemon.py`

## Evidence, 2026-09-04

**Unit tests** (fast, no model download — `FakeBackend` stands in for
CTranslate2 per the protocol-vs-backend split in the module docstring):
```
$ python3 -m pytest ct2-daemon/ -v
18 passed, 1 skipped in 0.04s
```
The 1 skip is a deliberately opt-in real-model test (mirrors the Rust
suite's `e2e_transcribes_spoken_digits_from_fixture`, `ignored` by default).

**Real end-to-end smoke test** — the actual daemon subprocess, real stdio,
real `faster-whisper` `tiny.en` model (auto-downloaded), two of the
operator's own recordings from `~/.local/share/voice-to-text/recordings/`:

| Recording | Audio length | `duration_ms` (inference only, model pre-loaded) |
|---|---|---|
| vtt_recording_gXyfPk.wav | 2.4s (typical push-to-talk) | 366ms, then 352ms on repeat |
| vtt_recording_8lufVX.wav | 43.9s (a long monologue, outlier length) | 1140ms |

**Criterion 2 is met for a typical-length recording** (366/352ms < 500ms)
but **not for the 44-second outlier** (1140ms > 500ms) — inference time
scales with audio length, and the criterion doesn't specify a length
assumption. Reporting both numbers rather than only the favourable one:
the daemon is fast enough for ordinary dictation: on this shared,
memory-pressured host, `tiny.en` is 38x real-time even on the long
outlier. A production CT2 backend would use a larger model
(large-v3/large-v3-turbo) for accuracy, which will be slower than
`tiny.en` — TASK-VTT054 (settings toggle) or a follow-up should re-measure
against the actual production model once that's wired up, not assume
`tiny.en`'s numbers carry over.

Full transcript output (real speech-to-text, unedited): both runs on the
2.4s clip produced identical text ("Well, not consumed interfaced with."),
confirming deterministic decode given fixed input. Daemon exited 0 after
`shutdown` in every run, no stderr output, no traceback.

Not implemented (explicitly out of scope for TASK-VTT053, belongs to
TASK-VTT054): the Rust-side client, settings toggle, process spawning,
crash-fallback to whisper-rs.
