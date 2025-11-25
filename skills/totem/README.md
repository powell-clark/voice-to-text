# Consciousness Installation Validator (Totem)

Validates consciousness tracking system installation in any project repository. Run this command inside a project where consciousness is installed to verify everything is configured correctly.

## Quick Reference

```bash
cd ~/projects/your-project
totem
```

## Exit Codes

| Code | Status | Description |
|------|--------|-------------|
| 0 | Valid | Installation valid (may have warnings) |
| 1 | Invalid | Validation failed with errors |
| 2 | Not installed | CONSCIOUSNESS directory not found |

## What Totem Validates

### 1. Directory Structure
- CONSCIOUSNESS directory exists
- Required subdirectories: roadmap/, epics/, stories/, tasks/

### 2. Required Files
- ROADMAP.md, ROADMAP-BACKLOG.md, ROADMAP-DONE.md
- EPICS.md, EPIC-BACKLOG.md, EPIC-DONE.md
- STORY.md, STORY-BACKLOG.md, STORY-DONE.md
- TASKS.md, TASK-BACKLOG.md, TASK-DONE.md
- TODO.md
- HUMAN-TIME-LOG.md, AGENT-TIME-LOG.md

### 3. File Format Validation
- ROADMAP files: bracket format `[Q4-2025] [EPIC-CC001]`
- STORY-BACKLOG: bracket format `[STORY-CC008] [epic: CC002]`
- EPIC/TASK/TIME-LOG files: pipe-delimited TSV format
- TODO.md: markdown checklist format

### 4. Hook Installation
- sync-todo.py present and executable
- update-time-log.py present and executable
- track-activity-context.py present and executable

### 5. CLAUDE.md Compliance
- File exists
- Contains required sections: CONSCIOUSNESS, TodoWrite

### 6. GPS Command
- pgps command exists
- Executes without errors

### 7. Time Log Command
- show-pretty.sh exists
- Executes without errors

### 8. Legacy File Detection
- Checks for old file names that should be removed
- GPS.md, ROADMAP-OLD.md, MASTER-ROADMAP.md, TODO-MASTER.md

### 9. Cross-Reference Validation
- Runs validate-cross-references.py if available
- Verifies task→story→epic links are valid

## Example Output

```
Consciousness Installation Validator

✓ CONSCIOUSNESS directory exists
✓ Subdirectory exists: CONSCIOUSNESS/roadmap/
✓ Subdirectory exists: CONSCIOUSNESS/epics/
✓ File valid: CONSCIOUSNESS/roadmap/ROADMAP.md
✓ File valid: CONSCIOUSNESS/epics/EPICS.md
✓ Hook installed: sync-todo.py
✓ GPS command (pgps) works
✓ Time log command (l2h) works
✓ All task→story→epic references valid

Summary:
Passed: 27
Warnings: 1

Installation VALID (with warnings)
```

## Validation States

**Passed (Green ✓)**
- File or check completed successfully
- No issues found

**Warning (Yellow ⚠)**
- File present but empty
- Optional feature missing
- Non-critical issue found

**Error (Red ✗)**
- Required file missing
- File format corrupted
- Critical validation failed

## Fixing Common Issues

### Missing subdirectories
```bash
mkdir -p CONSCIOUSNESS/{roadmap,epics,stories,tasks}
```

### Missing tracking files
```bash
# Copy from consciousness source
cp ~/projects/consciousness/precepts/ROADMAP.md CONSCIOUSNESS/roadmap/
cp ~/projects/consciousness/precepts/EPICS.md CONSCIOUSNESS/epics/
```

### Hooks not executable
```bash
chmod +x .claude/hooks/*.py
```

### Legacy files present
```bash
# Remove old files
rm GPS.md ROADMAP-OLD.md MASTER-ROADMAP.md TODO-MASTER.md
```

## Integration

Run totem as part of your development workflow:

**Before committing:**
```bash
totem && git commit
```

**In CI/CD:**
```bash
#!/bin/bash
if ! totem; then
    echo "Consciousness validation failed"
    exit 1
fi
```

**After propagation:**
```bash
cd ~/projects/consciousness
./scripts/propagate.sh
cd ~/projects/jabjab
totem  # Verify propagation succeeded
```

## Implementation

The totem command is a single bash script with inline validation logic. It uses colour-coded output and detailed error messages to help diagnose issues quickly.

**Location:** `~/.claude/skills/totem/totem`

**Language:** Bash (no dependencies beyond coreutils)

**Performance:** Executes in <100ms on typical installations

---

Last Updated: 2025-11-22 02:30 GMT
