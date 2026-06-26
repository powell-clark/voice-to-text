/// Portable hotkey monitor using rdev (macOS + Windows)
use super::{HotkeyCmd, KeyEvent};
use rdev::{self, EventType, Key};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keycode_to_rdev_default_and_explicit_scrolllock() {
        // 0 means "use the default" which is Scroll Lock.
        assert_eq!(keycode_to_rdev(0), Key::ScrollLock);
        assert_eq!(keycode_to_rdev(78), Key::ScrollLock);
    }

    #[test]
    fn keycode_to_rdev_maps_f_keys() {
        assert_eq!(keycode_to_rdev(67), Key::F1);
        assert_eq!(keycode_to_rdev(72), Key::F6);
        assert_eq!(keycode_to_rdev(76), Key::F10);
        assert_eq!(keycode_to_rdev(95), Key::F11);
        assert_eq!(keycode_to_rdev(96), Key::F12);
    }

    #[test]
    fn keycode_to_rdev_maps_navigation_and_lock_keys() {
        assert_eq!(keycode_to_rdev(127), Key::Pause);
        assert_eq!(keycode_to_rdev(118), Key::Insert);
        assert_eq!(keycode_to_rdev(110), Key::Home);
        assert_eq!(keycode_to_rdev(115), Key::End);
        assert_eq!(keycode_to_rdev(112), Key::PageUp);
        assert_eq!(keycode_to_rdev(117), Key::PageDown);
        assert_eq!(keycode_to_rdev(66), Key::CapsLock);
        assert_eq!(keycode_to_rdev(77), Key::NumLock);
    }

    #[test]
    fn keycode_to_rdev_unknown_falls_back_to_scrolllock() {
        // Defensive: any keycode we haven't explicitly mapped should fall back
        // to the default hotkey rather than silently dropping key events.
        assert_eq!(keycode_to_rdev(5), Key::ScrollLock);
        assert_eq!(keycode_to_rdev(200), Key::ScrollLock);
        assert_eq!(keycode_to_rdev(255), Key::ScrollLock);
    }

    #[test]
    fn rdev_key_name_covers_all_mapped_keycodes() {
        // Round-trip: every keycode that maps to a specific rdev key must also
        // have a human-readable label — otherwise the tray's hotkey menu shows
        // "Unknown" for keys the user actively selected.
        let mapped = [
            0, 78, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 95, 96, 127, 118, 110, 115, 112, 117,
            66, 77,
        ];
        for kc in mapped {
            let name = get_key_name(kc);
            assert_ne!(
                name, "Unknown",
                "keycode {} maps to a key with no human-readable name",
                kc
            );
        }
    }
}

pub fn start_monitor<F>(initial_keycode: u8, callback: F) -> anyhow::Result<mpsc::Sender<HotkeyCmd>>
where
    F: Fn(KeyEvent) + Send + 'static,
{
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let target = Arc::new(AtomicU32::new(initial_keycode as u32));

    let target_clone = target.clone();
    thread::Builder::new()
        .name("hotkey-monitor".into())
        .spawn(move || {
            let target_key = keycode_to_rdev(target_clone.load(Ordering::Relaxed) as u8);

            crate::vtt_log!("Hotkey monitor started ({})", rdev_key_name(target_key));

            // Suppress OS key auto-repeat: while a key is held the OS emits
            // repeated KeyPress events. Track pressed state so only the first
            // press fires Down and only a real release fires Up — parity with the
            // Linux X11 auto-repeat filter (FEAT-VTT013 / TASK-VTT100).
            let pressed = AtomicBool::new(false);

            // rdev::listen blocks forever
            if let Err(e) = rdev::listen(move |event| {
                let current_kc = target_clone.load(Ordering::Relaxed) as u8;
                let current_key = keycode_to_rdev(current_kc);

                match event.event_type {
                    EventType::KeyPress(key) if key == current_key => {
                        if !pressed.swap(true, Ordering::Relaxed) {
                            callback(KeyEvent::Down);
                        }
                    }
                    EventType::KeyRelease(key) if key == current_key => {
                        if pressed.swap(false, Ordering::Relaxed) {
                            callback(KeyEvent::Up);
                        }
                    }
                    _ => {}
                }
            }) {
                crate::vtt_log!("Hotkey monitor error: {:?}", e);
            }
        })?;

    // Handle commands in a separate thread
    let target_for_cmd = target;
    thread::Builder::new()
        .name("hotkey-cmd".into())
        .spawn(move || {
            while let Ok(cmd) = cmd_rx.recv() {
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

    Ok(cmd_tx)
}
