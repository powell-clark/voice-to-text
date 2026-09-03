//! Opt-in archive of ordinary dictation as training-grade audio (TASK-VTT150).
//!
//! The push-to-talk path already produces exactly what a speech-synthesis
//! training corpus is made of — a recording and the text that was spoken — and
//! until now threw both away: audio into a twenty-file debug ring, text into
//! the typed output and nowhere else. This module keeps them, paired, at the
//! device's own sample rate.
//!
//! It is off unless `archive=1` is in `settings.conf`. Everything here is
//! reached only through that flag, so a default install writes nothing and
//! behaves exactly as it did before this module existed.
//!
//! Layout, one directory per day so a year of dictation stays browsable:
//!
//! ```text
//! <archive_dir>/2026-09-03/vtt_20260903T034812_123.wav   48 kHz 16-bit mono
//! <archive_dir>/2026-09-03/vtt_20260903T034812_123.json  the transcript + metadata
//! ```

use std::path::{Path, PathBuf};

/// One archived recording's metadata, written beside its wav as JSON.
///
/// `app` is the focused window's class at the moment of dictation where the
/// platform makes that cheap to know, and `None` otherwise — a corpus row that
/// knows it was dictated into a terminal is more useful than one that does not,
/// but not at the cost of a fragile X11 round-trip on the typing path.
#[derive(Debug, Clone, PartialEq)]
pub struct Sidecar {
    pub id: String,
    pub recorded_at: String,
    pub duration_s: f64,
    pub sample_rate: u32,
    pub text: String,
    pub model: String,
    pub language: String,
    pub app: Option<String>,
}

/// Escape a string for embedding in a JSON double-quoted value.
///
/// Hand-rolled because the sidecar is eight fields and this crate carries no
/// serde. Covers the two structural characters, the C0 controls that JSON
/// forbids raw, and the named short escapes. Transcripts are the risky input
/// here: they routinely contain quotes and, after a `[Truncated]` prefix or a
/// correction rule, backslashes.
pub fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for c in input.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Render a sidecar as pretty-printed JSON. Stable key order so a diff between
/// two archived recordings is readable.
pub fn sidecar_json(s: &Sidecar) -> String {
    let app = match &s.app {
        Some(a) => format!("\"{}\"", escape_json(a)),
        None => "null".to_string(),
    };
    format!(
        "{{\n  \"id\": \"{}\",\n  \"recorded_at\": \"{}\",\n  \"duration_s\": {:.3},\n  \"sample_rate\": {},\n  \"text\": \"{}\",\n  \"model\": \"{}\",\n  \"language\": \"{}\",\n  \"app\": {}\n}}\n",
        escape_json(&s.id),
        escape_json(&s.recorded_at),
        s.duration_s,
        s.sample_rate,
        escape_json(&s.text),
        escape_json(&s.model),
        escape_json(&s.language),
        app,
    )
}

/// Whether a finished, transcribed recording becomes a corpus row.
///
/// The settings gate is upstream of this — with archiving off there is no
/// native-rate buffer to offer in the first place. What this decides is the
/// second half: a recording whose transcript came back empty, or whitespace, is
/// audio with no paired text, which is exactly the thing a training corpus
/// cannot use. Whisper filler and punctuation-only output are already dropped
/// before this point, so an empty string here means the model heard nothing.
pub fn should_archive(text: &str) -> bool {
    !text.trim().is_empty()
}

/// Where the archive lives. An empty setting means the default beside the rest
/// of the app's data; a leading `~/` expands against the home directory so a
/// hand-edited `settings.conf` behaves the way its author expects.
pub fn resolve_archive_dir(setting: &str, config_dir: &Path) -> PathBuf {
    let trimmed = setting.trim();
    if trimmed.is_empty() {
        return config_dir.join("archive");
    }
    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(trimmed)
}

/// The wav and json paths for one recording: `<root>/<date>/vtt_<id>.{wav,json}`.
pub fn archive_targets(root: &Path, date: &str, id: &str) -> (PathBuf, PathBuf) {
    let dir = root.join(date);
    (
        dir.join(format!("vtt_{id}.wav")),
        dir.join(format!("vtt_{id}.json")),
    )
}

/// Peak value of a 24-bit signed sample.
const I24_MAX: f32 = 8_388_607.0;

/// Write 24-bit mono PCM at `sample_rate` to `path`, creating the parent
/// directory.
///
/// Separate from `audio::write_wav`, which targets the temp dir and is always
/// 16 kHz 16-bit for the debug ring. This one preserves the capture rate and
/// takes the wider word: the consumer is a speech-synthesis corpus, whose
/// conventions call for 24-bit, and the extra headroom costs 1.5x disk on a
/// capped directory. Cheap now, expensive once a corpus exists.
pub fn write_wav_at(path: &Path, samples: &[f32], sample_rate: u32) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for &s in samples {
        writer.write_sample((s.clamp(-1.0, 1.0) * I24_MAX) as i32)?;
    }
    writer.finalize()?;
    Ok(())
}

/// Archive one recording: the wav at its capture rate plus the sidecar.
///
/// The wav is written first. If the sidecar then fails, the orphaned wav is
/// removed rather than left behind — an audio file with no transcript is not
/// training data, and a corpus importer that has to guess which wavs are real
/// is a worse problem than a lost recording.
pub fn write_archive(
    root: &Path,
    date: &str,
    samples: &[f32],
    meta: &Sidecar,
) -> anyhow::Result<PathBuf> {
    let (wav_path, json_path) = archive_targets(root, date, &meta.id);
    write_wav_at(&wav_path, samples, meta.sample_rate)?;
    if let Err(e) = std::fs::write(&json_path, sidecar_json(meta)) {
        std::fs::remove_file(&wav_path).ok();
        return Err(e.into());
    }
    Ok(wav_path)
}

/// Choose which archived recordings to delete so at most `max` remain,
/// oldest first. `max == 0` means unbounded — the user asked for no pruning.
///
/// Pure and total: takes `(path, mtime)` pairs already gathered from disk, so
/// the selection rule is unit-testable without a filesystem.
pub fn select_for_prune<T: Ord + Copy>(entries: &[(PathBuf, T)], max: usize) -> Vec<PathBuf> {
    if max == 0 || entries.len() <= max {
        return Vec::new();
    }
    let mut sorted: Vec<&(PathBuf, T)> = entries.iter().collect();
    // Newest first, then everything past the cap is surplus. Ties break on the
    // path so the choice is deterministic when two recordings share an mtime.
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted[max..].iter().map(|(p, _)| p.clone()).collect()
}

/// Every archived wav across every dated directory, with its modification time.
pub fn collect_archived(root: &Path) -> Vec<(PathBuf, std::time::SystemTime)> {
    let mut out = Vec::new();
    let days = match std::fs::read_dir(root) {
        Ok(d) => d,
        Err(_) => return out,
    };
    for day in days.flatten() {
        if !day.path().is_dir() {
            continue;
        }
        let files = match std::fs::read_dir(day.path()) {
            Ok(f) => f,
            Err(_) => continue,
        };
        for f in files.flatten() {
            let path = f.path();
            if path.extension().and_then(|e| e.to_str()) != Some("wav") {
                continue;
            }
            if let Ok(mtime) = f.metadata().and_then(|m| m.modified()) {
                out.push((path, mtime));
            }
        }
    }
    out
}

/// Enforce the archive cap, deleting each surplus wav and its sidecar together
/// so the two never drift apart. Returns how many recordings were removed.
/// Empty dated directories are tidied away afterwards.
pub fn prune_archive(root: &Path, max: usize) -> usize {
    let entries = collect_archived(root);
    let doomed = select_for_prune(&entries, max);
    let mut removed = 0;
    for wav in &doomed {
        if std::fs::remove_file(wav).is_ok() {
            std::fs::remove_file(wav.with_extension("json")).ok();
            removed += 1;
        }
    }
    if removed > 0 {
        if let Ok(days) = std::fs::read_dir(root) {
            for day in days.flatten() {
                let p = day.path();
                let empty = std::fs::read_dir(&p)
                    .map(|mut d| d.next().is_none())
                    .unwrap_or(false);
                if p.is_dir() && empty {
                    std::fs::remove_dir(&p).ok();
                }
            }
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(id: &str, text: &str) -> Sidecar {
        Sidecar {
            id: id.to_string(),
            recorded_at: "2026-09-03T03:48:12.123+01:00".to_string(),
            duration_s: 4.25,
            sample_rate: 48_000,
            text: text.to_string(),
            model: "small.en".to_string(),
            language: "en".to_string(),
            app: None,
        }
    }

    #[test]
    fn escape_json_handles_quotes_and_backslashes() {
        assert_eq!(escape_json(r#"say "hi""#), r#"say \"hi\""#);
        assert_eq!(escape_json(r"C:\path"), r"C:\\path");
    }

    #[test]
    fn escape_json_handles_newlines_and_controls() {
        assert_eq!(escape_json("a\nb\tc\r"), "a\\nb\\tc\\r");
        assert_eq!(escape_json("\u{01}"), "\\u0001");
    }

    #[test]
    fn escape_json_leaves_unicode_alone() {
        // The £ and é that TASK-VTT055 fixed in the typing path must survive
        // into the corpus unmangled.
        assert_eq!(escape_json("£20 café"), "£20 café");
    }

    #[test]
    fn sidecar_json_carries_every_key() {
        let json = sidecar_json(&meta("20260903T034812_123", "Hello there."));
        for key in [
            "\"id\"",
            "\"recorded_at\"",
            "\"duration_s\"",
            "\"sample_rate\"",
            "\"text\"",
            "\"model\"",
            "\"language\"",
            "\"app\"",
        ] {
            assert!(json.contains(key), "sidecar is missing {key}: {json}");
        }
        assert!(json.contains("\"app\": null"), "absent app must be null");
        assert!(json.contains("\"sample_rate\": 48000"));
    }

    #[test]
    fn sidecar_json_quotes_a_present_app() {
        let mut m = meta("x", "hi");
        m.app = Some("gnome-terminal".into());
        assert!(sidecar_json(&m).contains("\"app\": \"gnome-terminal\""));
    }

    #[test]
    fn should_archive_rejects_empty_and_whitespace_transcripts() {
        assert!(should_archive("Hello there."));
        assert!(!should_archive(""), "no text means no corpus row");
        assert!(
            !should_archive("   \n\t "),
            "whitespace is not a transcript"
        );
    }

    #[test]
    fn resolve_archive_dir_defaults_beside_config() {
        let cfg = PathBuf::from("/home/u/.config/voice-to-text");
        assert_eq!(
            resolve_archive_dir("", &cfg),
            cfg.join("archive"),
            "an absent setting must not scatter audio somewhere surprising"
        );
        assert_eq!(resolve_archive_dir("   ", &cfg), cfg.join("archive"));
    }

    #[test]
    fn resolve_archive_dir_honours_an_absolute_path() {
        let cfg = PathBuf::from("/home/u/.config/voice-to-text");
        assert_eq!(
            resolve_archive_dir("/mnt/corpus", &cfg),
            PathBuf::from("/mnt/corpus")
        );
    }

    #[test]
    fn archive_targets_pairs_wav_and_json_in_a_dated_dir() {
        let (wav, json) = archive_targets(Path::new("/a"), "2026-09-03", "abc");
        assert_eq!(wav, PathBuf::from("/a/2026-09-03/vtt_abc.wav"));
        assert_eq!(json, PathBuf::from("/a/2026-09-03/vtt_abc.json"));
    }

    #[test]
    fn select_for_prune_keeps_the_newest_and_drops_the_rest() {
        let e: Vec<(PathBuf, u64)> = vec![
            (PathBuf::from("a.wav"), 1),
            (PathBuf::from("b.wav"), 2),
            (PathBuf::from("c.wav"), 3),
        ];
        assert_eq!(select_for_prune(&e, 2), vec![PathBuf::from("a.wav")]);
        assert_eq!(
            select_for_prune(&e, 1),
            vec![PathBuf::from("b.wav"), PathBuf::from("a.wav")],
            "surplus comes back oldest-last, newest-kept"
        );
    }

    #[test]
    fn select_for_prune_under_cap_deletes_nothing() {
        let e: Vec<(PathBuf, u64)> = vec![(PathBuf::from("a.wav"), 1)];
        assert!(select_for_prune(&e, 5).is_empty());
        assert!(select_for_prune::<u64>(&[], 5).is_empty());
    }

    #[test]
    fn select_for_prune_zero_cap_means_unbounded() {
        let e: Vec<(PathBuf, u64)> = vec![(PathBuf::from("a.wav"), 1)];
        assert!(
            select_for_prune(&e, 0).is_empty(),
            "0 means the user asked for no pruning, not delete everything"
        );
    }

    #[test]
    fn write_archive_lands_wav_and_sidecar_together() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let samples: Vec<f32> = (0..480).map(|i| (i as f32 / 480.0) - 0.5).collect();
        let m = meta("20260903T034812_123", "The quick brown fox.");

        let wav = write_archive(root, "2026-09-03", &samples, &m).expect("archive write");
        assert!(wav.exists(), "wav must land");
        let json = wav.with_extension("json");
        assert!(json.exists(), "sidecar must land beside it");

        let reader = hound::WavReader::open(&wav).expect("readable wav");
        assert_eq!(reader.spec().sample_rate, 48_000, "capture rate preserved");
        assert_eq!(reader.spec().channels, 1);
        assert_eq!(
            reader.spec().bits_per_sample,
            24,
            "the corpus consumer's conventions call for 24-bit; 16 would be              silently accepted and quietly worse"
        );

        let text = std::fs::read_to_string(&json).expect("readable sidecar");
        assert!(text.contains("The quick brown fox."));
    }

    #[test]
    fn archived_audio_round_trips_through_24_bit() {
        // A declared bit depth that silently truncates would be worse than
        // 16-bit honestly labelled, so check the samples survive.
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("t.wav");
        let input: Vec<f32> = (0..512)
            .map(|i| (i as f32 / 512.0 * std::f32::consts::TAU * 3.0).sin() * 0.5)
            .collect();
        write_wav_at(&path, &input, 48_000).expect("write");

        let mut reader = hound::WavReader::open(&path).expect("readable");
        let decoded: Vec<f32> = reader
            .samples::<i32>()
            .map(|s| s.expect("sample") as f32 / 8_388_607.0)
            .collect();
        assert_eq!(decoded.len(), input.len());
        for (a, b) in input.iter().zip(decoded.iter()) {
            assert!(
                (a - b).abs() < 1e-5,
                "24-bit quantisation should be far finer than this: {a} vs {b}"
            );
        }
    }

    #[test]
    fn prune_archive_removes_oldest_pairs_across_dated_dirs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let samples = vec![0.0f32; 16];
        for (date, id) in [
            ("2026-09-01", "a"),
            ("2026-09-02", "b"),
            ("2026-09-03", "c"),
        ] {
            let mut m = meta(id, "x");
            m.id = id.to_string();
            write_archive(root, date, &samples, &m).expect("archive write");
            // Distinct mtimes so oldest-first is unambiguous.
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert_eq!(collect_archived(root).len(), 3);

        let removed = prune_archive(root, 1);
        assert_eq!(removed, 2, "two surplus recordings should go");

        let left = collect_archived(root);
        assert_eq!(left.len(), 1, "the newest survives");
        assert!(left[0].0.ends_with("vtt_c.wav"));
        assert!(
            left[0].0.with_extension("json").exists(),
            "the surviving sidecar must not be collateral"
        );
    }
}
