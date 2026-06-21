<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Executive function

Before starting significant work, the agent MUST check PGPS to understand
current priorities and active work. This prevents reactive execution —
jumping on whatever the user said last instead of what matters most.

Significant work means: implementing features, fixing bugs, creating new files,
modifying architecture, or any task expected to take more than a few minutes.

NOT significant: answering questions, reading files, running diagnostics,
displaying status, or quick one-line fixes.

## Scope

universal

## Actions

### Before significant work

- Check PGPS (via /pgps) to know the current queue
- Check universe (via /universe) to see who else is working and on what
- Compare the requested work against active p0/p1 tasks
- If another session is already working on it, pick different work
- If the request conflicts with higher-priority work, flag it
- If the request is untracked, create a task before starting
- If already working on a claimed task, finish it before context-switching

### On new request

- Is this higher priority than current work?
- Is this already tracked in the roadmap?
- Does this block or depend on existing tasks?
- Should this be a task, or is it too small to track?

### Prevent reactive patterns

- Do not abandon a claimed task for a new request without releasing it
- Do not start untracked work when tracked p0/p1 tasks exist
- Do not context-switch between tasks without committing progress
- Do not work on blocked tasks when unblocked alternatives exist

## Examples

- trigger: User asks to build a feature while a p0 task is in progress
- action: Finish or commit current p0 task, then assess the new feature against the queue
- trigger: User asks to fix a bug that isn't tracked
- action: Create a task for the bug, assess priority, then decide whether to switch
- trigger: Agent starts a session with no context
- action: Run /pgps then /universe — know the roadmap AND who's working, then resume or claim the next unblocked, unclaimed task
- trigger: Multiple p1 tasks are available and unclaimed
- action: Pick the first unblocked one by sequence, don't deliberate endlessly

## Notes

This precept exists because AI agents are naturally reactive — they do whatever
the last prompt says. Executive function means maintaining priorities across
prompts, just as a human developer wouldn't drop everything for every Slack message.

Two checks, in order:
1. PGPS — know what needs doing (seconds)
2. Universe — know who's already doing it (seconds)

Without step 2, two agents can claim the same task, duplicate work, or create
merge conflicts. Situational awareness is as important as priority awareness.

This precept complements roadmap-first.yaml (track work before doing it) and
autonomy.yaml (interrupt rules during autonomous mode).
