# TASK-VTT032: Delete dead C and Objective-C code

## Context
The project contains 7,638 lines of pre-Rust-rewrite C and Objective-C source still on disk: `src/linux/*.c` (3,359 lines, 9 files), `src/common/*.c` and `*.h` (403 lines, 6 files), `src/macos/*.m` (3,876 lines, 4 files). None of it is referenced by Cargo or by the Rust code. `Makefile.linux` builds it, but after TASK-VTT035 `debian/rules` stops invoking that Makefile.

Keeping dead code clutters navigation, confuses new contributors, and risks someone accidentally reviving obsolete logic. This task deletes it.

## Acceptance Criteria
1. `src/linux/audio.c`, `src/linux/audio.h`, `src/linux/gui.c`, `src/linux/gui.h`, `src/linux/keyboard.c`, `src/linux/keyboard.h`, `src/linux/typing.c`, `src/linux/typing.h`, `src/linux/transcribe.c`, `src/linux/transcribe.h`, `src/linux/main.c` are deleted (11 files)
2. `src/common/logging.c`, `src/common/logging.h`, `src/common/queue.c`, `src/common/queue.h`, `src/common/settings.c`, `src/common/settings.h` are deleted (6 files); `src/common/transcribe.py` is deleted by TASK-VTT031
3. `src/macos/VTTDaemon.m`, `src/macos/VTTDaemon.h`, `src/macos/VTTOnboarding.m`, `src/macos/VTTOnboarding.h` are deleted (4 files)
4. `Makefile.linux` is deleted (no longer invoked by debian/rules after TASK-VTT035)
5. `Makefile` is deleted or reduced to a tiny wrapper around `cargo build` so the old C-macOS build flow cannot be triggered
6. After cleanup, `find src -name "*.c" -o -name "*.h" -o -name "*.m"` returns zero results
7. `cargo build --release` still succeeds
8. The Debian source package built from the cleaned tree is smaller than the 1.0.16 source package by at least 5 MB

## Technical Approach
`git rm -r src/linux src/common/*.c src/common/*.h src/macos Makefile.linux` in one commit. Optionally delete `Makefile` in the same commit, or reduce it to `all: ; cargo build --release` as a convenience wrapper for `make` users.

Verify no `#include` lines reference the deleted headers; since none of the Rust code uses C headers, this is a pure deletion.

## Test Strategy
`cargo build --release` after deletion. Manual spot-check that no script under `scripts/` or `debian/` references the deleted files. Run `grep -rn "src/linux\|src/macos\|src/common/.*\.c" .` (outside `src/`) to catch stale references in build scripts, CI config, or documentation.

## Files
- `src/linux/` (delete directory, 11 files)
- `src/common/*.c`, `src/common/*.h` (delete 6 files)
- `src/macos/` (delete directory, 4 files)
- `Makefile.linux` (delete)
- `Makefile` (delete or reduce to cargo wrapper)
