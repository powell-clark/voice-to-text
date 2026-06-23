---
id: FEAT-VTT026
status: maintained
kano: must-have
verified: v2.0.0
---

# FEAT-VTT026: Automatic GGML model download from HuggingFace

## Kano
must-have (p1)

## Description
When the user selects a model that is not present in the local cache, VTT downloads the GGML file from the upstream whisper.cpp HuggingFace mirror, verifies its SHA-256 integrity, and stores it for future use. Progress is shown in the tray so the user knows the wait is productive.

## User Observable Behaviour
- Selecting `Medium` from the tray when only `Small` is cached: tray immediately shows `Downloading Medium model... 0%`
- Progress updates flow into the tray status: `Downloading Medium model... 47%`
- On completion: `Medium model downloaded` notification, then `Loading model...`, then `Ready`
- If the download fails mid-way, partial files are removed; next attempt starts fresh
- If SHA-256 does not match: file deleted, status shows `Model integrity check failed`, user asked to retry
- All downloaded models live at `~/.cache/voice-to-text/models/ggml-*.bin`

## Acceptance Criteria
- [x] `src/models.rs` exposes `ensure_model(name, progress_callback) -> Result<PathBuf>` — verified in source
- [x] `MODELS` constant contains real upstream SHA-256 hashes verified once during implementation; hashes are not placeholders — verified in `src/models.rs` TASK-VTT029
- [x] Downloads use HTTPS with certificate validation (rustls-tls); no plain HTTP ever — verified in source (reqwest with rustls-tls)
- [x] Partial downloads write to `<filename>.tmp` and atomically rename only after hash verification — verified in `src/models.rs`
- [x] Progress callback fires at least every 256 KB of downloaded data — verified in source
- [x] Network errors are user-friendly — verified in daily use (user-readable tray messages on failure)
- [x] Already-cached models are validated by SHA on first use per session; if corrupted, re-downloaded — verified in source logic
- [x] No partial files ever end up as the target filename — only atomic renames after verification — verified in source

## Linked Tasks
- TASK-VTT029

## Parent Story
- STORY-VTT010
