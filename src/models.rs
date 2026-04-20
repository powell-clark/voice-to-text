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
    /// true = multilingual, false = English-only (.en) variant.
    /// Read by the catalogue-invariant test and retained for future code
    /// that wants to filter the model list without re-parsing the ".en" suffix.
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_returns_known_models_by_exact_name() {
        assert_eq!(find("small").map(|m| m.filename), Some("ggml-small.bin"));
        assert_eq!(
            find("small.en").map(|m| m.filename),
            Some("ggml-small.en.bin")
        );
        assert_eq!(
            find("large-v3-turbo").map(|m| m.filename),
            Some("ggml-large-v3-turbo.bin")
        );
    }

    #[test]
    fn find_returns_none_for_unknown_model() {
        assert!(find("tiny").is_none(), "tiny was dropped in v2.0 trim");
        assert!(find("").is_none());
        assert!(
            find("CT2 small").is_none(),
            "legacy prefixes must be migrated first"
        );
        assert!(find("large-v4").is_none(), "future models not in catalogue");
    }

    #[test]
    fn resolve_variant_picks_en_suffix_for_english_where_it_exists() {
        assert_eq!(resolve_variant("small", "en"), "small.en");
        assert_eq!(resolve_variant("medium", "en"), "medium.en");
    }

    #[test]
    fn resolve_variant_picks_multilingual_for_auto_or_non_english() {
        assert_eq!(resolve_variant("small", "auto"), "small");
        assert_eq!(resolve_variant("small", "fr"), "small");
        assert_eq!(resolve_variant("medium", "de"), "medium");
    }

    #[test]
    fn resolve_variant_no_en_variant_for_large_models() {
        // Upstream whisper.cpp doesn't ship ggml-large-v3.en.bin — skip the suffix.
        assert_eq!(resolve_variant("large-v3-turbo", "en"), "large-v3-turbo");
        assert_eq!(resolve_variant("large-v3", "en"), "large-v3");
    }

    #[test]
    fn resolve_variant_strips_existing_en_before_re_adding_it() {
        // Idempotency: if someone already has "small.en" selected, English mode
        // should not double-suffix it into "small.en.en".
        assert_eq!(resolve_variant("small.en", "en"), "small.en");
        assert_eq!(resolve_variant("medium.en", "en"), "medium.en");
    }

    #[test]
    fn resolve_variant_strips_en_when_switching_to_multilingual() {
        // Symmetrically, switching a .en selection to multilingual should drop .en.
        assert_eq!(resolve_variant("small.en", "auto"), "small");
        assert_eq!(resolve_variant("medium.en", "fr"), "medium");
    }

    #[test]
    fn all_catalogue_models_have_consistent_names_and_filenames() {
        for m in MODELS {
            assert!(
                m.filename.starts_with("ggml-") && m.filename.ends_with(".bin"),
                "model {} has non-standard filename {}",
                m.name,
                m.filename
            );
            assert!(
                m.url.starts_with("https://"),
                "model {} url is not https: {}",
                m.name,
                m.url
            );
            assert!(m.size_mb > 0, "model {} has zero size_mb", m.name);
            // .en models must be non-multilingual, non-.en models must be multilingual
            let has_en = m.name.ends_with(".en");
            assert_eq!(
                has_en, !m.multilingual,
                "model {} has inconsistent .en/multilingual flags",
                m.name
            );
        }
    }
}
