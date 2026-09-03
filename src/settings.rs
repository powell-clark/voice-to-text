use std::fs;
use std::path::{Path, PathBuf};

/// How to send a `\n` inside a transcription. Many chat apps use plain Return
/// to send and Shift+Return for a new line, so ShiftReturn is the default.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewlineType {
    /// Plain Return — suitable for editors where Return means "new line".
    PlainReturn = 0,
    /// Shift+Return — suitable for chat apps where Return means "send".
    ShiftReturn = 1,
}

/// All user-tunable state, persisted to `settings.conf` in the config dir.
/// Loaded once at startup via `Settings::load(&config_dir)`, then shared
/// across threads via `Arc<RwLock<Settings>>`. The tray mutates through the
/// RwLock and calls `save()` to persist.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Menu-facing model name, e.g. "small", "large-v3-turbo". May include
    /// an ".en" suffix for English-only variants. See `models::MODELS`.
    pub selected_model: String,
    /// BCP-47-ish language code ("en" or "auto" in practice). "auto" means
    /// "let whisper detect" and selects the multilingual model variant.
    pub selected_language: String,
    /// String prepended to every transcription before typing (e.g. "[Voice] ").
    /// Also used as the `starts_with` check to suppress double-prefixing when
    /// whisper echoes the prompt back.
    pub voice_prefix: String,
    /// Whisper's `initial_prompt` — helps the model recognise technical
    /// vocabulary, names, or style specific to the user. Capped at 240 chars
    /// by the tray dialog; longer values from hand-edited settings.conf are
    /// silently truncated by whisper-rs.
    pub initial_prompt: String,
    /// User-editable list of `(misheard, correct)` pairs (FEAT-VTT037),
    /// applied as a deterministic whole-word/phrase substitution pass after
    /// Whisper transcribes, before the text is typed. Complements
    /// `initial_prompt`: the prompt biases inference, this guarantees a fix
    /// for a specific known mishearing regardless of how Whisper hears it.
    pub corrections: Vec<(String, String)>,
    /// cpal device index. -1 means "use default". Read from settings.conf
    /// and saved back, but not yet consumed by `audio::Audio::new()` which
    /// currently always uses the default input device. Wiring is tracked
    /// as TASK-VTT062 (multi-mic support via tray menu).
    pub selected_device_index: i32,
    /// X11 keycode of the push-to-talk key. 0 means "use Scroll Lock".
    /// Valid range: 8..=255 (enforced by the loader — anything else falls
    /// back to the default).
    pub hotkey_keycode: u8,
    /// If true, insert a newline before typing the next transcription.
    /// Only applies after the first transcription of a session.
    pub append_newline: bool,
    /// Plain vs Shift-qualified return. See `NewlineType`.
    pub newline_type: NewlineType,
    /// If false, `vtt_log!` becomes a no-op (daily log file is not written
    /// or created). Set via the tray menu, persisted to settings.conf.
    pub logging_enabled: bool,
    /// One-shot marker: has the first-run autostart default been applied yet?
    /// On first launch this is false, so VTT enables start-at-login by default
    /// (TASK-VTT109); thereafter the tray "Start at login" toggle is the sole
    /// control and we never re-enable behind the user's back. Persisted so the
    /// default fires exactly once, even across upgrades.
    pub autostart_initialized: bool,
    /// Opt-in, off by default: when true, every recording is additionally
    /// captured at the device's native sample rate (typically 48 kHz, versus
    /// the 16 kHz Whisper always uses) and archived to `archive_dir` with a
    /// JSON sidecar carrying its transcript. Off, capture and behaviour are
    /// byte-identical to a build with no archiving code at all — this flag
    /// gates the only place native-rate samples are ever read or written to
    /// disk. See README "Archiving your recordings" before enabling it: it
    /// saves your voice and what you said to disk indefinitely.
    /// Off by default. The card asked for default-on against fan noise, but
    /// the operator has no fan any more and the measured before/after was
    /// mixed rather than a win — 7 of 12 recordings unchanged, the rest
    /// changed on marginal audio in both directions. Shipping it on would
    /// change daily dictation for no demonstrated gain, so it is opt-in:
    /// `denoise=1` enables it. See `denoise.rs` for the measured profile.
    pub denoise: bool,
    pub archive_recordings: bool,
    /// Directory recordings are archived into when `archive_recordings` is
    /// true. Empty string means the default, `<config_dir>/archive`.
    pub archive_dir: String,
    /// Oldest-first cap on the number of archived recordings (wav+json pairs
    /// count as one), applied after every archive write. 0 means unbounded —
    /// the user has explicitly asked for no pruning.
    pub archive_max_files: usize,
    config_dir: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            selected_model: "small".into(),
            selected_language: "en".into(),
            voice_prefix: "[Voice] ".into(),
            initial_prompt: String::new(),
            corrections: Vec::new(),
            selected_device_index: -1,
            hotkey_keycode: 0, // 0 = use default Scroll Lock
            append_newline: true,
            newline_type: NewlineType::ShiftReturn,
            logging_enabled: true,
            autostart_initialized: false,
            denoise: false,
            archive_recordings: false,
            archive_dir: String::new(),
            archive_max_files: 5000,
            config_dir: PathBuf::new(),
        }
    }
}

impl Settings {
    pub fn load(config_dir: &Path) -> Self {
        let mut settings = Settings {
            config_dir: config_dir.to_path_buf(),
            ..Settings::default()
        };

        let path = config_dir.join("settings.conf");
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return settings,
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, raw_value)) = line.split_once('=') {
                let value = strip_quotes(raw_value.trim());
                let value = unescape(&value);

                match key.trim() {
                    "model" => settings.selected_model = value,
                    "language" => settings.selected_language = value,
                    "prefix" => settings.voice_prefix = value,
                    "prompt" => settings.initial_prompt = value,
                    "correction" => {
                        if let Some((from, to)) = value.split_once("=>") {
                            settings
                                .corrections
                                .push((from.trim().to_string(), to.trim().to_string()));
                        }
                    }
                    "device" => {
                        settings.selected_device_index = value.parse().unwrap_or(-1);
                    }
                    "hotkey" => {
                        let v: i32 = value.parse().unwrap_or(0);
                        if (8..=255).contains(&v) {
                            settings.hotkey_keycode = v as u8;
                        }
                    }
                    "newline" => settings.append_newline = value != "0",
                    "newline_type" => {
                        settings.newline_type = if value == "0" {
                            NewlineType::PlainReturn
                        } else {
                            NewlineType::ShiftReturn
                        };
                    }
                    "autostart_init" => settings.autostart_initialized = value == "1",
                    "denoise" => settings.denoise = value == "1",
                    "archive" => settings.archive_recordings = value == "1",
                    "archive_dir" => settings.archive_dir = value,
                    "archive_max_files" => {
                        settings.archive_max_files = value.parse().unwrap_or(5000);
                    }
                    _ => {}
                }
            }
        }
        settings
    }

    pub fn save(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        let path = self.config_dir.join("settings.conf");

        let mut out = String::with_capacity(512);
        out.push_str("# Voice to Text Settings\n");
        out.push_str("# Auto-generated - edit at your own risk\n\n");
        out.push_str(&format!("model={}\n", self.selected_model));
        out.push_str(&format!("language={}\n", self.selected_language));
        out.push_str(&format!("prefix=\"{}\"\n", escape(&self.voice_prefix)));
        out.push_str(&format!("prompt=\"{}\"\n", escape(&self.initial_prompt)));
        out.push_str("# correction=\"misheard=>correct\" — repeat this line for each word/phrase Whisper reliably mishears\n");
        for (from, to) in &self.corrections {
            out.push_str(&format!(
                "correction=\"{}=>{}\"\n",
                escape(from),
                escape(to)
            ));
        }
        out.push_str(&format!("device={}\n", self.selected_device_index));
        if self.hotkey_keycode >= 8 {
            out.push_str(&format!("hotkey={}\n", self.hotkey_keycode));
        }
        out.push_str(&format!(
            "newline={}\n",
            if self.append_newline { 1 } else { 0 }
        ));
        out.push_str(&format!("newline_type={}\n", self.newline_type as i32));
        out.push_str(&format!(
            "autostart_init={}\n",
            if self.autostart_initialized { 1 } else { 0 }
        ));
        out.push_str(
            "# denoise=1 high-passes low-frequency rumble out of the audio sent for\n\
             # transcription. Off by default. Archived audio is never filtered.\n",
        );
        out.push_str(&format!("denoise={}\n", if self.denoise { 1 } else { 0 }));
        out.push_str(
            "# archive=1 saves every recording (native sample rate) plus its transcript to\n\
             # archive_dir indefinitely, capped at archive_max_files. Off by default. See\n\
             # README \"Archiving your recordings\" before turning this on.\n",
        );
        out.push_str(&format!(
            "archive={}\n",
            if self.archive_recordings { 1 } else { 0 }
        ));
        if !self.archive_dir.is_empty() {
            out.push_str(&format!("archive_dir=\"{}\"\n", escape(&self.archive_dir)));
        }
        out.push_str(&format!("archive_max_files={}\n", self.archive_max_files));

        fs::write(&path, out)?;
        Ok(())
    }
}

fn strip_quotes(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn strip_quotes_handles_unquoted_plain_quoted_and_one_char() {
        assert_eq!(strip_quotes(""), "");
        assert_eq!(strip_quotes("plain"), "plain");
        assert_eq!(strip_quotes("\"quoted\""), "quoted");
        assert_eq!(strip_quotes("\""), "\"", "single quote is not a pair");
        assert_eq!(strip_quotes("\"\""), "", "empty pair becomes empty string");
    }

    #[test]
    fn escape_and_unescape_roundtrip_preserves_input() {
        let inputs = [
            "",
            "plain text",
            "with \"quotes\" in it",
            "back\\slash",
            "line1\nline2",
            "mixed \"\\\n end",
            "£ and é and — non-ASCII preserved",
        ];
        for s in &inputs {
            let round_tripped = unescape(&escape(s));
            assert_eq!(
                &round_tripped,
                s,
                "round-trip failed for {:?} -> {:?} -> {:?}",
                s,
                escape(s),
                round_tripped
            );
        }
    }

    #[test]
    fn unescape_of_trailing_backslash_is_literal_backslash() {
        assert_eq!(unescape("foo\\"), "foo\\");
    }

    #[test]
    fn settings_roundtrip_via_tempdir_preserves_all_fields() {
        let dir = tempdir().unwrap();
        let original = Settings {
            selected_model: "large-v3-turbo".into(),
            selected_language: "auto".into(),
            voice_prefix: "[Speech] ".into(),
            initial_prompt: "transcribe accurately".into(),
            corrections: vec![
                ("ard".into(), "odd".into()),
                ("amala vajrayana".into(), "Amala Vijnana".into()),
            ],
            selected_device_index: 3,
            hotkey_keycode: 78,
            append_newline: false,
            newline_type: NewlineType::PlainReturn,
            logging_enabled: false,
            autostart_initialized: true,
            denoise: false,
            archive_recordings: true,
            archive_dir: "/mnt/voice-archive".into(),
            archive_max_files: 12345,
            config_dir: dir.path().to_path_buf(),
        };
        original.save().expect("save should succeed");

        let loaded = Settings::load(dir.path());
        assert_eq!(loaded.selected_model, original.selected_model);
        assert_eq!(loaded.selected_language, original.selected_language);
        assert_eq!(loaded.voice_prefix, original.voice_prefix);
        assert_eq!(loaded.initial_prompt, original.initial_prompt);
        assert_eq!(loaded.corrections, original.corrections);
        assert_eq!(loaded.selected_device_index, original.selected_device_index);
        assert_eq!(loaded.hotkey_keycode, original.hotkey_keycode);
        assert_eq!(loaded.append_newline, original.append_newline);
        assert_eq!(loaded.newline_type, original.newline_type);
        assert_eq!(
            loaded.autostart_initialized, original.autostart_initialized,
            "autostart_init marker must survive a save/load round-trip"
        );
        assert_eq!(
            loaded.denoise, original.denoise,
            "denoise defaults on, so a saved 0 must survive or the setting is unusable"
        );
        assert_eq!(loaded.archive_recordings, original.archive_recordings);
        assert_eq!(loaded.archive_dir, original.archive_dir);
        assert_eq!(loaded.archive_max_files, original.archive_max_files);
    }

    #[test]
    fn archive_settings_default_off_and_survive_a_missing_file() {
        let dir = tempdir().unwrap();
        let s = Settings::load(dir.path());
        assert!(
            !s.archive_recordings,
            "archiving must default to off — it is opt-in"
        );
        assert_eq!(s.archive_dir, "");
        assert_eq!(s.archive_max_files, 5000);
    }

    #[test]
    fn a_pre_archive_settings_file_keeps_archiving_off() {
        // The upgrade case: someone running v2.3.11 has a settings.conf with
        // none of the three archive keys. Reading it must leave archiving off
        // and the cap at its default, so the upgrade changes nothing on disk
        // until they opt in.
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("settings.conf"),
            "hotkey=70\nmodel=\"small.en\"\nlanguage=\"en\"\nlogging=1\n",
        )
        .unwrap();
        let s = Settings::load(dir.path());
        assert!(
            !s.archive_recordings,
            "an upgrade must not start recording someone's voice to disk"
        );
        assert_eq!(s.archive_dir, "");
        assert_eq!(s.archive_max_files, 5000);
    }

    #[test]
    fn archive_dir_omitted_from_settings_conf_when_empty() {
        let dir = tempdir().unwrap();
        let s = Settings {
            config_dir: dir.path().to_path_buf(),
            ..Settings::default()
        };
        s.save().unwrap();
        let content = std::fs::read_to_string(dir.path().join("settings.conf")).unwrap();
        assert!(
            !content.contains("archive_dir="),
            "an empty archive_dir means \"use the default\" and should not be written, \
             so a fresh install's settings.conf stays free of a stale absolute path"
        );
        assert!(content.contains("archive=0"));
    }

    #[test]
    fn archive_max_files_zero_means_unbounded_and_round_trips() {
        let dir = tempdir().unwrap();
        let s = Settings {
            archive_max_files: 0,
            config_dir: dir.path().to_path_buf(),
            ..Settings::default()
        };
        s.save().unwrap();
        assert_eq!(Settings::load(dir.path()).archive_max_files, 0);
    }

    #[test]
    fn autostart_init_defaults_false_then_persists_true() {
        let dir = tempdir().unwrap();
        // Missing file → first run → marker is false so the default can fire.
        assert!(!Settings::load(dir.path()).autostart_initialized);

        let s = Settings {
            autostart_initialized: true,
            config_dir: dir.path().to_path_buf(),
            ..Settings::default()
        };
        s.save().unwrap();
        assert!(
            Settings::load(dir.path()).autostart_initialized,
            "once applied, the marker must stick so the default never re-fires"
        );
    }

    #[test]
    fn settings_load_returns_defaults_when_file_missing() {
        let dir = tempdir().unwrap();
        let s = Settings::load(dir.path());
        let d = Settings::default();
        assert_eq!(s.selected_model, d.selected_model);
        assert_eq!(s.selected_language, d.selected_language);
        assert_eq!(s.hotkey_keycode, d.hotkey_keycode);
    }

    #[test]
    fn settings_load_ignores_comments_and_blank_lines() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("settings.conf");
        std::fs::write(
            &path,
            "# comment line\n\n   \nmodel=medium\n# trailing comment\nlanguage=en\n",
        )
        .unwrap();
        let s = Settings::load(dir.path());
        assert_eq!(s.selected_model, "medium");
        assert_eq!(s.selected_language, "en");
    }

    #[test]
    fn settings_parses_multiple_correction_lines_in_order() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("settings.conf"),
            "correction=\"ard=>odd\"\ncorrection=\"amala vajrayana=>Amala Vijnana\"\n",
        )
        .unwrap();
        let s = Settings::load(dir.path());
        assert_eq!(
            s.corrections,
            vec![
                ("ard".to_string(), "odd".to_string()),
                ("amala vajrayana".to_string(), "Amala Vijnana".to_string())
            ]
        );
    }

    #[test]
    fn settings_ignores_correction_line_missing_arrow() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("settings.conf"),
            "correction=\"no arrow here\"\n",
        )
        .unwrap();
        let s = Settings::load(dir.path());
        assert!(s.corrections.is_empty());
    }

    #[test]
    fn settings_conf_documents_the_correction_line_format() {
        let dir = tempdir().unwrap();
        let s = Settings {
            config_dir: dir.path().to_path_buf(),
            ..Settings::default()
        };
        s.save().unwrap();
        let content = std::fs::read_to_string(dir.path().join("settings.conf")).unwrap();
        assert!(
            content.contains("correction=\"misheard=>correct\""),
            "settings.conf should self-document the correction line format for discoverability"
        );
    }

    #[test]
    fn settings_hotkey_out_of_range_keeps_default() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("settings.conf"), "hotkey=300\n").unwrap();
        let s = Settings::load(dir.path());
        assert_eq!(
            s.hotkey_keycode, 0,
            "300 is out of u8 XKB keycode range 8..=255, should be ignored"
        );

        std::fs::write(dir.path().join("settings.conf"), "hotkey=5\n").unwrap();
        let s = Settings::load(dir.path());
        assert_eq!(
            s.hotkey_keycode, 0,
            "5 is below the XKB keycode floor of 8, should be ignored"
        );

        // Boundary: 8 and 255 are both inside the accepted range.
        std::fs::write(dir.path().join("settings.conf"), "hotkey=8\n").unwrap();
        assert_eq!(Settings::load(dir.path()).hotkey_keycode, 8);

        std::fs::write(dir.path().join("settings.conf"), "hotkey=255\n").unwrap();
        assert_eq!(Settings::load(dir.path()).hotkey_keycode, 255);

        // Boundary: 256 is above u8's max, should be ignored.
        std::fs::write(dir.path().join("settings.conf"), "hotkey=256\n").unwrap();
        assert_eq!(Settings::load(dir.path()).hotkey_keycode, 0);

        // Malformed: non-numeric should fall back to default 0.
        std::fs::write(dir.path().join("settings.conf"), "hotkey=notanumber\n").unwrap();
        assert_eq!(Settings::load(dir.path()).hotkey_keycode, 0);
    }

    #[test]
    fn settings_escape_preserves_unicode_in_voice_prefix() {
        let dir = tempdir().unwrap();
        let original = Settings {
            voice_prefix: "£ € — naïve".into(),
            config_dir: dir.path().to_path_buf(),
            ..Settings::default()
        };
        original.save().unwrap();
        let loaded = Settings::load(dir.path());
        assert_eq!(loaded.voice_prefix, "£ € — naïve");
    }
}
