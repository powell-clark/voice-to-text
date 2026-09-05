/// Portable tray implementation using tray-icon + muda (macOS + Windows)
use super::{LastTranscription, UiMessage, UiSender};
use crate::logging;
use crate::settings::Settings;
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};

use muda::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{TrayIcon, TrayIconBuilder};

pub struct Tray {
    tray_icon: TrayIcon,
    cmd_rx: mpsc::Receiver<MenuCmd>,
    ui_rx: mpsc::Receiver<UiMessage>,
    ids: MenuIds,
    settings: Arc<RwLock<Settings>>,
    last_transcription: LastTranscription,
    work_tx: mpsc::Sender<crate::WorkItem>,
    /// TASK-VTT095. Set when UiMessage::UpdateAvailable arrives; MenuCmd::OpenUpdate
    /// reads it to know where to open. NOT independently verified on Windows/macOS —
    /// see the `update` field comment below for why.
    update_url: Option<String>,
    /// TASK-VTT098. The disabled explanatory row currently shown in the Logs
    /// submenu ("(no logs yet)" / an unreadable-directory reason), or None when
    /// the submenu is listing real files.
    logs_placeholder: Option<String>,
    /// When the log directory was last listed. poll_menu runs at ~10 Hz and
    /// listing a directory that often would be pure waste, so the check is
    /// throttled to LOGS_REFRESH_INTERVAL.
    logs_checked_at: std::time::Instant,
}

/// How often poll_menu re-lists the log directory (TASK-VTT098). Matches the
/// cadence the Linux tray uses for its own periodic menu refresh.
const LOGS_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(3);

struct MenuIds {
    status: MenuItem,
    quit: MenuItem,
    about: MenuItem,
    logging: MenuItem,
    autostart: Option<CheckMenuItem>,
    copy_last: MenuItem,
    retranscribe: MenuItem,
    models: Vec<(CheckMenuItem, String)>,
    lang_en: CheckMenuItem,
    lang_multi: CheckMenuItem,
    /// TASK-VTT054. NOT independently verified on Windows/macOS — written by
    /// analogy to the `status` item beside it (same MenuItem type, same
    /// `.set_text()` update pattern), but this module only compiles on those
    /// two targets, neither of which this dev machine can build for.
    backend: MenuItem,
    /// TASK-VTT095. Starts disabled ("Up to date") since muda::MenuItem has
    /// no visibility toggle (unlike GTK's hide()/show() used on Linux);
    /// UiMessage::UpdateAvailable re-labels it and enables it for a click.
    /// NOT independently verified — see `backend` above for why.
    update: MenuItem,
    /// Logs submenu (TASK-VTT098, parity with the Linux tray's FEAT-VTT004
    /// AC-4). Held so poll_menu can refresh its contents — muda has no
    /// "menu about to open" callback, which is what the GTK side uses.
    logs: Submenu,
    /// The log files currently listed in `logs`, paired with the menu id of
    /// the item that opens each. Rebuilt only when the file list actually
    /// changes, so the tray is not rebuilding a menu at poll frequency.
    log_items: Vec<(muda::MenuId, std::path::PathBuf)>,
}

// Commands from the event-matching thread (Send-safe) to the main thread.
enum MenuCmd {
    Quit,
    About,
    LoggingToggle,
    AutostartToggle,
    CopyLastTranscription,
    RetranscribeLast,
    LanguageSel(String),
    ModelSel(String),
    OpenUpdate,
    /// A menu id the event thread could not match — resolved on the main
    /// thread, which owns the dynamically rebuilt Logs items (TASK-VTT098).
    Unmatched(muda::MenuId),
}

impl Tray {
    pub fn new(
        settings: Arc<RwLock<Settings>>,
        _data_dir: &Path,
        last_transcription: LastTranscription,
        work_tx: mpsc::Sender<crate::WorkItem>,
    ) -> anyhow::Result<(Self, UiSender)> {
        let s = settings.read().unwrap();
        let current_model = s.selected_model.clone();
        let current_lang = s.selected_language.clone();
        let logging_enabled = s.logging_enabled;
        let backend_enabled = s.backend == "ct2";
        drop(s);

        // Build menu
        let menu = Menu::new();

        let status = MenuItem::new("Status: Initializing...", false, None);
        menu.append(&status)?;
        menu.append(&PredefinedMenuItem::separator())?;

        // Language
        let lang_en =
            CheckMenuItem::new("English only (fastest)", true, current_lang == "en", None);
        let lang_multi = CheckMenuItem::new(
            "Multilingual (99 languages)",
            true,
            current_lang != "en",
            None,
        );
        let lang_sub = Submenu::new(
            &format!(
                "Language: {}",
                if current_lang == "en" {
                    "English only"
                } else {
                    "Multilingual"
                }
            ),
            true,
        );
        lang_sub.append(&lang_en)?;
        lang_sub.append(&lang_multi)?;
        menu.append(&lang_sub)?;

        // Models — generated from the real catalogue so selections resolve via
        // models::find() (TASK-VTT086). The old hardcoded W*/CT2* names were
        // pre-v2.0 backend labels that no longer match anything.
        let model_sub = Submenu::new(&format!("Model: {}", current_model), true);
        let mut models = Vec::new();
        for info in crate::models::MODELS {
            let checked = info.name == current_model;
            let item = CheckMenuItem::new(info.name, true, checked, None);
            model_sub.append(&item)?;
            models.push((item, info.name.to_string()));
        }
        menu.append(&model_sub)?;
        menu.append(&PredefinedMenuItem::separator())?;

        // Backend (info label, TASK-VTT054)
        let backend = MenuItem::new(
            if backend_enabled {
                "Backend: CT2 (starting...)"
            } else {
                "Backend: Native"
            },
            false,
            None,
        );
        menu.append(&backend)?;

        // Logs submenu — parity with the Linux tray (TASK-VTT098, FEAT-VTT004
        // AC-4). Contents are filled in by refresh_logs_submenu below and
        // refreshed from poll_menu, because muda has no "menu about to open"
        // callback of the kind the GTK side hangs its rebuild on.
        let logs = Submenu::new("Logs", true);
        menu.append(&logs)?;

        // Logging
        let logging = MenuItem::new(
            if logging_enabled {
                "Logging: On"
            } else {
                "Logging: Off"
            },
            true,
            None,
        );
        menu.append(&logging)?;

        // Start at login — Windows only (Linux uses systemd --user, macOS uses a
        // LaunchAgent; TASK-VTT094). Shown only where the toggle persists.
        let autostart = if crate::autostart::SUPPORTED {
            let item =
                CheckMenuItem::new("Start at login", true, crate::autostart::is_enabled(), None);
            menu.append(&item)?;
            Some(item)
        } else {
            None
        };

        // Copy last transcription — recovery net (FEAT-VTT038)
        let copy_last = MenuItem::new("Copy last transcription", true, None);
        menu.append(&copy_last)?;

        // Re-transcribe last recording — recovery net (FEAT-VTT039)
        let retranscribe = MenuItem::new("Re-transcribe last recording", true, None);
        menu.append(&retranscribe)?;

        // Update available (TASK-VTT095) — starts disabled; UpdateAvailable
        // re-labels and enables it once a check finds a newer release.
        let update = MenuItem::new("Up to date", false, None);
        menu.append(&update)?;

        // About
        let about = MenuItem::new("About Voice to Text", true, None);
        menu.append(&about)?;
        menu.append(&PredefinedMenuItem::separator())?;

        // Quit
        let quit = MenuItem::new("Quit", true, None);
        menu.append(&quit)?;

        // Create tray icon
        let icon = create_icon(0, 180, 0);
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Voice to Text")
            .with_icon(icon)
            .build()?;

        let ids = MenuIds {
            status,
            quit,
            about,
            logging,
            autostart,
            copy_last,
            retranscribe,
            models,
            lang_en,
            lang_multi,
            backend,
            update,
            logs,
            log_items: Vec::new(),
        };

        // muda::MenuId wraps String so it is Clone + Send.
        // The MenuItem/CheckMenuItem/Submenu objects contain Rc<MenuId> and are NOT Send.
        // Solution: clone just the IDs into the event-matching thread; keep the items
        // in the Tray struct so poll_menu() can call set_checked/set_text on the main thread.
        let quit_id = ids.quit.id().clone();
        let about_id = ids.about.id().clone();
        let logging_id = ids.logging.id().clone();
        let autostart_id = ids.autostart.as_ref().map(|i| i.id().clone());
        let copy_last_id = ids.copy_last.id().clone();
        let retranscribe_id = ids.retranscribe.id().clone();
        let lang_en_id = ids.lang_en.id().clone();
        let lang_multi_id = ids.lang_multi.id().clone();
        let update_id = ids.update.id().clone();
        let model_ids: Vec<(muda::MenuId, String)> = ids
            .models
            .iter()
            .map(|(item, name)| (item.id().clone(), name.clone()))
            .collect();

        let (cmd_tx, cmd_rx) = mpsc::channel::<MenuCmd>();
        let menu_event_rx = MenuEvent::receiver().clone();

        std::thread::Builder::new()
            .name("menu-events".into())
            .spawn(move || loop {
                if let Ok(event) = menu_event_rx.recv() {
                    let id = event.id().clone();
                    let cmd = if id == quit_id {
                        Some(MenuCmd::Quit)
                    } else if id == about_id {
                        Some(MenuCmd::About)
                    } else if id == logging_id {
                        Some(MenuCmd::LoggingToggle)
                    } else if autostart_id.as_ref() == Some(&id) {
                        Some(MenuCmd::AutostartToggle)
                    } else if id == copy_last_id {
                        Some(MenuCmd::CopyLastTranscription)
                    } else if id == retranscribe_id {
                        Some(MenuCmd::RetranscribeLast)
                    } else if id == lang_en_id {
                        Some(MenuCmd::LanguageSel("en".into()))
                    } else if id == lang_multi_id {
                        Some(MenuCmd::LanguageSel("auto".into()))
                    } else if id == update_id {
                        Some(MenuCmd::OpenUpdate)
                    } else {
                        model_ids
                            .iter()
                            .find(|(mid, _)| *mid == id)
                            .map(|(_, name)| MenuCmd::ModelSel(name.clone()))
                            // Log items are created after this thread captured
                            // its id snapshot and are replaced whenever the file
                            // list changes, so they cannot be matched here.
                            // Forward the raw id; poll_menu owns that list and
                            // resolves it on the main thread (TASK-VTT098).
                            .or(Some(MenuCmd::Unmatched(id.clone())))
                    };
                    if let Some(cmd) = cmd {
                        if cmd_tx.send(cmd).is_err() {
                            break;
                        }
                    }
                }
            })?;

        // UI updates (status text + icon colour) are applied on the main thread
        // in poll_menu — tray-icon/muda handles are !Send, so the worker thread
        // cannot touch them. Keep the receiver in the struct rather than spawning
        // a thread that could only log.
        let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>();

        Ok((
            Tray {
                tray_icon,
                cmd_rx,
                ui_rx,
                ids,
                settings,
                last_transcription,
                work_tx,
                update_url: None,
                logs_placeholder: None,
                // Zero-initialised so the first poll_menu populates the submenu
                // immediately rather than showing an empty Logs menu for the
                // first few seconds.
                logs_checked_at: std::time::Instant::now() - LOGS_REFRESH_INTERVAL,
            },
            ui_tx,
        ))
    }

    /// Rebuild the Logs submenu when the set of log files has changed.
    ///
    /// Cheap on the common path: it lists a small directory and returns
    /// immediately unless the filenames differ from what is already shown, so
    /// the menu is not reconstructed at poll frequency (TASK-VTT098).
    fn refresh_logs_submenu(&mut self) {
        if self.logs_checked_at.elapsed() < LOGS_REFRESH_INTERVAL {
            return;
        }
        self.logs_checked_at = std::time::Instant::now();

        let files = match logging::list_log_filenames() {
            Ok(f) => f,
            // An unreadable log directory is shown in the menu rather than
            // logged, since the menu is where the user is looking.
            Err(e) => {
                let msg = format!("(log dir unreadable: {e})");
                if self.logs_placeholder.as_deref() != Some(&msg) {
                    self.set_logs_items(&[], Some(&msg));
                    self.logs_placeholder = Some(msg);
                }
                return;
            }
        };

        let log_dir = logging::get_dir();
        let unchanged = files.len() == self.ids.log_items.len()
            && files
                .iter()
                .zip(self.ids.log_items.iter())
                .all(|(name, (_, p))| p.file_name().map(|f| f == name.as_str()).unwrap_or(false));
        if unchanged && self.logs_placeholder.is_none() {
            return;
        }

        if files.is_empty() {
            self.set_logs_items(&[], Some("(no logs yet)"));
            self.logs_placeholder = Some("(no logs yet)".into());
            return;
        }

        let (today, yesterday) = logging::today_and_yesterday();
        let entries: Vec<(String, std::path::PathBuf)> = files
            .iter()
            .map(|name| {
                (
                    logging::format_log_label(name, &today, &yesterday),
                    log_dir.join(name),
                )
            })
            .collect();
        self.set_logs_items(&entries, None);
        self.logs_placeholder = None;
    }

    /// Replace the Logs submenu's contents. `placeholder` renders a single
    /// disabled explanatory row instead of file entries.
    fn set_logs_items(
        &mut self,
        entries: &[(String, std::path::PathBuf)],
        placeholder: Option<&str>,
    ) {
        while !self.ids.logs.items().is_empty() {
            self.ids.logs.remove_at(0);
        }
        self.ids.log_items.clear();

        if let Some(text) = placeholder {
            let item = MenuItem::new(text, false, None);
            let _ = self.ids.logs.append(&item);
            return;
        }

        for (label, path) in entries {
            let item = MenuItem::new(label, true, None);
            if self.ids.logs.append(&item).is_ok() {
                self.ids.log_items.push((item.id().clone(), path.clone()));
            }
        }
    }

    /// Process pending menu commands. Call from the main event loop at ~10 Hz.
    pub fn poll_menu(&mut self) {
        self.refresh_logs_submenu();
        // Apply queued UI updates on the main thread — tray-icon handles are
        // !Send so the worker cannot do this. Status → tooltip, state → icon
        // colour (idle green, recording red, processing amber).
        while let Ok(msg) = self.ui_rx.try_recv() {
            match msg {
                UiMessage::SetStatus(text) => {
                    // Update the menu's Status line (previously frozen at
                    // "Initializing...") and the hover tooltip.
                    self.ids.status.set_text(format!("Status: {text}"));
                    let _ = self
                        .tray_icon
                        .set_tooltip(Some(format!("Voice to Text — {text}")));
                }
                UiMessage::SetIcon(state) => {
                    let (r, g, b) = match state.as_str() {
                        "recording" => (220u8, 40u8, 40u8),
                        "processing" => (230u8, 160u8, 0u8),
                        _ => (0u8, 180u8, 0u8), // ready / idle
                    };
                    let _ = self.tray_icon.set_icon(Some(create_icon(r, g, b)));
                }
                UiMessage::SetBackendLabel(label) => {
                    self.ids.backend.set_text(format!("Backend: {label}"));
                }
                UiMessage::UpdateAvailable(version, url) => {
                    self.ids
                        .update
                        .set_text(format!("Update available: {version}"));
                    self.ids.update.set_enabled(true);
                    self.update_url = Some(url);
                }
            }
        }

        while let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                MenuCmd::Quit => {
                    super::quit::quit();
                }
                MenuCmd::About => {
                    crate::vtt_log!(
                        "About: Voice to Text — https://github.com/powell-clark/voice-to-text"
                    );
                    // A real, visible window with selectable text (TASK-VTT139),
                    // parity with the Linux tray's show_about_dialog. Windows
                    // only for now — macOS has no dialog toolkit wired up yet
                    // (tray-icon/muda ships none) and stays parked per this
                    // task's own scope note; the log line above is its fallback.
                    #[cfg(target_os = "windows")]
                    show_about_messagebox();
                }
                MenuCmd::LoggingToggle => {
                    let mut s = self.settings.write().unwrap();
                    s.logging_enabled = !s.logging_enabled;
                    logging::set_enabled(s.logging_enabled);
                    self.ids.logging.set_text(if s.logging_enabled {
                        "Logging: On"
                    } else {
                        "Logging: Off"
                    });
                    crate::vtt_log!(
                        "Logging {}",
                        if s.logging_enabled {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    );
                }
                MenuCmd::AutostartToggle => match crate::autostart::toggle() {
                    Ok(enabled) => {
                        if let Some(item) = &self.ids.autostart {
                            item.set_checked(enabled);
                        }
                        crate::vtt_log!(
                            "Autostart {}",
                            if enabled { "enabled" } else { "disabled" }
                        );
                    }
                    Err(e) => crate::vtt_log!("Autostart toggle failed: {}", e),
                },
                MenuCmd::CopyLastTranscription => {
                    let text = self.last_transcription.lock().unwrap().clone();
                    match text {
                        Some(text) => {
                            crate::typing::set_clipboard_text(&text);
                            crate::vtt_log!(
                                "Copied last transcription to clipboard ({} bytes)",
                                text.len()
                            );
                        }
                        None => crate::vtt_log!(
                            "Copy last transcription: nothing transcribed yet this run"
                        ),
                    }
                }
                MenuCmd::RetranscribeLast => {
                    // Ask the worker to re-run whisper on the newest archived
                    // WAV and re-type it (FEAT-VTT039). The worker locates and
                    // decodes the file and handles the empty-dir no-op.
                    if self
                        .work_tx
                        .send(crate::WorkItem::RetranscribeLast)
                        .is_err()
                    {
                        crate::vtt_log!("Re-transcribe: worker channel closed");
                    }
                }
                MenuCmd::LanguageSel(lang) => {
                    let is_en = lang == "en";
                    self.settings.write().unwrap().selected_language = lang;
                    self.ids.lang_en.set_checked(is_en);
                    self.ids.lang_multi.set_checked(!is_en);
                    self.settings.read().unwrap().save().ok();
                    crate::vtt_log!(
                        "Language changed to {}",
                        if is_en { "English" } else { "Multilingual" }
                    );
                }
                MenuCmd::ModelSel(name) => {
                    self.settings.write().unwrap().selected_model = name.clone();
                    for (item, item_name) in &self.ids.models {
                        item.set_checked(*item_name == name);
                    }
                    self.settings.read().unwrap().save().ok();
                    crate::vtt_log!("Model changed to {}", name);
                }
                MenuCmd::OpenUpdate => {
                    if let Some(url) = &self.update_url {
                        open_url(url);
                    }
                }
                MenuCmd::Unmatched(id) => {
                    // Only the Logs items are unmatched by design; anything
                    // else here is a menu id nothing claims, so ignore it
                    // rather than guessing at an action.
                    if let Some((_, path)) = self.ids.log_items.iter().find(|(mid, _)| *mid == id) {
                        open_path(path);
                    }
                }
            }
        }
    }
}

/// Open a log file in the platform's default handler (TASK-VTT098) — the
/// portable equivalent of the Linux tray's `xdg-open` helper.
fn open_path(path: &std::path::Path) {
    crate::vtt_log!("Opening file: {}", path.display());
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn().ok();
    }
    #[cfg(target_os = "windows")]
    {
        // Empty title argument first: `start` treats a lone quoted argument as
        // the window title rather than the thing to open.
        std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()
            .ok();
    }
}

/// Open the release page in the default browser (TASK-VTT095). NOT
/// independently verified — this module only compiles on macOS/Windows,
/// neither of which this dev machine can build for.
fn open_url(url: &str) {
    crate::vtt_log!("Opening URL: {}", url);
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn().ok();
    }
    #[cfg(target_os = "windows")]
    {
        // An empty title argument before the URL: `start` otherwise treats a
        // quoted first argument as the window title, not the target to open.
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .ok();
    }
}

/// A native modal About box (TASK-VTT139) — same text as the Linux tray's
/// `show_about_dialog`, so both platforms tell the same story. Blocks the
/// calling (menu-polling) thread until dismissed, same as Linux's
/// `dialog.run()`; MessageBoxW runs its own internal message loop while
/// shown, so this is the standard, expected behaviour for a native modal.
/// Win32's default MessageBoxW text is user-selectable via right-click
/// Copy / Ctrl+C, which is as close to "selectable text" as a message box
/// gets without a real GUI toolkit — muda ships none (see TASK-VTT138, the
/// spike that will decide whether one is worth adding for the fuller
/// settings dialog).
#[cfg(target_os = "windows")]
fn show_about_messagebox() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let text = wide(&format!(
        "Voice to Text\r\n\r\nVersion {}\r\nFree, open-source voice-to-text transcription\r\nhttps://github.com/powell-clark/voice-to-text\r\n\r\nHold your hotkey (see tray menu) and speak.\r\nRelease to transcribe.",
        env!("CARGO_PKG_VERSION")
    ));
    let title = wide("About Voice to Text");

    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

fn create_icon(r: u8, g: u8, b: u8) -> tray_icon::Icon {
    let size = 22u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0;
    let radius = center - 1.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let idx = ((y * size + x) * 4) as usize;
            if dx * dx + dy * dy < radius * radius {
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            }
        }
    }
    tray_icon::Icon::from_rgba(rgba, size, size).expect("Failed to create icon")
}
