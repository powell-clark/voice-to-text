# TASK-VTT132: Re-transcribe last recording tray item — decode newest WAV, re-type

## Context
Implements FEAT-VTT039. Emmanuel asked for a button that "tries to transcribe
the last recording if it fails" (2026-07-17). The newest archived WAV always
survives pruning (`prune_recordings` sorts newest-first, deletes the tail), so
it is a reliable recovery source across process restarts.

## Acceptance Criteria
1. "Re-transcribe last recording" tray item on the Linux GTK tray
   (`src/tray/linux.rs`) and the portable tray (`src/tray/portable.rs`).
2. On activation: find newest `.wav` in `config_dir/recordings/`, decode with
   `whisper::decode_wav_to_samples`, and send `WorkItem::Audio` to the
   transcription worker with an EMPTY `archive_path` so the worker re-types but
   skips `save_and_cleanup` (no self-copy / delete of the source).
3. Empty recordings dir → log line + no-op (no panic, no crash).
4. Pure helper `newest_wav(dir) -> Option<PathBuf>` unit-tested (newest-by-mtime,
   ignores non-wav, empty/missing dir → None).
5. cargo fmt / clippy -D warnings / cargo test green; actual-proof re-type on
   Linux against a known recording.

## Technical Approach
- Add a `WorkItem` sender (`work_tx` clone) to the tray so a menu handler can
  enqueue audio. Linux GTK tray runs on the main thread; the mpsc `Sender` is
  `Send`/`Clone` — pass it into `Tray::new` (and the portable equivalent).
- Reuse the existing worker path unchanged: `WorkItem::Audio { samples,
  archive_path: PathBuf::new() }` transcribes, re-types, updates
  `last_transcription`, and skips archiving because `archive_path` is empty
  (main.rs:636 guard).
- `newest_wav` is a pure fn (mtime max over `.wav` entries) — shared by both
  trays and unit-tested without a GUI.

## Dependencies
- FEAT-VTT039 (parent feature).
- Reuses `whisper::decode_wav_to_samples` (TASK-VTT023) and the existing
  transcription worker / `WorkItem` channel.
