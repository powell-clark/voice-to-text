# TASK-VTT169: Fix red windows-11-arm CI job

## Context

CI on main is red on EVERY push and has been for at least several commits: the build (windows-11-arm, aarch64-msvc, CPU whisper) job fails while ubuntu-24.04, macos-latest, windows-latest x86_64 and cargo audit all pass. Root cause is upstream and specific -- ggml/src/ggml-cpu/CMakeLists.txt:109 raises 'MSVC is not supported for ARM, use clang', so whisper-rs-sys cannot configure whisper.cpp for aarch64-pc-windows-msvc with the MSVC toolset the job currently selects (cmake is invoked with -G 'Visual Studio 17 2022' -Thost=x64 -AARM64). Evidence: run 33929869599 job 101206272524 on commit 2ee4fe9, a PGPS-only commit that touched no Rust, and reproduced identically on run 33935720348. Likely fix is to select the ClangCL toolset for that job (-T ClangCL via CMAKE_GENERATOR_TOOLSET) or otherwise point whisper-rs-sys at clang-cl; alternative is to drop or allow-failure the ARM64 job until TASK-VTT064's operator decision lands. Distinct from TASK-VTT064, which is about runtime verification on real Snapdragon hardware -- this is a build-time CI failure fixable without hardware. Blocks any task whose acceptance requires green CI, including TASK-VTT102.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT013

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
