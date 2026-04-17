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
1. `src/models.rs` exposes `ensure_model(name, progress_callback) -> Result<PathBuf>`
2. `MODELS` constant contains real upstream SHA-256 hashes verified once during implementation; hashes are not placeholders
3. Downloads use HTTPS with certificate validation (rustls-tls); no plain HTTP ever
4. Partial downloads write to `<filename>.tmp` and atomically rename only after hash verification
5. Progress callback fires at least every 256 KB of downloaded data
6. Network errors are user-friendly: `Cannot download: no internet. Connect and retry.` not a Rust error chain
7. Already-cached models are validated by SHA on first use per session; if corrupted, re-downloaded automatically
8. No partial files ever end up as the target filename — only atomic renames after verification

## Linked Tasks
- TASK-VTT029

## Parent Story
- STORY-VTT010
