# TASK-VTT154: Expose archiving and denoise in the settings dialog

## Context

archive, archive_dir, archive_max_files and denoise are settings-file-only. On 2026-09-03 that cost real time twice: the README pointed at a stale ~/.config/voice-to-text/settings.conf that the app has not read since a pre-2.0 version, and editing it silently did nothing. A checkbox in the dialog that already edits prefix, prompt, corrections and newline behaviour removes that entire class of error, and makes the privacy-sensitive archive setting visible rather than hidden in a file most users never open. Narrower than TASK-VTT051, whose criteria predate both settings and list a VAD toggle that does not exist yet.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT019

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
