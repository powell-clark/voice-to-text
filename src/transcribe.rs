use std::path::Path;
use std::process::Command;

/// Detect if model uses whisper.cpp backend (W prefix) vs CTranslate2 (CT2 prefix)
fn is_whisper_cpp(model: &str) -> bool {
    !model.starts_with("CT2 ")
}

/// Auto-append .en suffix for English models that have an English-only variant
fn adjust_model_for_language(model: &str, language: &str) -> String {
    if language != "en" || model.contains(".en") {
        return model.to_string();
    }
    let has_en = ["tiny", "base", "small", "medium"]
        .iter()
        .any(|m| model.contains(m));
    if has_en {
        let adjusted = format!("{}.en", model);
        crate::vtt_log!("Auto-selected .en model: {} (English mode)", adjusted);
        adjusted
    } else {
        model.to_string()
    }
}

pub fn transcribe_audio(
    audio_path: &Path,
    model: &str,
    language: &str,
    initial_prompt: &str,
) -> Option<String> {
    let model = if model.is_empty() { "CT2 small" } else { model };
    let language = if language.is_empty() { "en" } else { language };
    let adjusted = adjust_model_for_language(model, language);

    let output = if is_whisper_cpp(&adjusted) {
        transcribe_whisper_cpp(audio_path, &adjusted, language, initial_prompt)
    } else {
        transcribe_ct2(audio_path, &adjusted, language, initial_prompt)
    };

    match output {
        Ok(text) => {
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                crate::vtt_log!("Empty transcription result");
                None
            } else {
                Some(trimmed)
            }
        }
        Err(e) => {
            crate::vtt_log!("Transcription failed: {}", e);
            None
        }
    }
}

fn transcribe_whisper_cpp(
    audio_path: &Path,
    model: &str,
    language: &str,
    initial_prompt: &str,
) -> anyhow::Result<String> {
    // Find whisper-cli binary
    let whisper_cli = [
        "/usr/bin/whisper-cli",
        "/usr/local/bin/whisper-cli",
        "./third_party/whisper.cpp/build/bin/whisper-cli",
    ]
    .iter()
    .find(|p| Path::new(p).exists())
    .ok_or_else(|| anyhow::anyhow!("whisper-cli not found"))?;

    crate::vtt_log!("Using whisper-cli: {}", whisper_cli);

    // Strip "W " prefix
    let base_model = model.strip_prefix("W ").unwrap_or(model);
    let is_english_only = base_model.contains(".en");

    // Get clean model name (without .en suffix for file path)
    let clean_name = if is_english_only {
        base_model.strip_suffix(".en").unwrap_or(base_model)
    } else {
        base_model
    };
    let file_model = if clean_name == "large" {
        "large-v3"
    } else {
        clean_name
    };
    let extension = if is_english_only { ".en.bin" } else { ".bin" };

    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let model_file = format!("{}/.cache/whisper/ggml-{}{}", home, file_model, extension);

    if !Path::new(&model_file).exists() {
        anyhow::bail!("Model file not found: {}", model_file);
    }

    let mut cmd = Command::new(whisper_cli);
    cmd.args(["-m", &model_file])
        .args(["-f", &audio_path.to_string_lossy()])
        .args(["--no-timestamps"])
        .args(["--language", language])
        .args(["--threads", "4"])
        .stderr(std::process::Stdio::null());

    if !initial_prompt.is_empty() {
        cmd.args(["--prompt", initial_prompt]);
    }

    let output = cmd.output()?;
    if !output.status.success() {
        anyhow::bail!(
            "whisper-cli exited with status {}",
            output.status.code().unwrap_or(-1)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn transcribe_ct2(
    audio_path: &Path,
    model: &str,
    language: &str,
    initial_prompt: &str,
) -> anyhow::Result<String> {
    // Find transcribe.py script
    let script = [
        "/usr/share/voice-to-text/transcribe.py",
        "./src/common/transcribe.py",
        "src/common/transcribe.py",
    ]
    .iter()
    .find(|p| Path::new(p).exists())
    .ok_or_else(|| anyhow::anyhow!("transcribe.py not found"))?;

    crate::vtt_log!("Using transcribe.py: {}", script);

    // Strip "CT2 " prefix
    let base_model = model.strip_prefix("CT2 ").unwrap_or(model);

    // Map model names
    let ct2_model = match base_model {
        "large" => "large-v3".to_string(),
        "distil-large-v3" => "distil-whisper/distil-large-v3".to_string(),
        "distil-large-v3.5" => "distil-whisper/distil-large-v3.5-ct2".to_string(),
        other => other.to_string(),
    };

    let mut cmd = Command::new("python3");
    cmd.arg(script)
        .arg(audio_path.to_string_lossy().as_ref())
        .arg(&ct2_model)
        .arg(language)
        .stderr(std::process::Stdio::null());

    if !initial_prompt.is_empty() {
        cmd.arg(initial_prompt);
    }

    let output = cmd.output()?;
    if !output.status.success() {
        anyhow::bail!(
            "transcribe.py exited with status {}",
            output.status.code().unwrap_or(-1)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
