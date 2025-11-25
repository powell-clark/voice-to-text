# Time Log Viewing Skill

Pretty-formatted time log viewer with colour coding and improved readability.

## Quick Reference

```bash
# Most common - last 2 hours
l2h

# Other time ranges
l30m        # Last 30 minutes
l4h         # Last 4 hours
l8h         # Last 8 hours
ltoday      # Last 24 hours
lall        # Entire log

# Compact versions (one line per entry)
l2hc        # Last 2 hours, compact
l4hc        # Last 4 hours, compact
```

## Scripts

### show-pretty.sh (NEW - Recommended)

**Purpose:** Human-readable time log output with colour coding and visual hierarchy

**Features:**
- 🎨 Colour-coded sections (cyan for Claude, green for Emmanuel)
- 📊 Visual hierarchy with icons (⚙ for agent, 👤 for human)
- 🔍 Clear section headers
- 📝 Readable multi-line format
- ⚡ Optional compact mode
- 🎯 Filter by log type (--agent-only, --human-only)

**Usage:**
```bash
# Time ranges
show-pretty.sh m 30     # Last 30 minutes
show-pretty.sh h 2      # Last 2 hours
show-pretty.sh d 7      # Last 7 days
show-pretty.sh all      # Entire log

# Options
show-pretty.sh h 4 --compact       # Compact one-line format
show-pretty.sh h 2 --agent-only    # Only Claude activity
show-pretty.sh h 2 --human-only    # Only Emmanuel activity
show-pretty.sh h 2 --no-color      # Plain text (no colours)
```

**Example output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  CLAUDE ACTIVITY (What Tools Were Used)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚙ 02:24-02:27 (session: consci)
   Claude wrote new file 'show-pretty.sh' to ~/.claude/skills/time-log/show-pretty.sh

⚙ 02:21-02:24 (session: consci)
   Claude executed bash command: Run project GPS command

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  EMMANUEL ACTIVITY (What Was Requested)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
👤 02:21-02:24 (session: 653822)
   Watching Claude investigate codebase

👤 01:33-01:36 (session: 1d1f9c)
   Watching Claude make code changes
```

### show.sh (Original - Deprecated)

**Purpose:** Basic time log output (pipe-delimited raw format)

**Status:** Still works, but show-pretty.sh recommended for better readability

**Usage:**
```bash
show.sh h 2     # Last 2 hours
show.sh m 30    # Last 30 minutes
show.sh d 7     # Last 7 days
show.sh all     # Entire log
```

## Bash Aliases

Defined in `~/.bashrc`:

```bash
# Pretty formatted (recommended)
alias l2h='~/.claude/skills/time-log/show-pretty.sh h 2'
alias l30m='~/.claude/skills/time-log/show-pretty.sh m 30'
alias l4h='~/.claude/skills/time-log/show-pretty.sh h 4'
alias l8h='~/.claude/skills/time-log/show-pretty.sh h 8'
alias ltoday='~/.claude/skills/time-log/show-pretty.sh h 24'
alias lall='~/.claude/skills/time-log/show-pretty.sh all'

# Compact versions
alias l2hc='~/.claude/skills/time-log/show-pretty.sh h 2 --compact'
alias l4hc='~/.claude/skills/time-log/show-pretty.sh h 4 --compact'

# Old alias (compatibility)
alias la2h='l2h'
```

## File Locations

The scripts automatically find time logs in these locations:

1. **Project-specific:** `CONSCIOUSNESS/HUMAN-TIME-LOG.md` and `CONSCIOUSNESS/AGENT-TIME-LOG.md`
2. **Home directory:** `~/CONSCIOUSNESS/HUMAN-TIME-LOG.md` and `~/CONSCIOUSNESS/AGENT-TIME-LOG.md`

## Time Block Calculations

**3-minute blocks per time unit:**
- 1 minute = 0.33 entries (rounds to 1)
- 1 hour = 20 entries
- 1 day = 480 entries

**Examples:**
```bash
show-pretty.sh m 30  # Shows 10 entries (30 ÷ 3)
show-pretty.sh h 2   # Shows 40 entries (2 × 20)
show-pretty.sh d 1   # Shows 480 entries (1 × 480)
```

## Colour Guide

**When colours enabled:**
- **Cyan (⚙)**: Claude's activity (tools used, files modified, commands executed)
- **Green (👤)**: Emmanuel's activity (what was requested, high-level goals)
- **Dim text**: Session IDs (first 6 chars)
- **Bold**: Time blocks and section headers

**Disable colours:**
```bash
show-pretty.sh h 2 --no-color
```

## Development

**Adding features:**
1. Edit `show-pretty.sh` in this directory
2. Test with various time ranges
3. Check colour output and plain text mode
4. Verify both compact and pretty formats work
5. Update this README

**Philosophy:**
- Make time logs actually useful
- Visual hierarchy helps scanning
- Colour coding aids comprehension
- Compact mode for when you need density
- Always show most recent entries first

## Related

- **Documentation:** `~/projects/consciousness/docs/reference/TIME-TRACKING.md`
- **Hooks:** `~/.claude/hooks/update-time-log.py` and `track-activity-context.py`
- **Log format:** Pipe-delimited TSV with 3-minute block timestamps

---

**Last Updated:** 2025-11-22 02:30 GMT
