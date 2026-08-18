# TASK-VTT146: Fix hotkey release lost during typing wait

## Context

Down handler blocks the shared X11 event thread up to 30s waiting on typing_active, so the genuine KeyRelease queues, arrives at ~0ms elapsed, and is discarded by the 150ms stale-release guard. Recording stays true with no recovery path; audio saturates and Whisper repeats a hallucinated token that is typed each cycle.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Story: STORY-VTT015
- Directive: DIRECT-VTT002
