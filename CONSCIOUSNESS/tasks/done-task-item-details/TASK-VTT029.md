# TASK-VTT029: Model download and cache logic in src/models.rs

## Context
GGML models must exist on disk before `WhisperEngine::new` can load them. Rather than shipping multi-gigabyte models in the `.deb` (too large for a PPA), we download them on demand from `huggingface.co/ggerganov/whisper.cpp`. The default model is pre-fetched by the Debian postinst (TASK-VTT037), but additional models that the user selects from the tray menu download on first use.

## Acceptance Criteria
1. `src/models.rs` defines `pub struct ModelInfo { name: &'static str, filename: &'static str, url: &'static str, sha256: &'static str, size_mb: u32, multilingual: bool, english_only_variant: Option<&'static str> }` and a `pub const MODELS: &[ModelInfo]` list covering small, small.en, medium, medium.en, large-v3-turbo, large-v3
2. `pub fn model_path(name: &str) -> PathBuf` returns `~/.cache/voice-to-text/models/<filename>`; the directory is created on demand
3. `pub fn ensure_model(name: &str, progress: impl Fn(u64, u64)) -> anyhow::Result<PathBuf>` returns the cached path if the file exists and sha256-verifies; otherwise downloads from HuggingFace with streaming progress callbacks, writes atomically via a `.tmp` sibling file, verifies sha256 after download, and only then renames into place
4. Failed downloads leave no half-written file in the cache — the `.tmp` is deleted on any error
5. sha256 mismatch after download deletes the file and returns an error like "integrity check failed for ggml-small.en.bin"
6. The progress callback is invoked at least every 256 KB with `(bytes_downloaded, total_bytes)`; callers use this to drive tray status updates like "Downloading small.en... 47%"
7. Network-unavailable gracefully returns a user-facing error: "Cannot download model: no internet. Connect to the internet or select a model you already have."
8. The sha256 hashes are the real upstream values from `huggingface.co/ggerganov/whisper.cpp`, not placeholders — verified once during implementation and committed as constants

## Technical Approach
Use `reqwest::blocking::Client` (with `rustls-tls` so no OpenSSL system dep) in streaming mode via `.bytes_stream()`. Hash incrementally using `sha2::Sha256::update` on each chunk. Write to `<target>.tmp`, then `fs::rename` on successful hash match.

```rust
pub struct ModelInfo {
    pub name: &'static str,
    pub filename: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub size_mb: u32,
    pub multilingual: bool,
}

pub const MODELS: &[ModelInfo] = &[
    ModelInfo { name: "small.en", filename: "ggml-small.en.bin", url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin", sha256: "...", size_mb: 488, multilingual: false },
    // ...
];

pub fn ensure_model(name: &str, progress: impl Fn(u64, u64)) -> anyhow::Result<PathBuf> {
    let info = MODELS.iter().find(|m| m.name == name)
        .ok_or_else(|| anyhow::anyhow!("unknown model: {}", name))?;
    let path = model_path(info.filename);
    if path.exists() && sha256_file(&path)? == info.sha256 {
        return Ok(path);
    }
    download_with_progress(info, &path, progress)?;
    Ok(path)
}
```

## Test Strategy
Unit test: point `MODELS` at a tiny test URL (localhost fixture) with a known sha, download twice — first time fetches, second time is a cache hit with zero network traffic. Integration test: manually request a model download from the tray, watch the progress notification advance, verify the file appears at `~/.cache/voice-to-text/models/` with the correct sha.

## Files
- `src/models.rs` (create)
- `src/main.rs` (modify — `mod models;`)
- `Cargo.toml` (already has `reqwest` and `sha2` from TASK-VTT025)
