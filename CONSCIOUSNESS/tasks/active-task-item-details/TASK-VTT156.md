# TASK-VTT156: Log the resolved archiving state at startup

## Context

An archive-enabled binary that writes nothing is indistinguishable from a working one. On 2026-09-03 a process holding a deleted inode answered the hotkey for eighty minutes: transcription worked perfectly, no archive entry appeared, and nothing in the log said why. The operator had no signal at all — he only learned because two agent sessions were watching the filesystem. One startup line naming the resolved archive directory, whether archiving is on, and the file cap makes the state visible on every run at the cost of one line. Suggested by the epc-voice seat, which measured the eighty-minute gap independently.

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
