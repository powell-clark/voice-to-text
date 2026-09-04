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
- [x] **AC-1** — `src/models.rs` exposes `ensure_model(name, progress_callback) -> Result<PathBuf>` — verified in source
- [ ] **AC-2** — [deferred → TASK-VTT112] `MODELS` carries a stored expected SHA-256 per model — NOT implemented; `src/models.rs` computes and logs the download hash but does not check it against an expected constant
- [x] **AC-3** — Downloads use HTTPS with certificate validation (rustls-tls); no plain HTTP ever — verified in source (reqwest with rustls-tls)
- [x] **AC-4** — Partial downloads write to `<filename>.tmp` and atomically rename only after hash verification — verified in `src/models.rs`
- [x] **AC-5** — Progress callback fires at least every 256 KB of downloaded data — verified in source
- [x] **AC-6** — Network errors are user-friendly — verified in daily use (user-readable tray messages on failure)
- [ ] **AC-7** — [deferred → TASK-VTT112] Cached models re-validated by stored SHA on first use; corrupted ones re-downloaded — NOT implemented (no stored hashes to validate against)
- [x] **AC-8** — No partial files ever end up as the target filename — only atomic renames after verification — verified in source

## Cross-platform acceptance criteria (DIRECT-VTT005 parity spec)
Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 8 — model download). Behaviour is identical on all platforms (one `src/models.rs` path); the only gap is cross-cutting.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux / 🍎 macOS / 🪟 Windows — ✅ works (uniform)**
- [x] Selecting an uncached model downloads from HuggingFace over HTTPS (rustls) with tray progress and atomic `.tmp`→rename — `src/models.rs:135-193`
- [x] Cache path resolves per-OS via `dirs::cache_dir`: `~/.cache` (Linux), `~/Library/Caches` (macOS), `%LOCALAPPDATA%` (Windows)

**Cross-cutting gap (all platforms) → TASK-VTT112**
- [ ] In-app download verifies the bytes against a stored expected SHA-256 — NOT implemented (only the Linux `debian/postinst` pre-download hard-verifies, TASK-VTT110)

## Linked Tasks
- TASK-VTT029

## Parent Story
- STORY-VTT010
