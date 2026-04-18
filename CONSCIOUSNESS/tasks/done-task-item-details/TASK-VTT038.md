# TASK-VTT038: Bump changelog to 2.0.0

## Context
The semantic break (CT2-Python → whisper-rs-Rust) and the PPA switching from C to Rust are major user-visible changes. Semver demands a major version bump. The changelog must be explicit about the PPA situation so users understand why the install footprint has changed and what they must do if upgrading from 1.0.16.

## Acceptance Criteria
1. `debian/changelog` has a new entry `voice-to-text (2.0.0) noble; urgency=medium` at the top
2. The entry bullets enumerate every user-visible change:
   - Rust binary now shipped (1.0.x PPA releases installed the legacy C binary despite the rewrite existing since 1.0.16)
   - whisper-rs in-process replaces CT2 Python subprocess for sub-second transcription regardless of model size
   - Python runtime dependency removed (no more pip packages)
   - Vulkan GPU acceleration on Linux (works with NVIDIA, AMD, and Intel)
   - Model menu simplified: Small, Medium, Large-v3-turbo, Large-v3 (W and CT2 prefixes retired)
   - Default model (small.en) downloaded via postinst on first install
   - Breaking: existing `~/.cache/huggingface/hub/` CT2 models are no longer used; re-download in GGML format on first use
3. `Cargo.toml` version matches `2.0.0`
4. The changelog entry date and timezone are generated via `date -R` at commit time (not copy-pasted from earlier entries)
5. The signature line uses `Emmanuel Powell-Clark <emmanuel@powellclark.com>` (no AI attribution)

## Technical Approach
```
voice-to-text (2.0.0) noble; urgency=medium

  * Rust binary now shipped — previous 1.0.x releases installed the
    legacy C binary despite the rewrite being present in the tree
  * whisper-rs in-process replaces CT2 Python subprocess — model loads
    once at startup and stays resident for sub-second transcriptions
  * Python runtime dependency removed (no more pip install)
  * Vulkan GPU acceleration on Linux works with NVIDIA, AMD, and Intel
  * Model menu simplified: Small, Medium, Large-v3-turbo, Large-v3
  * Default model ggml-small.en.bin downloaded via postinst on first install
  * Breaking: existing CT2 model cache at ~/.cache/huggingface/ is no
    longer used; models re-download in GGML format on first selection
  * Delete 7,638 lines of obsolete C and Objective-C source code

 -- Emmanuel Powell-Clark <emmanuel@powellclark.com>  <date -R output>
```

## Test Strategy
`dpkg-parsechangelog --show-field Version` returns `2.0.0`. `dpkg-parsechangelog --show-field Date` returns a valid RFC 2822 datetime. `dpkg-parsechangelog --all` shows the full history with no parse errors.

## Files
- `debian/changelog` (prepend new entry)
- `Cargo.toml` (already bumped in TASK-VTT025)
