# TASK-VTT169: Fix red windows-11-arm CI job

## Context

CI on main is red on EVERY push and has been for at least several commits: the build (windows-11-arm, aarch64-msvc, CPU whisper) job fails while ubuntu-24.04, macos-latest, windows-latest x86_64 and cargo audit all pass. Root cause is upstream and specific -- ggml/src/ggml-cpu/CMakeLists.txt:109 raises 'MSVC is not supported for ARM, use clang', so whisper-rs-sys cannot configure whisper.cpp for aarch64-pc-windows-msvc with the MSVC toolset the job currently selects (cmake is invoked with -G 'Visual Studio 17 2022' -Thost=x64 -AARM64). Evidence: run 33929869599 job 101206272524 on commit 2ee4fe9, a PGPS-only commit that touched no Rust, and reproduced identically on run 33935720348. Likely fix is to select the ClangCL toolset for that job (-T ClangCL via CMAKE_GENERATOR_TOOLSET) or otherwise point whisper-rs-sys at clang-cl; alternative is to drop or allow-failure the ARM64 job until TASK-VTT064's operator decision lands. Distinct from TASK-VTT064, which is about runtime verification on real Snapdragon hardware -- this is a build-time CI failure fixable without hardware. Blocks any task whose acceptance requires green CI, including TASK-VTT102.

## Acceptance criteria

- [x] The `build (windows-11-arm, aarch64-msvc, CPU whisper)` CI job reaches a
      green conclusion on a real GitHub Actions run of a commit on `main` —
      cited by run id and job id, not inferred from a local change
- [x] The other four jobs (ubuntu-24.04, macos-latest, windows-latest x86_64,
      cargo audit) stay green in that same run — the fix must not trade one
      red job for another
- [x] Whatever route is taken is recorded on this card with the reason: a real
      toolchain fix (clang-cl) and a deliberate removal/allow-failure of the
      job are both acceptable outcomes, but silently disabling a job while
      claiming it "fixed" is not
- [x] If the ARM64 build proves unfixable from CI configuration alone, the job
      is explicitly marked non-blocking (or removed) with a comment naming
      TASK-VTT064 as the owner of the real ARM64 story, so main stops being
      permanently red — NOT NEEDED: the real toolchain fix landed, so no job
      was disabled, removed or marked non-blocking. The ARM64 build is
      genuinely building again, not silenced.

## Evidence

Run 33936833870, commit 8fefb91, `conclusion: success` — all five jobs green,
which is the first fully-green run on `main` in this stretch:

| job id | job | conclusion |
|---|---|---|
| 101226335817 | build (windows-11-arm, aarch64-msvc, CPU whisper) | success |
| 101226335633 | fmt + clippy + test + build (ubuntu-24.04) | success |
| 101226335747 | build (windows-latest, x86_64-msvc, Vulkan whisper) | success |
| 101226335832 | build (macos-latest, arm64, Metal whisper) | success |
| 101226335911 | cargo audit (vulnerabilities only) | success |

The fix is two environment variables on that job alone (`.github/workflows/ci.yml`),
no source change and no change to any other job:

- `CMAKE_GENERATOR: Ninja` plus `CC_/CXX_aarch64_pc_windows_msvc: clang-cl` —
  clears ggml's `MSVC is not supported for ARM, use clang`. The Visual Studio
  generator takes its compiler from the `-T` toolset, which the cmake crate
  only exposes through a builder call whisper-rs-sys never makes; under Ninja
  the crate forwards the cc-crate compiler as `CMAKE_C_COMPILER` instead, so
  `CC`/`CXX` actually reach CMake.
- `CXXFLAGS_aarch64_pc_windows_msvc: /EHsc` — restores the C++ exception
  handling CMake would normally supply for an MSVC-family compiler, which the
  cmake crate's wholesale `CMAKE_CXX_FLAGS` override drops. This was latent
  all along and only became visible once the guard above stopped aborting the
  build at configure time.

Both clang-cl (LLVM 20.1.6) and ninja (1.13.2) are preinstalled on the
`windows-11-arm` image, so nothing new is installed at CI time.

Prior red state, for the record: run 33929869599 job 101206272524 on commit
2ee4fe9 — a PGPS-only commit touching no Rust — failed identically, which is
what established the failure as pre-existing rather than caused by any
in-flight change.

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT013
- Related: TASK-VTT064 (Windows ARM64 Snapdragon CPU build) — hardware/runtime
  side of ARM64; this task is only the CI build-time failure
- Blocks: TASK-VTT102 (Rename binary) and any other task whose acceptance
  requires green CI across all platforms

## Progress log

- **Attempt 1** (commit 9107512, run 33936231818): switched the ARM job to the
  Ninja generator with `CC_aarch64_pc_windows_msvc=clang-cl`. This DID clear the
  original blocker — `MSVC is not supported for ARM, use clang` is gone, CMake
  configured, ran the ARM feature probes (SVE/SME/FP16) and began compiling.
  The job still failed, but further along and for a different reason:
  `ggml/src/gguf.cpp(420,13): error: cannot use 'try' with exceptions disabled`.
  This is the pre-mortem's "fix works at configure time, build step then fails"
  failure mode, arriving exactly as predicted.
- **Diagnosis of that second failure**: CMake normally supplies `/EHsc` for MSVC-
  family compilers, but the cmake crate overwrites `CMAKE_CXX_FLAGS` wholesale
  with the flags it gets from the cc crate, so that default never reaches
  clang-cl and C++ exceptions stay off. Verified in the crate sources rather than
  guessed: cmake-0.1.58 `set_compiler` builds `-DCMAKE_CXX_FLAGS=` from
  `compiler.args()` and its `skip_arg` filter drops only `-O*`/`/O*`/`-g`, while
  cc-1.4.4 appends `CXXFLAGS` (target-scoped variants included) to those args.
- **Attempt 2** (commit 019205d, verified on run 33936833870): add
  `CXXFLAGS_aarch64_pc_windows_msvc=/EHsc`. Green. Note that 019205d's own run
  (33936788018) was cancelled by a subsequent documentation push landing while
  it was still building — the workflow's concurrency group cancels in-flight
  runs — so the verifying run is the next one, 33936833870 on 8fefb91, which
  carries the identical CI change. Lesson for future CI work in this repo: do
  not push anything while a verification run is in flight.

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
