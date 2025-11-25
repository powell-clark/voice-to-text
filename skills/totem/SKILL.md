# Totem Skill

**Trigger phrases:** totem, validate consciousness

**Purpose:** Validate consciousness installation integrity in current project.

## What This Skill Does

Runs the totem validation script which checks:
- CONSCIOUSNESS directory structure (4 subdirectories)
- All required tracking files (18 files)
- File format validation (TSV pipes, bracket notation)
- Hook installation (3 core hooks)
- CLAUDE.md compliance (preamble, required sections)
- GPS and time log commands functionality
- Legacy file cleanup
- Cross-reference validation (consciousness repo only)

## When to Use

- After propagating consciousness to a new project
- When troubleshooting consciousness issues
- To verify project consciousness installation is valid
- Before committing consciousness structure changes

## How to Respond

When the user says "totem":

1. Run the totem validation script:
   ```bash
   ~/.claude/skills/totem/totem
   ```

2. Show the raw output - do not interpret or summarize

3. If validation fails, the script shows exactly which checks failed

4. Exit codes:
   - 0 = VALID (all checks passed)
   - 1 = INVALID (errors found)
   - 2 = Not installed (no CONSCIOUSNESS directory)

## Example Response

```
Consciousness Installation Validator

✓ CONSCIOUSNESS directory exists
✓ Subdirectory exists: CONSCIOUSNESS/roadmap/
...
✓ GPS command (pgps) works

Summary:
Passed: 29

Installation VALID
```

**Never create custom status reports.** Just run the script and show output.
