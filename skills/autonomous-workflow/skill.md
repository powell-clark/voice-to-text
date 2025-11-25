# Autonomous Workflow Skill

**Purpose:** Execute tasks from TASKS.md autonomously using run-debug validation loops.

**Trigger:** User says "work autonomously", "execute tasks", "build the system", or "follow the plan"

---

## What This Does

Reads TASKS.md, finds in-progress tasks, executes them with validation, updates status, commits changes, and moves to next task. No human intervention except for blockers.

---

## The Loop

```
1. Run GPS to see current state
2. Find task with status="in-progress"
3. Execute using run-debug loop with validation
4. Update TASKS.md: in-progress → done
5. Commit with structured message
6. Repeat until no in-progress tasks or blocked
```

---

## Usage

**User:** "Work autonomously on the current task"

**Claude:**
1. Runs GPS
2. Sees TASK-001 is in-progress: "Make validate-gps-format-v2.py run all 15 section checks in under 100ms"
3. Executes: modifies validation script, runs tests, checks performance
4. Validates: procedural checks (linters) + semantic check (achieves success criteria?)
5. Updates TASKS.md to mark done
6. Commits: "TaskCompleted: taskId=TASK-001, story=STORY-CC002, duration=47m"
7. Reports completion and asks if should continue to next task

---

## Implementation

See docs/AUTONOMOUS-EXECUTION-SPEC.md for complete specification.

Key components:
- **Run-debug loop:** Execute → Validate → Retry on failure → Escalate after N attempts
- **Dual-phase validation:** Procedural (linters/type-checkers) + Semantic (LLM review)
- **Checkpoint system:** Save state at each phase for recovery
- **Structured commits:** Include task ID, story, duration, success criteria

---

## Multi-Session Support

When multiple Claude sessions work on same repo:
1. Each session claims a task by marking it in-progress
2. Immediately commits and pushes the claim
3. If push fails (conflict), pulls and claims different task
4. Work partitioning prevents conflicts

---

## Session Handoff

When switching machines:
```bash
# Laptop: finish work
git add . && git commit -m "Session handoff from laptop" && git push

# Desktop: continue work
git pull origin main
# Claude reads .consciousness/session-handoff.json
# Continues from last_task_completed + 1
```

---

## Success Criteria

Task marked "done" when:
- ✅ Procedural validation passes
- ✅ Semantic validation passes (achieves stated goal)
- ✅ Tests pass
- ✅ Committed to git
- ✅ TASKS.md updated

Task marked "blocked" when:
- ❌ Retry limit exceeded
- ❌ Merge conflict after 3 attempts
- ❌ Human decision required

---

## Example Session

```
User: Work autonomously

Claude: Running GPS to check current state...
        Found TASK-032: "Update pgps to show correct sections"
        Story: STORY-CC042
        Success: GPS display shows all required sections

        Executing task...
        [reads skills/gps/project-gps.sh]
        [modifies script to fix sections]
        [runs: bash skills/gps/project-gps.sh]
        [validates output matches required sections]

        Procedural validation: ✅ Passed (shellcheck clean)
        Semantic validation: ✅ Passed (shows correct sections)

        Updating TASKS.md...
        Committing changes...

        TaskCompleted: TASK-032
        Duration: 12 minutes
        Next task: TASK-002 (Hook JSON format standardisation)

        Continue to next task? [yes/no]
```

---

## Status

**Current:** Draft specification created
**Next:** Implement basic autonomous loop
**Future:** Add checkpoint recovery, multi-session coordination

---

**Last Updated:** 2025-11-16
