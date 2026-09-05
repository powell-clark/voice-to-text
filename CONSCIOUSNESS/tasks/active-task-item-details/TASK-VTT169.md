# TASK-VTT169: Fix red windows-11-arm CI job

## Context

CI on main is red on EVERY push and has been for at least several commits: the build (windows-11-arm, aarch64-msvc, CPU whisper) job fails while ubuntu-24.04, macos-latest, windows-latest x86_64 and cargo audit all pass. Root cause is upstream and specific -- ggml/src/ggml-cpu/CMakeLists.txt:109 raises 'MSVC is not supported for ARM, use clang', so whisper-rs-sys cannot configure whisper.cpp for aarch64-pc-windows-msvc with the MSVC toolset the job currently selects (cmake is invoked with -G 'Visual Studio 17 2022' -Thost=x64 -AARM64). Evidence: run 33929869599 job 101206272524 on commit 2ee4fe9, a PGPS-only commit that touched no Rust, and reproduced identically on run 33935720348. Likely fix is to select the ClangCL toolset for that job (-T ClangCL via CMAKE_GENERATOR_TOOLSET) or otherwise point whisper-rs-sys at clang-cl; alternative is to drop or allow-failure the ARM64 job until TASK-VTT064's operator decision lands. Distinct from TASK-VTT064, which is about runtime verification on real Snapdragon hardware -- this is a build-time CI failure fixable without hardware. Blocks any task whose acceptance requires green CI, including TASK-VTT102.

## Acceptance criteria

- [ ] The `build (windows-11-arm, aarch64-msvc, CPU whisper)` CI job reaches a
      green conclusion on a real GitHub Actions run of a commit on `main` —
      cited by run id and job id, not inferred from a local change
- [ ] The other four jobs (ubuntu-24.04, macos-latest, windows-latest x86_64,
      cargo audit) stay green in that same run — the fix must not trade one
      red job for another
- [ ] Whatever route is taken is recorded on this card with the reason: a real
      toolchain fix (clang-cl) and a deliberate removal/allow-failure of the
      job are both acceptable outcomes, but silently disabling a job while
      claiming it "fixed" is not
- [ ] If the ARM64 build proves unfixable from CI configuration alone, the job
      is explicitly marked non-blocking (or removed) with a comment naming
      TASK-VTT064 as the owner of the real ARM64 story, so main stops being
      permanently red

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT013
- Related: TASK-VTT064 (Windows ARM64 Snapdragon CPU build) — hardware/runtime
  side of ARM64; this task is only the CI build-time failure
- Blocks: TASK-VTT102 (Rename binary) and any other task whose acceptance
  requires green CI across all platforms

## Pre-mortem

### Failure modes

- Switching the generator toolset to ClangCL fixes ggml's refusal but breaks a
  different dependency that assumes MSVC (windows-sys, rdev, or whisper-rs's
  own bindgen), trading one red job for another.
- clang-cl is not installed on the `windows-11-arm` runner image, so `-T
  ClangCL` fails at configure time with a different error rather than fixing
  anything.
- The fix works at configure time but the ARM64 link step then fails, meaning
  the job goes red later in the build and the loop repeats with slower
  feedback each iteration (each CI round-trip is several minutes).
- Every verification round-trip needs a push to `main`, so a wrong guess is
  publicly visible red CI rather than a local failure.

### Weak assumptions

- That the failure is purely toolchain selection. It may instead need a
  whisper.cpp submodule bump, since the `MSVC is not supported for ARM` guard
  is upstream code that may have been added or relaxed in a later revision.
- That `windows-11-arm` is a GitHub-hosted runner that will keep existing —
  if it is a preview label that gets withdrawn, the job breaks again for an
  unrelated reason.
- That ARM64 Windows is still wanted at all. TASK-VTT064 is
  OPERATOR-DECISION-PENDING, so the honest outcome may be to stop building it
  in CI rather than to fix it.
