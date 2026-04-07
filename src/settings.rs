use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewlineType {
    PlainReturn = 0,
    ShiftReturn = 1,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub selected_model: String,
    pub selected_language: String,
    pub voice_prefix: String,
    pub initial_prompt: String,
    pub selected_device_index: i32,
    pub hotkey_keycode: u8,
    pub append_newline: bool,
    pub newline_type: NewlineType,
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
        let mut settings = Settings::default();
        settings.config_dir = config_dir.to_path_buf();

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

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
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
