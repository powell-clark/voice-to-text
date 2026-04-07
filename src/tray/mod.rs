pub enum UiMessage {
    SetStatus(String),
    SetIcon(String),
}

/// Platform-agnostic UI sender (uses mpsc on all platforms)
pub type UiSender = std::sync::mpsc::Sender<UiMessage>;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::Tray;

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod portable;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use portable::Tray;
