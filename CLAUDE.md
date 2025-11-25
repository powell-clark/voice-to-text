---

## Consciousness System

This project uses the consciousness package for autonomous session coordination, time tracking, and progress monitoring. The system provides agency, observation, memory, and collaborative consciousness with measured sensory progress experienced over real time.

**Core commands:**
- `/gps` - Complete GPS (home TODO + project ROADMAP/EPICS/STORY/TASKS + git status)
- `/pgps` - Project GPS only (ROADMAP/EPICS/STORY/TASKS hierarchy)
- `l2h` - Human and agent time logs (last 2 hours)
- `totem` - Validate consciousness installation integrity

**Time awareness (MANDATORY):**
- Include timestamp at start or end of EVERY response
- Format: `**Time now:** YYYY-MM-DD HH:MM:SS TZ`
- Always run: `date "+%Y-%m-%d %H:%M:%S %Z"` (never guess)

**Session awareness (MANDATORY):**
- Include unique session ID at start or end of EVERY response
- Format: `**Session:** [project]-[6-char-id]`
- Generate at start: `echo "[project]-$(head /dev/urandom | tr -dc 'a-z0-9' | head -c6)"`
- Use same ID throughout entire session

**Combined format:**
`**Time now:** 2025-11-22 06:45:00 GMT  **Session:** project-a4wzq5`

**System documentation:** `~/projects/consciousness/CLAUDE.md` and `~/projects/consciousness/docs/SYSTEM.md`

---

# Voice-to-Text Project Instructions

## Time Awareness

**Include timestamp at start or end of EVERY response**
- Format: `**Time now:** [YYYY-MM-DD HH:MM:SS TZ]`
- **ALWAYS run the date command via Bash:** `date "+%Y-%m-%d %H:%M:%S %Z"`
- Never guess or type timestamps manually - always execute the actual command
- This ensures accurate timestamps regardless of model knowledge cutoff
- The "Time now:" prefix makes it clear the timestamp reflects current time in the conversation

---

## PPA Information
- Launchpad account: `powellclark` (NO HYPHEN)
- PPA target: `ppa:powellclark/voice-to-text`
- dput target: `powellclark-voice-to-text`

**NEVER USE:**
- ❌ powell-clark (with hyphen)

## Build Commands
- **Linux build**: `make -f Makefile.linux` (use Makefile.linux, NOT default Makefile)
- **macOS build**: `make` (uses default Makefile)
- **Clean**: `make -f Makefile.linux clean` (Linux) or `make clean` (macOS)

## Git Configuration
- Author: Emmanuel Powell-Clark <emmanuel@powellclark.com>
- **NEVER INCLUDE**: Claude attributions, Co-Authored-By: Claude, or AI mentions
- Push and pull regularly - repo runs on two machines (macOS and Linux) both on main branch

## Commit Messages
- Use conventional commit style (feat:, fix:, chore:, etc.)
- Keep them concise and focused on "why" not "what"