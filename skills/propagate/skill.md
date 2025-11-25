# Propagate Skill

Deploy consciousness system changes (hooks, skills, scripts) from this repo to all registered projects and home directory.

## Purpose

After making changes to hooks, skills, or GPS scripts in this repo, propagate them to:
- `~/.claude/hooks/` (home directory)
- `~/.claude/skills/` (home directory)
- Other registered projects' `.claude/` directories

## Usage

Invoke this skill when:
- GPS scripts have been updated (gps.sh, pgps)
- Hooks have been modified or added
- Skills have been changed
- You need to sync changes across all projects

## What It Does

1. **Identify changed files** - Find what's been modified in hooks/, skills/, scripts/
2. **Copy to home directory** - Deploy to ~/.claude/hooks/ and ~/.claude/skills/
3. **Copy to registered projects** - Update each project in ~/.claude/projects-registry.txt
4. **Set permissions** - Ensure executability on scripts
5. **Validate deployment** - Check files copied correctly
6. **Report results** - Show what was deployed where

## Deployment Targets

**Always deployed:**
- `~/.claude/hooks/` - All hook scripts
- `~/.claude/skills/gps/` - GPS scripts (gps.sh, pgps)
- `~/.claude/skills/autonomous-workflow/` - Workflow patterns

**Per-project (if .claude/ exists):**
- `project/.claude/hooks/` - Project-specific hook overrides
- `project/.claude/skills/` - Project-specific skills

## File Categories

**Hooks (15 files):**
- SessionStart: session-init.py, load-todo.py
- SessionEnd: post-session.py, post-session-metrics.py
- PreToolUse: protect-changelog.py, protect-active-adrs.py, track-session-state.py
- PostToolUse: update-time-log.py, track-activity-context.py, sync-todo.py, sync-todo-to-tasks.py, post-tool-use-orchestrator.py, track-session-state.py
- UserPromptSubmit: check-todo-changes.py, capture-user-message.py, display-time-summary.py

**Skills:**
- gps/ - GPS scripts (gps.sh, pgps)
- autonomous-workflow/ - Self-directed patterns

**Scripts:**
- validate-tsv.py - TSV format validation
- validate-cross-references.py - Cross-reference checking
- validate-story.py - SMART criteria validation
- allocate-id.py - ID generation with locking

## Safety Checks

Before deploying:
1. Verify source files exist in this repo
2. Check target directories are writable
3. Backup existing files (optional, ask user)
4. Validate syntax on Python files
5. Test GPS commands after deployment

## Example Output

```
=== Consciousness Propagation ===

Changed files detected:
  • skills/gps/pgps (modified)
  • skills/gps/gps.sh (modified)

Deploying to targets:
  [1/3] ~/.claude/skills/gps/ ... ✓
  [2/3] ~/projects/jabjab/.claude/skills/gps/ ... ✓
  [3/3] ~/projects/nichiren-buddhism-library/.claude/skills/gps/ ... ✓

Testing GPS commands:
  pgps ... ✓ (runs without errors)
  gps.sh ... ✓ (runs without errors)

=== Propagation Complete ===
Deployed to 3 locations in 2.4s
```

## Integration

This skill works with:
- `scripts/install.sh` - Initial setup for new machines/projects
- `/gps` and `/pgps` - Validates deployment by running commands
- Git status - Shows what's changed since last commit

## When NOT to Use

Don't use this skill for:
- Installing on new machine (use `scripts/install.sh` instead)
- Adding new projects (manually edit `~/.claude/projects-registry.txt`)
- Modifying tracking files (ROADMAP.md, STORY.md, etc.)
- Time logs (HUMAN-TIME-LOG.md, AGENT-TIME-LOG.md)
