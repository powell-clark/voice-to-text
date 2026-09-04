# TASK-VTT050: Silero VAD integration

## Acceptance Criteria
1. Recording auto-stops within 1s of speech ending (silence detected by Silero VAD)
2. Background noise below a configurable threshold does not trigger recording
3. VAD runs in-process alongside whisper-rs — no additional process or network call
4. The feature is toggleable in `settings.conf`; default is enabled

## Crate research, 2026-09-04 — NOT claimed, needs an operator decision first

Checked crates.io for a Silero VAD crate before claiming this. The ecosystem is
fragmented and immature — no single dominant, well-maintained option:

| crate | max version | total downloads |
|---|---|---|
| silero | 0.7.0 | 5,243 |
| silero-vad-rust | 6.2.2 | 4,626 |
| wavekat-vad | 0.1.17 | 4,178 |
| silero-vad-rs | 0.1.2 | 2,805 |
| silero-vad-crs | 0.4.0 | 655 |
| silero_vad_burn | 0.1.1 | 382 |
| vad-silero-rs | 0.1.5 | 272 |
| silero-vad-pure | 0.1.1 | 138 |

Every option in the mainstream candidates bundles or depends on an ONNX Runtime
inference path (`silero`, `silero-vad-rust`, `silero-vad-rs`, `wavekat-vad`) —
only `silero-vad-pure` and `silero-vad-crs` claim to avoid it (pure-Rust /
zero-runtime-deps respectively), at the cost of being the least-downloaded,
least-proven options. None has meaningful download counts by normal crates.io
standards, and none is obviously the "just use this" choice the way `whisper-rs`
is for this project's existing engine.

This matters specifically for this project: it already fights hard for small,
predictable, cross-platform (Windows/macOS/Linux) build artifacts (see
`packaging/`, the WiX installer, the Debian package, the PPA build). Bundling
ONNX Runtime is a real binary-size and cross-compilation-complexity cost, and
picking the wrong crate here is expensive to unwind once settings/tray UX
depend on it.

Per `precept:precept_specification`'s cousin rule for this repo (brainstorming
skill's architectural-decision gate) and `authorship-flow`'s "file an ADR
before implementing any one-way-door architectural decision", the crate choice
(and whether ONNX Runtime bundling is acceptable at all) needs operator
sign-off before implementation starts — not something to pick silently in an
unattended session. Left in backlog, unclaimed, with this research attached so
the next session (human or agent, with the operator's steer) can decide fast
rather than re-doing this survey.
