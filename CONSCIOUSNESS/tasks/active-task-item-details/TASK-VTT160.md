# TASK-VTT160: Prune only the recordings we wrote

## Context

collect_archived matches any .wav one level under archive_dir, and prune_archive deletes the oldest past archive_max_files along with any .json beside it, then removes directories that empty. Pointing archive_dir at a directory that already holds audio — a typo, or a reasonable misreading of a setting called archive_dir — makes the tool silently delete the operator's unrelated files after the next dictation. Data loss from a config typo, in a feature whose entire purpose is keeping audio. Prune must match only the vtt_<id>.wav shape this code writes, and the dated-directory shape it creates.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT020

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
