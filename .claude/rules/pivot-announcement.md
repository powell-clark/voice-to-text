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

A claim is invisible on the trunk until it merges, and that gap is wide
enough to double-claim through. The card moves from backlog to in_progress
inside a change that is still under review, so the trunk index goes on
reading "unclaimed" — accurately, because on the trunk it is unclaimed. The
index is not stale or broken; it is answering a different question from the
one the pre-flight is actually asking. So two sessions can both read a
truthful index and both conclude the task is free, with no race, no clock
skew and no misread required. This is a property of the claim flow rather
than of any one forge: wherever a claim travels in a change that is reviewed
before it lands, the window exists.

Measured 2026-08-14 on TASK-APGPS29139 (Add target ref to control requests).
One session claimed it out of the backlog index and applied a database
migration to production while another session's change for the same task had
been open for two hours. Both had authored an additive migration adding the
same column, and the one that reached production carried the later
timestamp, so merging the open change would have failed twice over — the
column already exists, plus a linear-history violation from applying an
earlier revision after a later one. It was rolled back losslessly, but only
because that table held no rows and no shipped code read the column, and
neither of those is guaranteed next time. The remedy is one command before
the claim, which is why it belongs here as a required step rather than as
advice about being careful.

## Requires

- MUST run a pre-flight before working a claimed task: check the trunk index, git log, merged changes, AND every open unmerged change, for evidence the task is already claimed elsewhere or its acceptance criteria are already satisfied
- MUST search the forge's open unmerged changes for the task id before claiming, matching the source branch name as well as the title and body — on a GitHub remote that is `gh pr list --state open --search <TASK-ID>` confirmed against each open change's head branch, and the equivalent merge-request query on any other forge
- MUST treat a trunk index row reading unclaimed as inconclusive while an open unmerged change carries the same task id — the claim travels in the change that moves the card to in_progress, so the trunk is accurate and not yet informative
- MUST stop and report when the claimed task is already shipped, blocked, or moot — close or release it with evidence instead of starting adjacent work
- MUST declare any pivot in one line naming the abandoned scope, the new scope, and the reason, before the first action of the new scope
- MUST make the claim and plan visible in the transcript before the first mutating action of an autonomous cycle

## Forbids

- MUST NOT silently substitute different work for the dispatched or claimed task
- MUST NOT expand scope beyond the claimed task without declaring the expansion first
- MUST NOT treat an already-done task as licence to begin the nearest adjacent work — report the finding and re-enter the claim flow
- MUST NOT treat an unclaimed row on the trunk index as evidence that a task is free without also checking open unmerged changes — the claim is invisible on the trunk until the change carrying it merges

## References

- precept:task_lifecycle
- precept:consciousness
- precept:precept_specification
