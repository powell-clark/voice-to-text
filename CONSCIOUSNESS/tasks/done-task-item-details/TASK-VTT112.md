# TASK-VTT112: Add stored per-model SHA-256 verification to src/models.rs

## Context

`models::ensure()` already hashed every downloaded GGML model but only
logged the digest — it never compared against a known-good value, so a
corrupted or tampered download would be silently accepted and handed to
whisper.cpp.

## Fix

- Added a `sha256: &'static str` field to `ModelInfo`, populated for all
  six catalogue entries with hashes read from HuggingFace's upstream
  git-lfs pointer files (`https://huggingface.co/ggerganov/whisper.cpp/raw/main/<filename>`
  returns `oid sha256:<hash>` without needing a full download).
- `ensure()` now compares the computed digest against `info.sha256`
  (case-insensitive) after download; on mismatch it deletes the `.tmp`
  file and returns an `Err` instead of installing the file.
- Extended the catalogue-invariant test to assert every `sha256` is
  64 lowercase hex chars, plus a new test asserting all six are unique.

## Verification

- One hash cross-checked for real: downloaded the full `ggml-small.en.bin`
  (487,614,201 bytes) in this session and ran `sha256sum` independently —
  matched the catalogue value exactly
  (`c6138d6d58ecc8322097e0f987c32f1be8bb0a18532a3f88f734d1bbf9c41e5d`).
  The other five came from the same authoritative LFS-pointer source.
- `cargo build --release`, `cargo clippy --release --all-targets -D
  warnings`, `cargo fmt --check`, and `cargo test --release` (99/99
  passing) all green on Linux.

## Acceptance criteria

- [x] Each catalogue model carries a stored expected SHA-256
- [x] `ensure()` verifies the digest and fails (not silently accepts) on
      mismatch, discarding the partial/tampered `.tmp` file
- [x] Catalogue invariants covered by unit tests (format + uniqueness)

## Dependencies

- Directive: DIRECT-VTT002
- Feature: FEAT-VTT026
