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