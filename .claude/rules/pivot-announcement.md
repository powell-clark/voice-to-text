<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Pivot announcement

Every scope change is announced before it is acted on: a claimed task found already shipped, blocked, or moot is reported and released — never silently swapped for different work — and any pivot is declared in one line (from, to, because) before the first action of the new scope.

## Narrative

Origin incident, 2026-07-31: a session dispatched to a runner-toggle task
found it already merged and silently moved on to adding concurrency control
across fourteen workflows, leaving the operator asking "whats going on
here?". The July insights corpus shows the pattern generalises: sessions
split hard between fully-achieved and interrupted, and the split tracks
almost exactly with whether a plan was visible before work began. Operators
interrupt what they cannot see.

The fix is not less autonomy — autonomous cycles are the federation's
highest-leverage mode. It is a visible seam at every scope boundary: the
claim is announced, the plan is stated, and any change of target is declared
before the first action of the new scope. A one-line declaration costs
nothing; an interrupted loop costs the whole cycle.

Claims are coordination data, not implementation cargo. Publish the atomic
backlog-to-in_progress move to trunk before implementation begins; do not
leave ownership hidden inside a code branch or pull request. That closes the
old review-window race, but a row can still outlive its actor after a crash,
so the trunk index is necessary rather than sufficient evidence. Pre-flight
also checks the live universe, worktrees and branch history. Query the forge
only when a human-tier gated change or an older open change may carry relevant
evidence; ordinary reversible work has no pull request to inspect.

Measured 2026-08-14 on TASK-APGPS29139 (Add target ref to control requests).
One session claimed it out of the backlog index and applied a database
migration to production while another session's change for the same task had
been open for two hours. Both had authored an additive migration adding the
same column, and the one that reached production carried the later
timestamp, so merging the old open change would have failed twice over — the
column already exists, plus a linear-history violation from applying an
earlier revision after a later one. It was rolled back losslessly, but only
because that table held no rows and no shipped code read the column, and
neither of those is guaranteed next time. The remedy is an atomic published
claim plus a liveness-aware pre-flight, which is why both are required rather
than left as advice about being careful.

## Requires

- MUST run a pre-flight before working a task: check the trunk index, live universe, worktrees, git log and branches for evidence that another live actor owns it or its acceptance criteria are already satisfied
- MUST atomically publish the task's backlog-to-in_progress claim to trunk before implementation begins; a claim left only on a feature branch or pull request does not reserve the task
- MUST inspect open gated changes when the task touches a human-tier surface or local history shows an older task branch — on a GitHub remote use `gh pr list --state open --search <TASK-ID>` and confirm the head branch
- MUST treat a trunk claim without matching live-session evidence as potentially stale and investigate before taking or releasing it
- MUST stop and report when the claimed task is already shipped, blocked, or moot — close or release it with evidence instead of starting adjacent work
- MUST declare any pivot in one line naming the abandoned scope, the new scope, and the reason, before the first action of the new scope
- MUST make the claim and plan visible in the transcript before the first mutating action of an autonomous cycle

## Forbids

- MUST NOT silently substitute different work for the dispatched or claimed task
- MUST NOT expand scope beyond the claimed task without declaring the expansion first
- MUST NOT treat an already-done task as licence to begin the nearest adjacent work — report the finding and re-enter the claim flow
- MUST NOT hide a claim inside implementation work or assume that creating a local branch made ownership visible to another actor
- MUST NOT require a forge query for ordinary reversible work when the trunk claim, live universe and local branch evidence settle ownership; the PR lane is an exception, not the registry

## References

- precept:task_lifecycle
- precept:consciousness
- precept:precept_specification
