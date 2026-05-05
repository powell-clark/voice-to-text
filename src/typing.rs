use crate::settings::NewlineType;
use enigo::{Direction, Enigo, Key, Keyboard, Settings as EnigoSettings};
use std::thread;
use std::time::Duration;

const INITIAL_DELAY_MS: u64 = 75;
const KEYSTROKE_DELAY_MS: u64 = 12;

/// Zero-sized "typer" handle. Holds no state because each call to
/// `type_text` creates a fresh `Enigo` instance — `Enigo` owns an X11
/// connection that isn't safe to share across threads, so stashing one
/// in the struct would complicate the worker thread handoff. The upfront
/// cost of `Enigo::new` is negligible (< 1 ms) relative to a
/// transcription, so per-call creation is simpler and correct.
pub struct Typer;

impl Typer {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Typer)
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

        let (typed, fallback_chars) = type_chars(&mut enigo, text, newline_type);

        // If any char could not be synthesised (rare — Key::Unicode handles £, é, —, etc.
        // but may fail on some emoji / CJK on X11), fall back to clipboard paste for
        // just those chars. Ctrl+V is unreliable in many TUIs, so only use as last resort.
        if !fallback_chars.is_empty() {
            crate::vtt_log!(
                "Typing fallback: {} chars via clipboard paste",
                fallback_chars.chars().count()
            );
            paste_text(&mut enigo, &fallback_chars);
        }

        crate::vtt_log!("Typing completed ({} chars typed directly)", typed);
    }
}

/// Type every character via `Key::Unicode`, which enigo's X11 backend maps to a
/// temporary keysym — this handles £, é, —, and most Latin-extended characters.
/// Returns (chars typed directly, chars that need clipboard fallback).
fn type_chars(enigo: &mut Enigo, text: &str, newline_type: NewlineType) -> (usize, String) {
    let mut typed = 0usize;
    let mut fallback = String::new();

    for c in text.chars() {
        let result = if c == '\n' {
            match newline_type {
                NewlineType::ShiftReturn => {
                    enigo.key(Key::Shift, Direction::Press).ok();
                    let r = enigo.key(Key::Return, Direction::Click);
                    enigo.key(Key::Shift, Direction::Release).ok();
                    r
                }
                NewlineType::PlainReturn => enigo.key(Key::Return, Direction::Click),
            }
        } else if c == '\t' {
            enigo.key(Key::Tab, Direction::Click)
        } else {
            enigo.key(Key::Unicode(c), Direction::Click)
        };

        if result.is_ok() {
            typed += 1;
        } else {
            fallback.push(c);
        }
        thread::sleep(Duration::from_millis(KEYSTROKE_DELAY_MS));
    }

    (typed, fallback)
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
