use crate::settings::NewlineType;
use enigo::{Direction, Enigo, Key, Keyboard, Settings as EnigoSettings};
use std::thread;
use std::time::Duration;

const INITIAL_DELAY_MS: u64 = 75;
const KEYSTROKE_DELAY_MS: u64 = 4;

pub struct Typer {
    // Enigo is created per-use since it holds X11 connection state
    // that shouldn't be shared across threads
}

impl Typer {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Typer {})
    }

    pub fn type_text(&self, text: &str, newline_type: NewlineType) {
        let mut enigo = match Enigo::new(&EnigoSettings::default()) {
            Ok(e) => e,
            Err(e) => {
                crate::vtt_log!("Failed to create Enigo instance: {:?}", e);
                return;
            }
        };

        crate::vtt_log!("Typing {} bytes", text.len());

        // Initial delay before first keystroke
        thread::sleep(Duration::from_millis(INITIAL_DELAY_MS));

        // Find first non-ASCII character
        let first_non_ascii = text.find(|c: char| !c.is_ascii());

        match first_non_ascii {
            None => {
                // All ASCII — type character by character for natural appearance
                type_ascii(&mut enigo, text, newline_type);
            }
            Some(0) => {
                // Starts with non-ASCII — clipboard paste everything
                paste_text(&mut enigo, text);
            }
            Some(pos) => {
                // Mixed: type ASCII prefix, paste the rest
                type_ascii(&mut enigo, &text[..pos], newline_type);
                paste_text(&mut enigo, &text[pos..]);
            }
        }

        crate::vtt_log!("Typing completed");
    }
}

fn type_ascii(enigo: &mut Enigo, text: &str, newline_type: NewlineType) {
    for c in text.chars() {
        if c == '\n' {
            match newline_type {
                NewlineType::ShiftReturn => {
                    enigo.key(Key::Shift, Direction::Press).ok();
                    enigo.key(Key::Return, Direction::Click).ok();
                    enigo.key(Key::Shift, Direction::Release).ok();
                }
                NewlineType::PlainReturn => {
                    enigo.key(Key::Return, Direction::Click).ok();
                }
            }
        } else if c == '\t' {
            enigo.key(Key::Tab, Direction::Click).ok();
        } else {
            enigo.key(Key::Unicode(c), Direction::Click).ok();
        }
        thread::sleep(Duration::from_millis(KEYSTROKE_DELAY_MS));
    }
}

fn paste_text(enigo: &mut Enigo, text: &str) {
    // Set clipboard via arboard
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            if let Err(e) = clipboard.set_text(text) {
                crate::vtt_log!("Failed to set clipboard: {}", e);
                return;
            }
        }
        Err(e) => {
            crate::vtt_log!("Failed to open clipboard: {}", e);
            // Fallback: try xclip subprocess
            paste_via_xclip(text);
            simulate_ctrl_v(enigo);
            return;
        }
    }

    thread::sleep(Duration::from_millis(100)); // Let clipboard settle

    // Simulate Ctrl+V
    simulate_ctrl_v(enigo);
    thread::sleep(Duration::from_millis(10));
}

fn simulate_ctrl_v(enigo: &mut Enigo) {
    enigo.key(Key::Control, Direction::Press).ok();
    thread::sleep(Duration::from_millis(1));
    enigo.key(Key::Unicode('v'), Direction::Click).ok();
    thread::sleep(Duration::from_millis(1));
    enigo.key(Key::Control, Direction::Release).ok();
}

fn paste_via_xclip(text: &str) {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let child = Command::new("xclip")
        .args(["-selection", "clipboard", "-i"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match child {
        Ok(mut child) => {
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(text.as_bytes()).ok();
            }
            child.wait().ok();
        }
        Err(_) => {
            // Try xsel as fallback
            let child = Command::new("xsel")
                .args(["--clipboard", "--input"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            if let Ok(mut child) = child {
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(text.as_bytes()).ok();
                }
                child.wait().ok();
            }
        }
    }
}
