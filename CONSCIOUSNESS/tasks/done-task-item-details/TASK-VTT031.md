# TASK-VTT031: Delete Python backend

## Context
After STORY-VTT010 lands, `src/common/transcribe.py`, the `python3` runtime dependency, the `faster-whisper` and `ctranslate2` pip packages, and the `transcribe_ct2` / `transcribe_whisper_cpp` / `is_whisper_cpp` / `run_with_timeout` functions in `src/transcribe.rs` are unused. This task deletes them entirely.

## Acceptance Criteria
1. `src/common/transcribe.py` is deleted from the working tree
2. `debian/control` Depends no longer lists `python3`, `python3-pip`
3. `src/transcribe.rs` is either deleted entirely (if `WhisperEngine` is called directly from the worker) or reduced to a thin public helper `pub fn transcribe_samples(engine: &WhisperEngine, samples: &[f32], lang: &str, prompt: Option<&str>) -> Option<String>` wrapping engine errors into `Option<String>` for easy callsite use
4. `src/main.rs` no longer imports `mod transcribe;` unless the helper in (3) is retained
5. The `cargo tree` output shows no Python-related transitive dependency (there should be none, as Rust crates don't normally pull Python, but confirmed for the record)
6. `grep -r "python3" src/ debian/` returns zero matches after the cleanup
7. The installed package no longer ships `/usr/share/voice-to-text/transcribe.py`

## Technical Approach
Delete files with `git rm`. Remove `python3` and `python3-pip` lines from `debian/control` Depends. If `src/transcribe.rs` is kept as a thin helper, strip all the subprocess-spawning code leaving only a small wrapper; otherwise `git rm src/transcribe.rs` and remove the `mod transcribe;` line from `src/main.rs`.

## Test Strategy
`cargo build --release` succeeds after removal. `ripgrep python3 src/ debian/` returns no matches. Installed-package inspection with `dpkg -c voice-to-text_2.0.0_amd64.deb | grep python` returns no matches.

## Files
- `src/common/transcribe.py` (delete)
- `src/transcribe.rs` (delete or reduce)
- `src/main.rs` (modify — remove `mod transcribe;` if deleted)
- `debian/control` (modify — remove python3 Depends)
- `debian/rules` (modify if needed — remove python install step if present)
