# TASK-VTT161: Resolve archive_dir predictably or refuse it

## Context

resolve_archive_dir expands a leading tilde-slash and otherwise takes the string literally. A bare tilde becomes a directory named tilde in the process cwd; a dollar-HOME prefix becomes a directory named dollar-HOME since settings.conf is not shell-expanded; and a relative path resolves against the service cwd, which for a systemd user unit is not the operator's shell directory, so recordings land somewhere unfindable or fail silently. This setting decides where the operator's voice is written and it should either resolve predictably or refuse and say so. Pair with TASK-VTT156, which already prints the resolved directory at startup, so a refused setting becomes visible rather than silent.

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
