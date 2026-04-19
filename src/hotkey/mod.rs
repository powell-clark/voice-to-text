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
pub use linux::{get_key_name, start_monitor};

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod portable;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use portable::{get_key_name, start_monitor};
