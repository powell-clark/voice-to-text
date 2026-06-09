# TASK-VTT064: Windows ARM64 (Snapdragon) CPU build for Kyle

## Context
Kyle's machine is a Lenovo IdeaPad with a Snapdragon X Plus — Windows-on-ARM
(aarch64-pc-windows-msvc), Adreno integrated GPU. This is the concrete reason
STORY-VTT013 is active now. Blocked by TASK-VTT063 (x86-64 must compile green
first — same code, simpler target, proves the portable paths before adding
ARM toolchain variables).

## Approach
- Target `aarch64-pc-windows-msvc` (Rust Tier 2).
- **CPU-only whisper** — no Vulkan/Adreno. The Snapdragon Oryon cores run
  base/small models in real time on CPU; the Adreno Vulkan path on Windows-ARM
  is unproven and is a separate optimisation, not a blocker for Kyle.
- GitHub now offers `windows-11-arm` runners — add an ARM job to CI once the
  x86-64 job is green, OR cross-build aarch64 from the x86-64 windows runner
  (no whisper.cpp emulation needed for a cross `cargo build`, but C++ cross to
  aarch64-msvc needs the ARM64 MSVC toolchain installed).

## Acceptance Criteria
1. Blocked-by TASK-VTT063 resolved (x86-64 Windows compiles green).
2. An `aarch64-pc-windows-msvc` build produced (CI job or documented local
   build on an ARM runner).
3. CPU-only whisper confirmed — no Vulkan SDK / Adreno dependency.
4. Binary verified to run push-to-talk dictation on Kyle's actual Snapdragon
   hardware (gensho — actual proof on the target machine).
5. The proprietary built-in stopgap is retired for Kyle.

## Test Strategy
Hand Kyle the binary, watch him dictate into a real app via the push-button
hotkey. Real hardware is the only acceptance — emulation does not count.

## Files
- `.github/workflows/ci.yml` (ARM job, when added)
- Possibly `Cargo.toml` / target config for aarch64-msvc specifics
