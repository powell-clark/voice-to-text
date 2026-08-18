mod gate;
pub use gate::{Action, PushToTalk, MAX_HOLD};

/// True when this X11 keysym belongs to a key the user types with.
///
/// Push-to-talk grabs its key globally, so the grab swallows every press
/// before the focused window sees it. Bind an ordinary key and that key stops
/// working everywhere: binding space cost the operator every space character
/// they typed, and fired a recording on each one (TASK-VTT147).
///
/// Latin-1 keysyms 0x20..=0xFF are exactly the printable characters — space,
/// letters, digits, punctuation. The editing keys sit up in the function-key
/// range but are just as destructive to grab, so they are named explicitly.
pub fn is_typing_key(keysym: u32) -> bool {
    const RETURN: u32 = 0xFF0D;
    const KP_ENTER: u32 = 0xFF8D;
    const TAB: u32 = 0xFF09;
    const BACKSPACE: u32 = 0xFF08;
    const ESCAPE: u32 = 0xFF1B;
    const DELETE: u32 = 0xFFFF;

    (0x20..=0xFF).contains(&keysym)
        || matches!(
            keysym,
            RETURN | KP_ENTER | TAB | BACKSPACE | ESCAPE | DELETE
        )
}

#[cfg(test)]
mod typing_key_tests {
    use super::is_typing_key;

    #[test]
    fn space_is_a_typing_key() {
        // The one that started it: keycode 65 maps to XK_space.
        assert!(is_typing_key(0x020));
    }

    #[test]
    fn letters_digits_and_punctuation_are_typing_keys() {
        for keysym in [0x041, 0x061, 0x030, 0x039, 0x02C, 0x03B, 0x0FF] {
            assert!(
                is_typing_key(keysym),
                "keysym {keysym:#05x} should be typing"
            );
        }
    }

    #[test]
    fn editing_keys_are_typing_keys() {
        for keysym in [0xFF0D, 0xFF8D, 0xFF09, 0xFF08, 0xFF1B, 0xFFFF] {
            assert!(
                is_typing_key(keysym),
                "keysym {keysym:#06x} should be typing"
            );
        }
    }

    #[test]
    fn scroll_lock_and_friends_are_safe_to_bind() {
        // Scroll_Lock, Pause, Num_Lock, Caps_Lock, Insert, Home, Print.
        for keysym in [0xFF14, 0xFF13, 0xFF7F, 0xFFE5, 0xFF63, 0xFF50, 0xFF61] {
            assert!(
                !is_typing_key(keysym),
                "keysym {keysym:#06x} should be bindable"
            );
        }
    }

    #[test]
    fn function_keys_are_safe_to_bind() {
        for keysym in 0xFFBE..=0xFFC9 {
            assert!(
                !is_typing_key(keysym),
                "F-key {keysym:#06x} should be bindable"
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    Down,
    Up,
}

pub enum HotkeyCmd {
    SetKeycode(u8),
    Stop,
}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{get_key_name, keycode_is_typing, start_monitor};

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod portable;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use portable::start_monitor;
