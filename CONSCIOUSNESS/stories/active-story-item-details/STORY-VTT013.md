# STORY-VTT013: Windows builds

## User Story
As Emmanuel I want Windows builds so that my Windows desktop — and Kyle's
Snapdragon laptop — has the same VTT as Linux and Mac.

## Why This Matters
The IVM is explicit: "Free, open-source, cross-platform voice-to-text that
outperforms paid alternatives on **Windows**, Mac, and Linux" serving "the
majority of machines, not just the elite." Windows IS the majority of
machines. Today we ship Linux (Rust/xdotool) and macOS (legacy bundle) but
**zero Windows**. Routing a Windows user to a proprietary built-in dictation
tool contradicts the mission — it is the elite-substitute the project exists
to displace.

The codebase was architected for this from the Rust rewrite (STORY-VTT005):
`src/hotkey/portable.rs` and `src/tray/portable.rs` are cfg-gated to
macOS+Windows, and `Cargo.toml` carries a `cfg(target_os = "windows")`
dependency block (enigo, tray-icon, muda, rdev, whisper-rs). But it has
**never been compiled, run, or CI'd on Windows.** "Designed-in" is risho
(theory); a green Windows build is gensho (actual proof). This story closes
that gap.

The concrete trigger: Kyle owns a Lenovo IdeaPad with a Snapdragon X Plus —
Windows-on-ARM (aarch64). He becomes our first real Windows user and the
proving ground. A built-in tool is an explicit *interim* stopgap that retires
the moment our binary runs on his machine.

## Scope
- **In scope (this pass):** x86-64 Windows compile-green + a Windows CI job
  (TASK-VTT063); ARM64/Snapdragon CPU build for Kyle (TASK-VTT064); the
  pre-existing runtime-feature fixes (singleton mutex TASK-VTT044, signal
  handler TASK-VTT045) once compilation is green; `.msi` installer
  (TASK-VTT046); Authenticode signing (TASK-VTT047).
- **First cut is CPU-only whisper** on Windows — no Vulkan/Adreno dependency.
  The Snapdragon's Oryon cores run base/small models comfortably in real time
  on CPU, and Vulkan-on-Windows-ARM via Adreno is unproven. GPU acceleration
  on Windows is a deliberate follow-up, not a blocker.
- **Out of scope:** full release-matrix automation (STORY-VTT014 / TASK-VTT048
  tracks tag-driven multi-OS release); macOS work (STORY-VTT012).

## Build-isolation guarantee
Windows work must NOT skew the Linux/mac build. The `portable.rs` split and
per-target `Cargo.toml` blocks already enforce this: Linux compiles the exact
same shared code whether or not Windows works. ARM-specific concerns stay
cfg-gated — no platform hacks bleed into shared code.

## Tasks
- TASK-VTT063 — Windows x86-64 compile-green + CI build job (foundational; the missing prerequisite)
- TASK-VTT064 — Windows ARM64 (Snapdragon) CPU build for Kyle
- TASK-VTT044 — Windows singleton: replace flock with CreateMutexW named mutex
- TASK-VTT045 — Windows signal handling: replace sigwait with SetConsoleCtrlHandler
- TASK-VTT046 — cargo-wix .msi installer with Start Menu shortcut
- TASK-VTT047 — Windows Authenticode code signing

## Definition of Done
A green Windows CI build on every push/PR. A runnable Windows binary that does
push-to-talk dictation. Kyle's Snapdragon transcribing with our tool, the
built-in stopgap retired.
