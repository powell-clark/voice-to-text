pub enum UiMessage {
    SetStatus(String),
    SetIcon(String),
    /// A newer release exists on GitHub (TASK-VTT095). Carries the version
    /// tag and the release page URL; the tray surfaces this as a menu item
    /// that opens the URL. Informational only — nothing is downloaded.
    UpdateAvailable(String, String),
}

/// Platform-agnostic UI sender (uses mpsc on all platforms)
pub type UiSender = std::sync::mpsc::Sender<UiMessage>;

/// Most recent successful transcription's exact typed text (including any
/// `[Truncated] ` prefix), set by the worker thread and read by the tray's
/// "Copy last transcription" menu item (FEAT-VTT038). `None` until the first
/// transcription completes this run.
pub type LastTranscription = std::sync::Arc<std::sync::Mutex<Option<String>>>;

mod quit;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::Tray;

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod portable;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use portable::Tray;
