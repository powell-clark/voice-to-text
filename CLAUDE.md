# Voice-to-Text Project Instructions

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