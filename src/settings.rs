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
    /// cpal device index. -1 means "use default". Read from settings.conf
    /// but not yet consumed by `audio::Audio::new()` — see TASK-VTT050 track.
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
    config_dir: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            selected_model: "small".into(),
            selected_language: "en".into(),
            voice_prefix: "[Voice] ".into(),
            initial_prompt: String::new(),
            selected_device_index: -1,
            hotkey_keycode: 0, // 0 = use default Scroll Lock
            append_newline: true,
            newline_type: NewlineType::ShiftReturn,
            logging_enabled: true,
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
        out.push_str(&format!("device={}\n", self.selected_device_index));
        if self.hotkey_keycode >= 8 {
            out.push_str(&format!("hotkey={}\n", self.hotkey_keycode));
        }
        out.push_str(&format!(
            "newline={}\n",
            if self.append_newline { 1 } else { 0 }
        ));
        out.push_str(&format!("newline_type={}\n", self.newline_type as i32));

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
            selected_device_index: 3,
            hotkey_keycode: 78,
            append_newline: false,
            newline_type: NewlineType::PlainReturn,
            logging_enabled: false,
            config_dir: dir.path().to_path_buf(),
        };
        original.save().expect("save should succeed");

        let loaded = Settings::load(dir.path());
        assert_eq!(loaded.selected_model, original.selected_model);
        assert_eq!(loaded.selected_language, original.selected_language);
        assert_eq!(loaded.voice_prefix, original.voice_prefix);
        assert_eq!(loaded.initial_prompt, original.initial_prompt);
        assert_eq!(loaded.selected_device_index, original.selected_device_index);
        assert_eq!(loaded.hotkey_keycode, original.hotkey_keycode);
        assert_eq!(loaded.append_newline, original.append_newline);
        assert_eq!(loaded.newline_type, original.newline_type);
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
