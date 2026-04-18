# TASK-VTT024: Commit ADR-0003 and approve the architectural direction

## Context
ADR-0003 at `CONSCIOUSNESS/adr/0003-whisper-rs-in-process-model.md` documents the decision to replace CT2-via-Python-subprocess with whisper-rs in-process. It must be committed to the `whisper-rs-migration` branch before any implementation work begins so the decision trail is explicit in version control, and the ADR status must transition from `Proposed` to `Accepted`.

## Acceptance Criteria
1. `CONSCIOUSNESS/adr/0003-whisper-rs-in-process-model.md` exists with the full Context, Decision, Consequences, Rollback, and References sections
2. The Status line reads `Accepted` (not `Proposed`) after Emmanuel's verbal approval is captured in the session log
3. The ADR is committed on branch `whisper-rs-migration` with commit message `docs(adr): 0003 whisper-rs in-process model replaces CT2 Python subprocess`
4. The commit author is Emmanuel Powell-Clark per global commit policy
5. `CONSCIOUSNESS/stories/STORY-ACTIVE-INDEX.md` and `CONSCIOUSNESS/tasks/TASK-BACKLOG-INDEX.md` reference ADR-0003 in the doc fields where relevant
6. `git log --oneline` on the branch shows the ADR commit landed before any code changes

## Technical Approach
1. Update the Status field in `0003-whisper-rs-in-process-model.md` from `Proposed` to `Accepted` once Emmanuel says "go"
2. `git add CONSCIOUSNESS/adr/0003-whisper-rs-in-process-model.md CONSCIOUSNESS/stories/ CONSCIOUSNESS/tasks/ CONSCIOUSNESS/features/`
3. `git commit -m "docs(adr): 0003 whisper-rs in-process model replaces CT2 Python subprocess"`

## Test Strategy
`git log -1 --stat` shows the ADR file was the primary change. `cat CONSCIOUSNESS/adr/0003-*.md | head -5` confirms Status is Accepted.

## Files
- `CONSCIOUSNESS/adr/0003-whisper-rs-in-process-model.md` (status update)
- `CONSCIOUSNESS/stories/STORY-ACTIVE-INDEX.md`, `CONSCIOUSNESS/tasks/TASK-BACKLOG-INDEX.md`, `CONSCIOUSNESS/features/FEATURE-BACKLOG-INDEX.md` (already updated in the grooming step)
