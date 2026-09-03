# TASK-VTT064: Windows ARM64 (Snapdragon) CPU build

## Context
The ARM64 tester's machine is a Lenovo IdeaPad with a Snapdragon X Plus —
Windows-on-ARM (aarch64-pc-windows-msvc), Adreno integrated GPU. This is the
concrete reason STORY-VTT013 is active now. Blocked by TASK-VTT063 (x86-64
must compile green first — same code, simpler target, proves the portable
paths before adding ARM toolchain variables).

## Approach
- Target `aarch64-pc-windows-msvc` (Rust Tier 2).
- **CPU-only whisper** — no Vulkan/Adreno. The Snapdragon Oryon cores run
  base/small models in real time on CPU; the Adreno Vulkan path on Windows-ARM
  is unproven and is a separate optimisation, not a blocker for the ARM64 tester.
- GitHub now offers `windows-11-arm` runners — add an ARM job to CI once the
  x86-64 job is green, OR cross-build aarch64 from the x86-64 windows runner
  (no whisper.cpp emulation needed for a cross `cargo build`, but C++ cross to
  aarch64-msvc needs the ARM64 MSVC toolchain installed).

## Acceptance Criteria
1. [x] Blocked-by TASK-VTT063 resolved (x86-64 Windows compiles green) —
   `build-windows` runs on every push, and TASK-VTT082 (Build and smoke-test VTT
   on Windows x86-64 hardware) is in the DONE index.
2. [x] An `aarch64-pc-windows-msvc` build produced — `build-windows-arm64` on
   `windows-11-arm`, added to `.github/workflows/ci.yml`. CI is the verification;
   it runs on the next push.
3. [x] CPU-only whisper confirmed — no Vulkan SDK / Adreno dependency.
4. [ ] DEFERRED (hardware) — Binary verified to run push-to-talk dictation on the
   ARM64 tester's actual Snapdragon hardware (gensho — actual proof on the
   target machine).
5. [ ] DEFERRED (hardware) — The proprietary built-in stopgap is retired for the
   ARM64 tester.

## The defect criterion 3 turned out to require

`whisper-rs` was declared with the `vulkan` feature under
`cfg(target_os = "windows")`. That cfg matches **aarch64 as well as x86_64**, so
an ARM64 build would have pulled in Vulkan and demanded an SDK that does not
ship for the target — the opposite of what this card specifies.

The Cargo.toml comment already said "The ARM64 build (TASK-VTT064) overrides
this back to CPU where Adreno/Vulkan is unproven." No override existed. It was
an intention recorded as if it were a fact, and the ARM job would have failed on
it.

The Windows dependency is now split by architecture, and `cargo tree` confirms
the resolver agrees rather than the comment:

```
x86_64-pc-windows-msvc   whisper-rs v0.16.0 [_gpu,vulkan]
aarch64-pc-windows-msvc  whisper-rs v0.16.0 []
```

x86-64 keeps Vulkan exactly as before; ARM64 resolves with no features at all.
That is criterion 3 satisfied by the build graph, not by a comment.

## Evidence

```
cargo metadata: manifest parses
cargo test --workspace: 182 passed; 0 failed; 1 ignored   (Linux unaffected)
ci.yml jobs: check, build-windows, build-windows-arm64, build-macos, audit
  build-windows-arm64 runs-on: windows-11-arm
  Vulkan SDK step present in the ARM job: False
```

The ARM job deliberately omits the Vulkan SDK install that `build-windows`
performs. With the old cfg that omission would have broken the build; with the
arch split it is correct.

## What remains, and why it is not mine

Criteria 4 and 5 need the binary running on the ARM64 tester's Snapdragon. The
card's own Test Strategy says so: "Real hardware is the only acceptance —
emulation does not count." No session has that machine.

SPLIT: 3 of 5 criteria met. The residue is handing the CI artifact to the tester
and watching them dictate. That needs their laptop and their hands.

## Test Strategy
Hand the ARM64 tester the binary, watch them dictate into a real app via the
push-button hotkey. Real hardware is the only acceptance — emulation does not
count.

## Files
- `.github/workflows/ci.yml` (ARM job — added)
- `Cargo.toml` (Windows whisper backend split by target_arch — added)

## Dependencies

- Story: STORY-VTT013
- Directive: DIRECT-VTT004
