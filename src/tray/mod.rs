// The shared `Set*` prefix predates this variant and is used at 44 call
// sites across both tray implementations — renaming all three to drop it
// is a real cleanup but out of scope for the task that added this one
// variant (TASK-VTT054).
#[allow(clippy::enum_variant_names)]
pub enum UiMessage {
    SetStatus(String),
    SetIcon(String),
    /// Persistent backend label (TASK-VTT054) — distinct from SetStatus,
    /// which cycles through Ready/Recording/Transcribing and would
    /// overwrite a transient "Backend: CT2" the user could never actually
    /// read. Sent once at startup and again if the CT2 daemon dies
    /// mid-session (falls back to native without user intervention).
    SetBackendLabel(String),
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
