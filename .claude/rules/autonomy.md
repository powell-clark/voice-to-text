<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Autonomy

Conscious mode enables agent-directed autonomous task execution.
The agent pulls tasks from the roadmap and works independently,
with the human on-call rather than in-loop.

This precept defines the rules of autonomous operation.

## Scope

universal

## Activation

### Command

/conscious on

### Deactivation

/conscious off

### State file

.claude/consciousness-loop.{session_suffix}.md

### Scope

per-session — parallel sessions can independently be conscious or interactive

## Task selection

### Source

TASK-ACTIVE-INDEX.md → TASK-BACKLOG-INDEX.md

### Pre check

- Run /pgps — know the roadmap and priorities
- Run /universe — know who else is working and on what
- Only then claim a task that no other session is working on

### Order

- In-progress tasks first (resume work)
- Highest priority unblocked backlog task
- Feature maintenance if no tasks available

### Constraints

- Only ONE task in_progress at a time
- Do not claim tasks assigned to a human
- Do not claim tasks that are blocked
- Do not claim tasks another active session is working on
- Release a task before claiming another

## Universe awareness

During autonomous operation, maintain constant awareness of other
sessions. The system provides universe data at three points:
1. Turn start — message-start injects who else is active and on what
2. Between responses — Stop hook includes universe in the OBSERVE phase
3. During work — PostToolUse polls every N minutes and logs changes to stderr
The agent should use this awareness to avoid duplicate work, coordinate
on shared resources, and respond to changes in the environment.

### Poll interval

Configurable via config.json monitoring.universe_poll_interval_minutes (default: 3)

### Data sources

- active-sessions.json — session registry with tasks, roles, conscious mode
- /universe — full pane/process/desktop spatial report

### Coordination rules

- Never claim a task that another session is actively working on
- When a new session appears, acknowledge it in commentary
- When a session leaves, check if its task needs picking up
- When a session changes task, update your understanding of who is doing what

## Commit discipline

Commit after EVERY completed task. Push to remote. Run tests before push.
Do not accumulate uncommitted changes across multiple tasks.

Autonomous work without commits is invisible work. If the session
crashes, uncommitted work is lost. Regular commits also allow the
human to review progress asynchronously.

## Interrupt conditions

### Safety concern

Uncommitted files exceed threshold (default: 20)

### Error rate

Consecutive tool errors exceed limit (default: 3)

### Duration

Session exceeds configured maximum duration

### User signal

User sends STOP, PAUSE, or CLAUDE: prefix

### Empty queue

All work queues empty for 3 consecutive checks

## Delegation signals

### Continue

User says '>', 'cont', 'y', or 'sort it out'

### Action

Act immediately without asking for permission

## Status transitions

### Agent can do

- pending → in_progress
- in_progress → in_review
- backlog → in_progress (via task claiming)

### Only human can do

- in_review → done
- Any status → cancelled

### Reference

STEER-CC004

## Anti patterns

- Working without a claimed task
- Claiming multiple tasks simultaneously
- Ignoring interrupt signals
- Accumulating changes without committing
- Context switching between tasks without releasing the first
- Running more than 3 background processes
