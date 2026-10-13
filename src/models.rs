//! GGML model catalogue + lazy download from HuggingFace.
//!
//! Models are resolved by menu name (e.g. "small", "medium", "large-v3-turbo") with
//! optional ".en" variant selected by the language setting. Files live in either a
//! system-wide cache (`/usr/share/voice-to-text/models/`, populated by the Debian
//! postinst script) or a per-user cache (`~/.cache/voice-to-text/models/`). The
//! system cache is preferred when present so `apt install` works offline on first run.
//!
//! Each entry names the engine that runs it (ADR-0011): choosing a model
//! chooses its engine, so no setting can pair an engine with a model that is
//! not its own.

use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

/// Which speech engine a catalogue model runs on (ADR-0011).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKind {
    /// OpenAI Whisper through whisper.cpp (`WhisperEngine`).
    Whisper,
    /// NVIDIA Parakeet TDT through whisper.cpp's Parakeet API (`ParakeetEngine`).
    Parakeet,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelInfo {
    /// Menu-facing name (e.g. "small", "medium", "large-v3-turbo")
    pub name: &'static str,
    /// The engine that loads and runs this file.
    pub engine: EngineKind,
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
    /// Expected SHA-256, lowercase hex, sourced from the upstream HuggingFace
    /// git-lfs pointer file (`.../raw/main/<filename>` returns `oid sha256:...`)
    /// rather than a full download. `ensure()` hard-fails on mismatch (TASK-VTT112).
    pub sha256: &'static str,
    /// Credit written to the log when the file is downloaded, for models
    /// whose licence asks for attribution (Parakeet's CC-BY-4.0).
    pub attribution: Option<&'static str>,
}

/// Canonical catalogue of supported GGML models. Whisper URLs point at the
/// upstream ggerganov/whisper.cpp HuggingFace repo, the authoritative source;
/// Parakeet URLs at ggml-org/parakeet-GGUF, pinned to the revision the
/// sha256 was read from.
pub const MODELS: &[ModelInfo] = &[
    ModelInfo {
        name: "small.en",
        engine: EngineKind::Whisper,
        filename: "ggml-small.en.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin",
        size_mb: 466,
        multilingual: false,
        sha256: "c6138d6d58ecc8322097e0f987c32f1be8bb0a18532a3f88f734d1bbf9c41e5d",
        attribution: None,
    },
    ModelInfo {
        name: "small",
        engine: EngineKind::Whisper,
        filename: "ggml-small.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
        size_mb: 466,
        multilingual: true,
        sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        attribution: None,
    },
    ModelInfo {
        name: "medium.en",
        engine: EngineKind::Whisper,
        filename: "ggml-medium.en.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin",
        size_mb: 1462,
        multilingual: false,
        sha256: "cc37e93478338ec7700281a7ac30a10128929eb8f427dda2e865faa8f6da4356",
        attribution: None,
    },
    ModelInfo {
        name: "medium",
        engine: EngineKind::Whisper,
        filename: "ggml-medium.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
        size_mb: 1462,
        multilingual: true,
        sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        attribution: None,
    },
    ModelInfo {
        name: "large-v3-turbo",
        engine: EngineKind::Whisper,
        filename: "ggml-large-v3-turbo.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin",
        size_mb: 1536,
        multilingual: true,
        sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
        attribution: None,
    },
    ModelInfo {
        name: "large-v3",
        engine: EngineKind::Whisper,
        filename: "ggml-large-v3.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin",
        size_mb: 2963,
        multilingual: true,
        sha256: "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2",
        attribution: None,
    },
    // Parakeet TDT 0.6B v3 (ADR-0011): 25 European languages, detected
    // automatically, so there is one multilingual entry and no ".en" variant.
    ModelInfo {
        name: "parakeet-v3",
        engine: EngineKind::Parakeet,
        filename: "ggml-parakeet-tdt-0.6b-v3-q8_0.bin",
        url: "https://huggingface.co/ggml-org/parakeet-GGUF/resolve/35156454d1a39de06863303dd209fd2bed6ee079/ggml-parakeet-tdt-0.6b-v3-q8_0.bin",
        size_mb: 638,
        multilingual: true,
        sha256: "4d64e9e96c2792186d072fde0034df0ad670cf680a2f53069052ead827fd600e",
        attribution: Some(
            "Parakeet TDT 0.6B v3 by NVIDIA, licensed CC-BY-4.0 \
             (https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3); \
             ggml conversion by ggml-org (https://huggingface.co/ggml-org/parakeet-GGUF)",
        ),
    },
];

/// The Silero voice-activity model whisper.cpp's VAD reads, for Whisper
/// recordings over 30 s (ADR-0011). Not a selectable model, so not in
/// MODELS; it downloads and is checked through `ensure` like one.
pub const VAD_MODEL: ModelInfo = ModelInfo {
    name: "silero-vad-v6.2.0",
    engine: EngineKind::Whisper,
    filename: "ggml-silero-v6.2.0.bin",
    url: "https://huggingface.co/ggml-org/whisper-vad/resolve/9ffd54a1e1ee413ddf265af9913beaf518d1639b/ggml-silero-v6.2.0.bin",
    size_mb: 1,
    multilingual: true,
    sha256: "2aa269b785eeb53a82983a20501ddf7c1d9c48e33ab63a41391ac6c9f7fb6987",
    attribution: None,
};

/// Find a model by menu name. Returns None for unknown names so the caller can
/// fall back to a default.
pub fn find(name: &str) -> Option<&'static ModelInfo> {
    MODELS.iter().find(|m| m.name == name)
}

/// Resolve the model name to use given the menu selection and language mode.
/// For base names where an `.en` variant exists in MODELS (currently "small"
/// and "medium"), English mode picks it; multilingual mode picks the bare
/// name. For "large-v3-turbo" and "large-v3", upstream doesn't ship English-
/// only GGML variants, so we always return the base name.
///
/// Derives the has-`.en` check from MODELS rather than hardcoding — adding
/// a future `.en` variant to MODELS automatically wires it in here.
pub fn resolve_variant(menu_name: &str, language: &str) -> String {
    let base = menu_name.trim_end_matches(".en");
    let en_name = format!("{}.en", base);
    let has_en_variant = MODELS.iter().any(|m| m.name == en_name);
    if language == "en" && has_en_variant {
        en_name
    } else {
        base.to_string()
    }
}

/// System-wide cache path populated by postinst. Linux-only; other targets
/// have no such postinst step.
#[cfg(target_os = "linux")]
fn system_cache() -> Option<PathBuf> {
    Some(PathBuf::from("/usr/share/voice-to-text/models"))
}

#[cfg(not(target_os = "linux"))]
fn system_cache() -> Option<PathBuf> {
    None
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
    if let Some(sys) = system_cache().map(|p| p.join(filename)) {
        if sys.exists() {
            return sys;
        }
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
    if let Some(credit) = info.attribution {
        // The log, or stderr under `--file`, so the credit is seen either way.
        crate::whisper::note(&format!("Model credit: {credit}"));
    }

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

    let digest = format!("{:x}", hasher.finalize());
    crate::vtt_log!(
        "Downloaded {} ({} bytes, sha256: {})",
        info.filename,
        downloaded,
        digest
    );

    if !digest.eq_ignore_ascii_case(info.sha256) {
        fs::remove_file(&tmp).ok();
        anyhow::bail!(
            "SHA-256 mismatch for {}: expected {}, got {} — download discarded, not installed",
            info.filename,
            info.sha256,
            digest
        );
    }

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
            assert_eq!(
                m.sha256.len(),
                64,
                "model {} sha256 is not 64 hex chars: {}",
                m.name,
                m.sha256
            );
            assert!(
                m.sha256
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
                "model {} sha256 is not lowercase hex: {}",
                m.name,
                m.sha256
            );
        }
    }

    #[test]
    fn parakeet_is_one_multilingual_entry_on_its_own_engine() {
        let p = find("parakeet-v3").expect("parakeet-v3 in the catalogue");
        assert_eq!(p.engine, EngineKind::Parakeet);
        assert!(p.multilingual, "Parakeet v3 detects its own language");
        // No ".en" variant exists, so English mode must keep the same model.
        assert_eq!(resolve_variant("parakeet-v3", "en"), "parakeet-v3");
        assert_eq!(resolve_variant("parakeet-v3", "auto"), "parakeet-v3");
        // CC-BY-4.0 asks for credit; the download logs it.
        assert!(p.attribution.is_some_and(|a| a.contains("CC-BY-4.0")));
        // Pinned to a repository revision, not a moving branch.
        assert!(!p.url.contains("/resolve/main/"), "{}", p.url);
    }

    #[test]
    fn vad_model_is_pinned_and_not_selectable() {
        assert!(find(VAD_MODEL.name).is_none(), "VAD is not a menu model");
        assert!(
            !VAD_MODEL.url.contains("/resolve/main/"),
            "{}",
            VAD_MODEL.url
        );
        assert!(VAD_MODEL.url.ends_with(VAD_MODEL.filename));
        assert_eq!(VAD_MODEL.sha256.len(), 64);
    }

    #[test]
    fn whisper_entries_keep_the_whisper_engine() {
        for name in ["small", "small.en", "medium", "large-v3-turbo", "large-v3"] {
            assert_eq!(find(name).map(|m| m.engine), Some(EngineKind::Whisper));
        }
    }

    #[test]
    fn catalogue_models_have_unique_sha256() {
        let mut seen = std::collections::HashSet::new();
        for m in MODELS {
            assert!(
                seen.insert(m.sha256),
                "model {} shares a sha256 with another catalogue entry",
                m.name
            );
        }
    }
}
