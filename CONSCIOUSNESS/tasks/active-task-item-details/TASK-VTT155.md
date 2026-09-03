# TASK-VTT155: Rename config_dir to data_dir throughout

## Context

main.rs:129 assigns dirs::data_local_dir() to a variable named config_dir, and the name propagates through settings.rs, archive.rs and both tray backends. That single misnomer caused three separate errors on 2026-09-03: the README documented ~/.config/voice-to-text/settings.conf three times including the deletion command, the epc-voice session read the code and confidently corrected the archive root to the wrong path and committed it into its importer card, and a stale ~/.config settings file from October 2025 meant the wrong path failed silently rather than loudly. Pure rename, no behaviour change, fully verified by the compiler.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT018

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
