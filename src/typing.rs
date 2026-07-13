use crate::settings::NewlineType;
use enigo::{Direction, Enigo, Key, Keyboard, Settings as EnigoSettings};
#[cfg(not(target_os = "windows"))]
use std::process::Command;
use std::thread;
use std::time::Duration;

const INITIAL_DELAY_MS: u64 = 75;
/// Delay between keystrokes passed to xdotool. 12ms matches xdotool's own
/// default and is reliable across terminals without XKB remapping overhead.
#[cfg(not(target_os = "windows"))]
const XDOTOOL_DELAY_MS: u64 = 12;
/// Fallback delay used only when xdotool is unavailable and enigo takes over.
#[cfg(not(target_os = "windows"))]
const ENIGO_DELAY_MS: u64 = 20;

/// Zero-sized "typer" handle. On Linux, delegates to xdotool which uses
/// XKeysymToKeycode (no XKB remapping) plus --clearmodifiers to reset stale
/// modifier state. Falls back to enigo if xdotool is not installed.
pub struct Typer;

impl Typer {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Typer)
    }

    pub fn type_text(&self, text: &str, newline_type: NewlineType) {
        crate::vtt_log!("Typing {} bytes", text.len());

        thread::sleep(Duration::from_millis(INITIAL_DELAY_MS));

        // Windows: type the whole string in one SendInput batch via enigo.text().
        // The char-by-char path dropped leading/upper-case characters and
        // reordered them through the clipboard fallback (TASK-VTT092); xdotool is
        // Linux-only so probing for it here is pointless.
        #[cfg(target_os = "windows")]
        {
            type_via_enigo_text(text, newline_type);
        }

        #[cfg(not(target_os = "windows"))]
        {
            if xdotool_type(text, newline_type) {
                crate::vtt_log!("Typing completed via xdotool");
                return;
            }

            // xdotool unavailable — fall back to enigo
            crate::vtt_log!("xdotool unavailable, falling back to enigo");
            let mut enigo = match Enigo::new(&EnigoSettings::default()) {
                Ok(e) => e,
                Err(e) => {
                    crate::vtt_log!("Failed to create Enigo instance: {:?}", e);
                    return;
                }
            };
            let (typed, fallback_chars) = enigo_type_chars(&mut enigo, text, newline_type);
            if !fallback_chars.is_empty() {
                paste_text(&mut enigo, &fallback_chars);
            }
            crate::vtt_log!("Typing completed via enigo ({} chars)", typed);
        }
    }
}

/// Windows typing: one `enigo.text()` batch per newline-delimited segment, with
/// the configured newline key (Shift+Return vs Return) between segments. Batched
/// SendInput avoids the per-character drops the char-by-char path produced.
#[cfg(target_os = "windows")]
fn type_via_enigo_text(text: &str, newline_type: NewlineType) {
    let mut enigo = match Enigo::new(&EnigoSettings::default()) {
        Ok(e) => e,
        Err(e) => {
            crate::vtt_log!("Failed to create Enigo instance: {:?}", e);
            return;
        }
    };
    let segments: Vec<&str> = text.split('\n').collect();
    for (i, segment) in segments.iter().enumerate() {
        if !segment.is_empty() {
            if let Err(e) = enigo.text(segment) {
                crate::vtt_log!("enigo.text failed: {:?}", e);
            }
        }
        if i < segments.len() - 1 {
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
        }
    }

    // Also place the text on the clipboard so Ctrl+V works as a fallback — parity
    // with the Linux xclip path (FEAT-VTT012 / TASK-VTT099).
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(text.to_string());
    }
    crate::vtt_log!("Typing completed via enigo.text ({} bytes)", text.len());
}

/// Type text via xdotool subprocess. Splits on newlines and fires
/// `xdotool key shift+Return` (or plain Return) between segments.
/// Returns false if xdotool is not installed so the caller can fall back.
#[cfg(not(target_os = "windows"))]
fn xdotool_type(text: &str, newline_type: NewlineType) -> bool {
    let segments: Vec<&str> = text.split('\n').collect();

    for (i, segment) in segments.iter().enumerate() {
        if !segment.is_empty() {
            let result = Command::new("xdotool")
                .arg("type")
                .arg("--delay")
                .arg(XDOTOOL_DELAY_MS.to_string())
                .arg("--clearmodifiers")
                .arg("--")
                .arg(segment)
                .status();

            match result {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    crate::vtt_log!("xdotool not found: {}", e);
                    return false;
                }
                Err(e) => {
                    crate::vtt_log!("xdotool error: {}", e);
                    return false;
                }
                Ok(s) if !s.success() => {
                    crate::vtt_log!("xdotool exited with: {}", s);
                    return false;
                }
                Ok(_) => {}
            }
        }

        if i < segments.len() - 1 {
            let key = match newline_type {
                NewlineType::ShiftReturn => "shift+Return",
                NewlineType::PlainReturn => "Return",
            };
            let _ = Command::new("xdotool").args(["key", key]).status();
        }
    }

    true
}

#[cfg(not(target_os = "windows"))]
fn enigo_type_chars(enigo: &mut Enigo, text: &str, newline_type: NewlineType) -> (usize, String) {
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
        thread::sleep(Duration::from_millis(ENIGO_DELAY_MS));
    }

    (typed, fallback)
}

#[cfg(not(target_os = "windows"))]
fn paste_text(enigo: &mut Enigo, text: &str) {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            if let Err(e) = clipboard.set_text(text) {
                crate::vtt_log!("Failed to set clipboard: {}", e);
                return;
            }
        }
        Err(e) => {
            crate::vtt_log!("Failed to open clipboard: {}", e);
            paste_via_xclip(text);
            simulate_paste_shortcut(enigo);
            return;
        }
    }

    thread::sleep(Duration::from_millis(100));
    simulate_paste_shortcut(enigo);
    thread::sleep(Duration::from_millis(10));
}

/// Places `text` on the system clipboard without simulating a paste
/// keystroke — used by the tray "Copy last transcription" recovery action
/// (FEAT-VTT038), where the user picks the destination window afterwards.
pub fn set_clipboard_text(text: &str) {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            if let Err(e) = clipboard.set_text(text.to_string()) {
                crate::vtt_log!("Failed to set clipboard: {}", e);
                #[cfg(not(target_os = "windows"))]
                paste_via_xclip(text);
            }
        }
        Err(e) => {
            crate::vtt_log!("Failed to open clipboard: {}", e);
            #[cfg(not(target_os = "windows"))]
            paste_via_xclip(text);
        }
    }
}

/// The paste modifier: Cmd+V on macOS, Ctrl+V everywhere else this fn is
/// compiled for (Linux; Windows has its own paste_text above). Using the
/// wrong modifier on macOS opens Mission Control / does nothing instead of
/// pasting (TASK-VTT114).
#[cfg(target_os = "macos")]
const PASTE_MODIFIER: Key = Key::Command;
#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
const PASTE_MODIFIER: Key = Key::Control;

#[cfg(not(target_os = "windows"))]
fn simulate_paste_shortcut(enigo: &mut Enigo) {
    enigo.key(PASTE_MODIFIER, Direction::Press).ok();
    thread::sleep(Duration::from_millis(1));
    enigo.key(Key::Unicode('v'), Direction::Click).ok();
    thread::sleep(Duration::from_millis(1));
    enigo.key(PASTE_MODIFIER, Direction::Release).ok();
}

#[cfg(not(target_os = "windows"))]
fn paste_via_xclip(text: &str) {
    use std::io::Write;
    use std::process::Stdio;

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
