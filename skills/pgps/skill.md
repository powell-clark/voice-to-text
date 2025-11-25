# GPS Skill

**Purpose:** Real-time positioning system for your work - shows exactly where you are in the project/system landscape right now. Displays current TODOs, running services, git state, database status, background processes, future roadmap, and cross-project context.

**When to use:** Emmanuel asks "where are we?", "gps", "what's our position?", or needs real-time awareness of current system state and trajectory.

---

## Quick GPS Command (Optimized)

**CRITICAL: When user says "gps", execute the bash command and STOP. NO COMMENTARY. NO SUMMARY. NO INTERPRETATION. The GPS output is complete and self-documenting. Any text after GPS output breaks the deterministic format.**

When user says "gps", execute this single bash command:

```bash
# GPS - All sections always displayed with empty indicators
{
    echo "GPS | $(date '+%Y-%m-%d %H:%M:%S %Z') | $(basename $PWD) | $(git branch --show-current 2>/dev/null)";
    echo "";

    # SESSION - least important, at top
    echo "SESSION.ID:";
    echo "  $(basename $PWD)-$(git branch --show-current | sed 's/[^a-z0-9]/-/g')-$(echo $RANDOM | md5sum | head -c6)";

    # SERVICES - infrastructure status (always show)
    echo "";
    echo "SERVICES.STATUS:";
    PM2=$(pm2 list 2>/dev/null | grep -c "online")
    [ -z "$PM2" ] && PM2="0"
    DB=$(psql -h localhost -p 5435 -U postgres -d jabjab_dev -c "SELECT 1" 2>/dev/null > /dev/null && echo "up" || echo "down")
    if [ -n "$PM2" ] || [ -n "$DB" ]; then
        echo "  pm2: $PM2 | db: $DB";
    else
        echo "  [not applicable - no services]";
    fi

    # EPICS.MASTER - master epics (always show)
    echo "";
    echo "EPICS.MASTER:";
    if [ -f ~/ROADMAP.md ]; then
        MASTER_EPICS=$(grep -E "^\- \[EPIC-" ~/ROADMAP.md 2>/dev/null | head -3 | sed 's/^- /  /')
        if [ -n "$MASTER_EPICS" ]; then
            echo "$MASTER_EPICS"
        else
            echo "  [no master epics defined]"
        fi
    else
        echo "  [file not found: ~/ROADMAP.md]";
    fi

    # EPICS.PROJECT - project epics (always show)
    echo "";
    echo "EPICS.PROJECT:";
    if [ -f docs/ROADMAP.md ]; then
        PROJECT_EPICS=$(grep -E "^\- \[EPIC-" docs/ROADMAP.md 2>/dev/null | head -3 | sed 's/^- /  /')
        if [ -n "$PROJECT_EPICS" ]; then
            echo "$PROJECT_EPICS"
        else
            echo "  [no project epics defined]"
        fi
    else
        echo "  [file not found: docs/ROADMAP.md]";
    fi

    # STORY.MASTER - master stories (always show)
    echo "";
    echo "STORY.MASTER:";
    if [ -f ~/STORY.md ]; then
        MASTER_STORIES=$(grep -E "^- " ~/STORY.md 2>/dev/null | sed 's/^- /  /' | head -3)
        if [ -n "$MASTER_STORIES" ]; then
            echo "$MASTER_STORIES"
        else
            echo "  [no master stories]"
        fi
    else
        echo "  [file not found: ~/STORY.md]";
    fi

    # STORY.PROJECT - project stories (always show)
    echo "";
    echo "STORY.PROJECT:";
    if [ -f STORY.md ]; then
        PROJECT_STORIES=$(grep -E "^- " STORY.md 2>/dev/null | sed 's/^- /  /' | head -3)
        if [ -n "$PROJECT_STORIES" ]; then
            echo "$PROJECT_STORIES"
        else
            echo "  [no project stories]"
        fi
    else
        echo "  [file not found: STORY.md]";
    fi

    # ROADMAP.MASTER - master roadmap (always show)
    echo "";
    echo "ROADMAP.MASTER:";
    if [ -f ~/ROADMAP.md ]; then
        MASTER_ROADMAP=$(grep -E "^##+ " ~/ROADMAP.md 2>/dev/null | head -3 | sed 's/^#* /  /')
        if [ -n "$MASTER_ROADMAP" ]; then
            echo "$MASTER_ROADMAP"
        else
            echo "  [no master roadmap]"
        fi
    else
        echo "  [file not found: ~/ROADMAP.md]";
    fi

    # ROADMAP.PROJECT - project roadmap (always show)
    echo "";
    echo "ROADMAP.PROJECT:";
    if [ -f docs/ROADMAP.md ]; then
        # Look for epics (format: [EPIC-XXX])
        EPICS=$(grep -E "^\- \[EPIC-" docs/ROADMAP.md 2>/dev/null | head -3 | sed 's/^- /  /')
        if [ -n "$EPICS" ]; then
            echo "$EPICS"
        else
            echo "  [no epics found in file]"
        fi
    else
        echo "  [file not found: docs/ROADMAP.md]";
    fi

    # BACKLOG.MASTER - master backlog (always show)
    echo "";
    echo "BACKLOG.MASTER:";
    if [ -f ~/BACKLOG.md ]; then
        MASTER_BACKLOG=$(grep -E "^- " ~/BACKLOG.md 2>/dev/null | sed 's/^- /  /' | head -3)
        if [ -n "$MASTER_BACKLOG" ]; then
            echo "$MASTER_BACKLOG"
        else
            echo "  [empty]"
        fi
    else
        echo "  [file not found: ~/BACKLOG.md]";
    fi

    # BACKLOG.PROJECT - project backlog (always show)
    echo "";
    echo "BACKLOG.PROJECT:";
    if [ -f BACKLOG.md ]; then
        ITEMS=$(grep -E "^- " BACKLOG.md 2>/dev/null | sed 's/^- /  /' | head -3)
        if [ -n "$ITEMS" ]; then
            echo "$ITEMS"
        else
            echo "  [empty]"
        fi
    else
        echo "  [file not found: BACKLOG.md]";
    fi

    # GIT - current changes (always show)
    echo "";
    echo "GIT.STATUS:";
    BRANCH=$(git branch --show-current 2>/dev/null || echo "no-repo")
    CHANGES=$(git status --porcelain 2>/dev/null | wc -l)
    echo "  branch: $BRANCH | changes: $CHANGES";

    # Master TODO - cross-project (always show)
    echo "";
    echo "TODO.MASTER:";
    if [ -f ~/TODO-MASTER.md ]; then
        TASKS=$(grep -E "^\- \[ \]" ~/TODO-MASTER.md 2>/dev/null | sed 's/^- \[ \] /  /' | head -5)
        if [ -n "$TASKS" ]; then
            echo "$TASKS"
        else
            echo "  [empty]"
        fi
    else
        echo "  [file not found: ~/TODO-MASTER.md]";
    fi

    # Project TODO - current work (always show)
    echo "";
    echo "TODO.PROJECT:";
    if [ -f TODO.md ]; then
        # Get current/active session tasks
        TASKS=$(sed -n '/## Session:.*Active/,/^---$/{/### Tasks/,/^---$/{/^- /p}}' TODO.md | grep -v DONE | grep -v "Recently Completed" | sed 's/^- /  /' | head -5)
        if [ -z "$TASKS" ]; then
            # Try alternative format
            TASKS=$(grep -E "^- \[ \]" TODO.md 2>/dev/null | sed 's/^- \[ \] /  /' | head -5)
            if [ -z "$TASKS" ]; then
                echo "  [empty]"
            else
                echo "$TASKS"
            fi
        else
            echo "$TASKS"
        fi
    else
        echo "  [file not found: TODO.md]";
    fi

    # Current task in progress (always show)
    echo "";
    echo "CURRENT.TASK:";
    if [ -f TODO.md ]; then
        # Extract task marked with [IN_PROGRESS]
        TASK=$(grep "\[IN_PROGRESS\]" TODO.md 2>/dev/null | head -1 | sed 's/.*\[IN_PROGRESS\] /  /')
        if [ -n "$TASK" ]; then
            echo "$TASK"
        else
            echo "  [no active task]"
        fi
    else
        echo "  [no active task - TODO.md not found]";
    fi

    # Acceptance criteria (always show)
    echo "";
    echo "ACCEPTANCE.CRITERIA:";
    if [ -f TODO.md ]; then
        if grep -q "Success:" TODO.md 2>/dev/null; then
            grep "Success:" TODO.md | head -1 | sed 's/.*Success: /  /';
        elif grep -q "Done when:" TODO.md 2>/dev/null; then
            grep "Done when:" TODO.md | head -1 | sed 's/.*Done when: /  /';
        elif grep -q "Complete when:" TODO.md 2>/dev/null; then
            grep "Complete when:" TODO.md | head -1 | sed 's/.*Complete when: /  /';
        else
            echo "  [no acceptance criteria defined]";
        fi
    else
        echo "  [no acceptance criteria - TODO.md not found]";
    fi
} 2>&1
```

Alternative full command (detailed):

```bash
# Full GPS with all details
{
    echo "=== GPS POSITION REPORT ===";
    echo "Time: $(date '+%Y-%m-%d %H:%M:%S %Z')";
    echo "";

    echo "PROJECT TODO:";
    grep -E "^\- \[ \]" TODO.md 2>/dev/null | sed 's/^/  /' || echo "  No tasks";

    echo -e "\nHOME TODO:";
    grep -E "^\- \[ \]" ~/TODO-MASTER.md 2>/dev/null | sed 's/^/  /' || echo "  No tasks";

    echo -e "\nGIT STATUS:";
    git status --short | head -10 | sed 's/^/  /';

    echo -e "\nSERVICES:";
    pm2 list 2>/dev/null | grep -E "online|stopped|errored" | sed 's/^/  /' || echo "  PM2: No services";
    psql -h localhost -p 5435 -U postgres -d jabjab_dev -c "SELECT 'Database: Connected'" 2>/dev/null | grep Connected || echo "  Database: Not running";
} 2>&1
```

---

## Output Format

**Concise, scannable, one view - showing both project and home TODOs:**

```
## TODO (Project - TODO.md)
Session: xk4j9m | 2 tasks

- [x] Restart PM2 with new granular mode control
- [ ] Rerun E2E tests with updated step definition

## TODO (Master - ~/TODO-MASTER.md)
Session: home-f8m2q9 | 1 task

- [ ] Update home directory documentation

## BACKLOG (first 15 items)
Must-Have: 8 items | Performance: 8 items | Nice-to-Have: 4 items | Delighters: 12 items

(First 15 items shown)
- Visual regression tests for ET1 mobile responsiveness (#40-45)
- Form progress percentage indicator (#38)
...

## ROADMAP (phases)
# Phase 1: Foundation (MVP)
## Core Features
- User authentication and basic profile
- Form ET1 implementation
- Chat interface

# Phase 2: Enhanced UX
## Advanced Features
...

## Git Status
M CLAUDE.md
M ecosystem.config.js
M package.json
?? .claude/skills/status/

## PM2 Services
┌────┬───────────────┬─────────┬────────┬─────────┬──────────┐
│ id │ name          │ pid     │ status │ cpu     │ mem      │
├────┼───────────────┼─────────┼────────┼─────────┼──────────┤
│ 0  │ jabjab-api    │ 2560926 │ online │ 0%      │ 95.5mb   │
│ 1  │ jabjab-web    │ 2560932 │ online │ 0%      │ 94.6mb   │
└────┴───────────────┴─────────┴────────┴─────────┴──────────┘

## Background Processes
bash_id: e4bfcc - E2E tests running (test 24/80)
```

---

## When to Use Each Section

### TODO Sections (Project + Master)
**Show both separately:**

**Project TODO (TODO.md):**
- Session ID (for identifying which Claude Code instance)
- Count of tasks (pending/in_progress/completed)
- All active tasks in current project

**Master TODO (~/TODO-MASTER.md):**
- Session ID (for home directory work)
- Count of tasks (cross-project work)
- All active home-level tasks

**When empty:** Report "No active tasks" for each section

**Key insight:** Shows what you're working on at both project and system levels simultaneously

### BACKLOG Section
**Show:**
- Count by Kano category (Must-Have, Performance, Nice-to-Have, Delighters)
- First 15 items for quick overview
- Link to full file if needed

**When to check:** User asks about future work or "what's next"

### ROADMAP Section
**Show:**
- Phase headers (## Phase N: Name)
- Major feature areas under each phase
- Strategic direction at a glance

**When to check:** User asks about strategic vision, product phases, or long-term plans

### Database Section
**Show:**
- Connection status to development database
- Quick health check

**Key info:**
- Is database running and accessible?
- Should be "Connected" if Docker container is up

### Git Status Section
**Show:**
- Uncommitted changes (`git status --short`)
- Current branch
- Commits ahead/behind origin

**Key info:**
- Files to commit before ending session
- Whether on feature branch or main

### PM2 Services Section
**Show:**
- Service status (online/stopped/errored)
- PID, CPU, memory
- Which mode (check process command with `ps aux | grep pnpm`)

**Key info:**
- Are both API and Web running?
- Memory issues (>500mb)?

### Background Processes Section
**Show:**
- Active background bash processes (tests, builds, etc)
- bash_id for monitoring with BashOutput
- Progress if available

**Key info:**
- Tests still running?
- Forgotten processes to kill?

---

## Extended Status (GitHub Integration)

**When user asks for full project status, add:**

```bash
echo -e "\n## GitHub Issues (Open)"
gh issue list --limit 10 --state open

echo -e "\n## GitHub PRs (Open)"
gh pr list --limit 5 --state open

echo -e "\n## Recent Commits"
git log --oneline -5
```

**Only show when explicitly requested** to keep default status fast.

---

## Status Report Frequency

**Automatic status checks:**
- Never automatic - only when user asks

**When user asks "status?":**
- Show full status (all sections)
- Identify blockers or issues
- Suggest next actions if idle

**When user asks specific question:**
- "tests passing?" → Just test output
- "what's in backlog?" → Just BACKLOG section
- "git status?" → Just git section

---

## Implementation Pattern

```typescript
// Pseudocode for status command (includes both TODOs)
async function getStatus() {
  const [projectTodo, masterTodo, backlog, git, pm2, bg] = await Promise.all([
    readTodo('TODO.md'),           // Project-specific
    readTodo('~/TODO-MASTER.md'),  // Cross-project home work
    readBacklog(),
    getGitStatus(),
    getPM2Status(),
    getBackgroundProcesses()
  ]);

  return formatStatus({ projectTodo, masterTodo, backlog, git, pm2, bg });
}
```

**Run checks in parallel** for speed (< 1 second total).

**Key:** Read both TODO files concurrently, display them separately but in same output

---

## Key Reminders

**DO:**
- ✅ Keep output concise and scannable
- ✅ Show counts/summaries, not full details
- ✅ Highlight blockers or issues
- ✅ Include session ID for multi-instance tracking

**DON'T:**
- ❌ Dump full TODO/BACKLOG/ROADMAP
- ❌ Run status checks automatically
- ❌ Include verbose logs or output

---

**Last Updated:** 2025-11-11 18:18:06 GMT
