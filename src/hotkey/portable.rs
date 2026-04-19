/// Portable hotkey monitor using rdev (macOS + Windows)
use super::{HotkeyCmd, KeyEvent};
use rdev::{self, EventType, Key};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;

/// Map our keycode (X11-style u8) to an rdev Key.
/// On macOS/Windows, we use a simpler key mapping:
/// 0 or 78 = ScrollLock (default), or map common keys.
fn keycode_to_rdev(keycode: u8) -> Key {
    // Default: Scroll Lock
    if keycode == 0 || keycode == 78 {
        return Key::ScrollLock;
    }
    // F-keys: X11 keycodes 67-76 = F1-F10, 95-96 = F11-F12
    match keycode {
        67 => Key::F1,
        68 => Key::F2,
        69 => Key::F3,
        70 => Key::F4,
        71 => Key::F5,
        72 => Key::F6,
        73 => Key::F7,
        74 => Key::F8,
        75 => Key::F9,
        76 => Key::F10,
        95 => Key::F11,
        96 => Key::F12,
        127 => Key::Pause,
        118 => Key::Insert,
        110 => Key::Home,
        115 => Key::End,
        112 => Key::PageUp,
        117 => Key::PageDown,
        66 => Key::CapsLock,
        77 => Key::NumLock,
        _ => Key::ScrollLock, // Fallback
    }
}

fn rdev_key_name(key: Key) -> &'static str {
    match key {
        Key::ScrollLock => "Scroll Lock",
        Key::CapsLock => "Caps Lock",
        Key::NumLock => "Num Lock",
        Key::Pause => "Pause",
        Key::PrintScreen => "Print Screen",
        Key::Insert => "Insert",
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "Page Up",
        Key::PageDown => "Page Down",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        _ => "Unknown",
    }
}

pub fn get_key_name(keycode: u8) -> String {
    let key = keycode_to_rdev(keycode);
    rdev_key_name(key).to_string()
}

pub fn start_monitor<F>(initial_keycode: u8, callback: F) -> anyhow::Result<mpsc::Sender<HotkeyCmd>>
where
    F: Fn(KeyEvent) + Send + 'static,
{
    let (cmd_tx, _cmd_rx) = mpsc::channel();
    let target = Arc::new(AtomicU32::new(initial_keycode as u32));

    let target_clone = target.clone();
    thread::Builder::new()
        .name("hotkey-monitor".into())
        .spawn(move || {
            let target_key = keycode_to_rdev(target_clone.load(Ordering::Relaxed) as u8);

            crate::vtt_log!("Hotkey monitor started ({})", rdev_key_name(target_key));

            // rdev::listen blocks forever
            if let Err(e) = rdev::listen(move |event| {
                let current_kc = target_clone.load(Ordering::Relaxed) as u8;
                let current_key = keycode_to_rdev(current_kc);

                match event.event_type {
                    EventType::KeyPress(key) if key == current_key => {
                        callback(KeyEvent::Down);
                    }
                    EventType::KeyRelease(key) if key == current_key => {
                        callback(KeyEvent::Up);
                    }
                    _ => {}
                }
            }) {
                crate::vtt_log!("Hotkey monitor error: {:?}", e);
            }
        })?;

    // Handle commands in a separate thread
    let target_for_cmd = target;
    let cmd_tx_clone = cmd_tx.clone();
    thread::Builder::new()
        .name("hotkey-cmd".into())
        .spawn(move || {
            while let Ok(cmd) = _cmd_rx.recv() {
                match cmd {
                    HotkeyCmd::SetKeycode(kc) => {
                        target_for_cmd.store(kc as u32, Ordering::Relaxed);
                        crate::vtt_log!("Hotkey changed to keycode {}", kc);
                    }
                    HotkeyCmd::Stop => {
                        crate::vtt_log!("Hotkey monitor stop requested");
                        break;
                    }
                }
            }
        })?;

    Ok(cmd_tx_clone)
}
