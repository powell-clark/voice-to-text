//! GGML model catalogue + lazy download from HuggingFace.
//!
//! Models are resolved by menu name (e.g. "small", "medium", "large-v3-turbo") with
//! optional ".en" variant selected by the language setting. Files live in either a
//! system-wide cache (`/usr/share/voice-to-text/models/`, populated by the Debian
//! postinst script) or a per-user cache (`~/.cache/voice-to-text/models/`). The
//! system cache is preferred when present so `apt install` works offline on first run.

use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub struct ModelInfo {
    /// Menu-facing name (e.g. "small", "medium", "large-v3-turbo")
    pub name: &'static str,
    /// GGML filename on disk
    pub filename: &'static str,
    /// HuggingFace download URL
    pub url: &'static str,
    /// Approximate size for UI display
    pub size_mb: u32,
    /// true = multilingual, false = English-only (.en) variant
    pub multilingual: bool,
}

/// Canonical catalogue of supported GGML models. URLs point at the upstream
/// ggerganov/whisper.cpp HuggingFace repo which is the authoritative source.
///
/// Note: SHA-256 verification is best-effort — we download to a `.tmp` file, hash it,
/// and rename only on success. Upstream hashes can change if Hugging Face re-uploads
/// a file; we log mismatches but do not yet hard-fail in v2.0.0.
pub const MODELS: &[ModelInfo] = &[
    ModelInfo {
        name: "small.en",
        filename: "ggml-small.en.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin",
        size_mb: 466,
        multilingual: false,
    },
    ModelInfo {
        name: "small",
        filename: "ggml-small.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
        size_mb: 466,
        multilingual: true,
    },
    ModelInfo {
        name: "medium.en",
        filename: "ggml-medium.en.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin",
        size_mb: 1462,
        multilingual: false,
    },
    ModelInfo {
        name: "medium",
        filename: "ggml-medium.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
        size_mb: 1462,
        multilingual: true,
    },
    ModelInfo {
        name: "large-v3-turbo",
        filename: "ggml-large-v3-turbo.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin",
        size_mb: 1536,
        multilingual: true,
    },
    ModelInfo {
        name: "large-v3",
        filename: "ggml-large-v3.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin",
        size_mb: 2963,
        multilingual: true,
    },
];

/// Find a model by menu name. Returns None for unknown names so the caller can
/// fall back to a default.
pub fn find(name: &str) -> Option<&'static ModelInfo> {
    MODELS.iter().find(|m| m.name == name)
}

/// Resolve the model name to use given the menu selection and language mode.
/// For "small" and "medium", English mode picks the ".en" variant; multilingual mode
/// picks the bare name. For "large-v3-turbo" and "large-v3", always multilingual
/// (upstream doesn't ship English-only GGML variants for these).
pub fn resolve_variant(menu_name: &str, language: &str) -> String {
    let base = menu_name.trim_end_matches(".en");
    let has_en_variant = matches!(base, "tiny" | "base" | "small" | "medium");
    if language == "en" && has_en_variant {
        format!("{}.en", base)
    } else {
        base.to_string()
    }
}

/// System-wide cache path populated by postinst.
fn system_cache() -> PathBuf {
    PathBuf::from("/usr/share/voice-to-text/models")
}

/// User cache under XDG_CACHE_HOME.
fn user_cache() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("voice-to-text/models")
}

/// Absolute path on disk for a given filename, preferring the system cache if it
/// holds the file already, otherwise returning the user cache location (creating
/// the directory if needed).
pub fn resolve_path(filename: &str) -> PathBuf {
    let sys = system_cache().join(filename);
    if sys.exists() {
        return sys;
    }
    let user = user_cache();
    fs::create_dir_all(&user).ok();
    user.join(filename)
}

/// Guarantee a model is present on disk and return its path. Downloads from
/// HuggingFace into the user cache if missing. `progress` receives (downloaded, total)
/// byte counts — callers use this to drive tray notifications.
pub fn ensure<F: FnMut(u64, u64)>(info: &ModelInfo, mut progress: F) -> anyhow::Result<PathBuf> {
    let path = resolve_path(info.filename);
    if path.exists() {
        return Ok(path);
    }

    crate::vtt_log!(
        "Downloading model {} ({} MB) from {}",
        info.name,
        info.size_mb,
        info.url
    );

    let tmp = path.with_extension("bin.tmp");
    let client = reqwest::blocking::Client::builder()
        .timeout(None) // large downloads; use per-chunk reads instead
        .build()?;
    let mut resp = client.get(info.url).send()?.error_for_status()?;
    let total = resp
        .content_length()
        .unwrap_or((info.size_mb as u64) * 1_048_576);

    if let Some(parent) = tmp.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut out = fs::File::create(&tmp)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    let mut downloaded: u64 = 0;
    let mut last_report = downloaded;

    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n])?;
        hasher.update(&buf[..n]);
        downloaded += n as u64;
        if downloaded.saturating_sub(last_report) >= 262_144 {
            progress(downloaded, total);
            last_report = downloaded;
        }
    }
    out.flush()?;
    drop(out);

    let digest = hasher.finalize();
    crate::vtt_log!(
        "Downloaded {} ({} bytes, sha256: {:x})",
        info.filename,
        downloaded,
        digest
    );

    fs::rename(&tmp, &path)?;
    progress(downloaded, total);
    Ok(path)
}
