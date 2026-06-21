# Voice-to-Text Project Instructions

## PPA Information
- Launchpad account: `powellclark` (NO HYPHEN)
- PPA target: `ppa:powellclark/voice-to-text`
- dput target: `powellclark-voice-to-text`

## Build Commands
- **Linux build**: `cargo build --release` (produces `target/release/vtt-linux`)
- **Linux .deb**: `bash scripts/release-local.sh [--install]`
- **Linux PPA release**: `bash scripts/release-ppa.sh` (pbuilder hard-gate, auto-dput)
- **macOS build**: `make` (uses default Makefile — legacy Objective-C bundle)
- **Clean**: `cargo clean` (Linux) or `make clean` (macOS)

Makefile.linux was retired in v2.0 (TASK-VTT032) — the C sources it
built were replaced by the Rust crate. Any reference to it in older
docs is stale; see `CHANGELOG.md` for the v2.0 rewrite summary.

## Commit Messages
- Use conventional commit style (feat:, fix:, chore:, etc.)
- Keep them concise and focused on "why" not "what"

# Claude Code Guidelines for voice-to-text Project

## Multi-Machine Development Workflow

**CRITICAL**: This repository has Claude Code running on three machines simultaneously:
- **macOS** - Primary development machine
- **Linux** - Testing and Linux-specific development
- **Windows** - Testing and Windows-specific development

Both machines work on the `main` branch. To avoid conflicts:

1. **Always pull before starting work**: `git pull origin main`
2. **Commit frequently**: Small, focused commits
3. **Push regularly**: Share changes immediately after committing
4. **Pull after pushing**: Check for any updates from the other machine
