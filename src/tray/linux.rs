use super::{LastTranscription, UiMessage};
use crate::corrections;
use crate::hotkey::{self, HotkeyCmd};
use crate::logging;
use crate::settings::{NewlineType, Settings};
use glib::clone;
use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

pub struct Tray {
    _indicator: Rc<RefCell<AppIndicator>>,
}

struct TrayState {
    settings: Arc<RwLock<Settings>>,
    model_item: gtk::MenuItem,
    model_menu: gtk::Menu,
    language_item: gtk::MenuItem,
    hotkey_item: gtk::MenuItem,
    logging_item: gtk::MenuItem,
    hotkey_cmd_tx: Option<Sender<HotkeyCmd>>,
    /// To the transcription worker: a model or language change asks it to
    /// load the result now (WorkItem::Preload, ADR-0011).
    work_tx: Sender<crate::WorkItem>,
    initializing: bool,
}

impl Tray {
    /// Build the tray icon and menu. Returns the Tray and a glib::Sender for
    /// cross-thread UI updates.
    pub fn new(
        settings: Arc<RwLock<Settings>>,
        _data_dir: &Path,
        last_transcription: LastTranscription,
        work_tx: Sender<crate::WorkItem>,
    ) -> anyhow::Result<(Self, super::UiSender)> {
        // Create AppIndicator
        let mut indicator = AppIndicator::new("voice-to-text-linux", "audio-input-microphone");
        indicator.set_status(AppIndicatorStatus::Active);
        indicator.set_title("VTT");

        let indicator = Rc::new(RefCell::new(indicator));

        // Build menu
        let mut menu = gtk::Menu::new();

        // --- Status ---
        let status_item = gtk::MenuItem::with_label("Status: Initializing...");
        status_item.set_sensitive(false);
        menu.append(&status_item);
        menu.append(&gtk::SeparatorMenuItem::new());

        // --- Language ---
        let lang = settings.read().unwrap().selected_language.clone();
        let lang_display = if lang == "en" {
            "English only"
        } else {
            "Multilingual"
        };
        let language_item = gtk::MenuItem::with_label(&format!("Language: {}", lang_display));
        let language_menu = gtk::Menu::new();

        let lang_en = gtk::RadioMenuItem::with_label("English only (fastest)");
        let lang_multi = gtk::RadioMenuItem::with_label_from_widget(
            &lang_en,
            Some("Multilingual (99 languages)"),
        );
        if lang == "en" {
            lang_en.set_active(true);
        } else {
            lang_multi.set_active(true);
        }
        language_menu.append(&lang_en);
        language_menu.append(&lang_multi);
        language_item.set_submenu(Some(&language_menu));
        menu.append(&language_item);

        // --- Model ---
        let model = settings.read().unwrap().selected_model.clone();
        let model_item = gtk::MenuItem::with_label(&format!("Model: {}", model));
        let model_menu = gtk::Menu::new();
        model_item.set_submenu(Some(&model_menu));
        menu.append(&model_item);

        // --- Microphone (info label) ---
        let mic_item = gtk::MenuItem::with_label("Microphone: default");
        mic_item.set_sensitive(false);
        menu.append(&mic_item);

        // --- Backend (info label, TASK-VTT054) ---
        let backend_label = if settings.read().unwrap().backend == "ct2" {
            "Backend: CT2 (starting...)"
        } else {
            "Backend: Native"
        };
        let backend_item = gtk::MenuItem::with_label(backend_label);
        backend_item.set_sensitive(false);
        menu.append(&backend_item);

        // --- Hotkey ---
        let hk_code = settings.read().unwrap().hotkey_keycode;
        let hk_name = hotkey::get_key_name(hk_code);
        let hotkey_item = gtk::MenuItem::with_label(&format!("Hotkey: {}", hk_name));
        menu.append(&hotkey_item);

        // --- Settings (TASK-VTT051: single consolidated dialog, replacing
        // hand-editing settings.conf; language/model/hotkey stay also
        // reachable from their own tray items above, unchanged) ---
        let prompt_item = gtk::MenuItem::with_label("Settings...");
        menu.append(&prompt_item);

        // --- Copy last transcription (recovery net, FEAT-VTT038) ---
        let copy_last_item = gtk::MenuItem::with_label("Copy last transcription");
        menu.append(&copy_last_item);

        // --- Re-transcribe last recording (recovery net, FEAT-VTT039) ---
        let retranscribe_item = gtk::MenuItem::with_label("Re-transcribe last recording");
        menu.append(&retranscribe_item);

        menu.append(&gtk::SeparatorMenuItem::new());

        // --- Logging toggle ---
        let log_enabled = settings.read().unwrap().logging_enabled;
        let logging_item = gtk::MenuItem::with_label(if log_enabled {
            "Logging: On"
        } else {
            "Logging: Off"
        });
        menu.append(&logging_item);

        // --- Logs submenu ---
        // Refilled in place by a poll below, not on a show signal: Ayatana
        // AppIndicator exports the menu over D-Bus and never shows it as a GTK
        // widget, so a show handler fires once at startup and the menu
        // freezes at daemon start (TASK-VTT176).
        let logs_item = gtk::MenuItem::with_label("Logs");
        let logs_menu = gtk::Menu::new();
        let shown_logs: RefCell<Vec<LogRow>> = RefCell::new(Vec::new());
        refresh_logs_menu(&logs_menu, &shown_logs);
        logs_item.set_submenu(Some(&logs_menu));
        menu.append(&logs_item);

        menu.append(&gtk::SeparatorMenuItem::new());

        // --- Update available (TASK-VTT095, hidden until a check finds one) ---
        let update_item = gtk::MenuItem::with_label("Update available");
        update_item.set_no_show_all(true);
        update_item.hide();
        menu.append(&update_item);

        // --- About ---
        let about_item = gtk::MenuItem::with_label("About Voice to Text");
        menu.append(&about_item);
        menu.append(&gtk::SeparatorMenuItem::new());

        // --- Quit ---
        let quit_item = gtk::MenuItem::with_label("Quit");
        menu.append(&quit_item);

        menu.show_all();
        indicator.borrow_mut().set_menu(&mut menu);

        // --- Shared state ---
        let state = Rc::new(RefCell::new(TrayState {
            settings: settings.clone(),
            model_item: model_item.clone(),
            model_menu: model_menu.clone(),
            language_item: language_item.clone(),
            hotkey_item: hotkey_item.clone(),
            logging_item: logging_item.clone(),
            hotkey_cmd_tx: None,
            work_tx: work_tx.clone(),
            initializing: true,
        }));

        // Build initial model menu
        rebuild_model_menu(&state);
        state.borrow_mut().initializing = false;

        // --- Connect signals ---

        // Language: English
        {
            let st = state.clone();
            lang_en.connect_activate(move |item| {
                if !item.is_active() {
                    return;
                }
                let s = st.borrow();
                if s.initializing {
                    return;
                }
                s.settings.write().unwrap().selected_language = "en".into();
                update_language_label(&s, "en");
                drop(s);
                rebuild_model_menu(&st);
                save_settings(&st);
                crate::vtt_log!("Language changed to English");
            });
        }
        // Language: Multilingual
        {
            let st = state.clone();
            lang_multi.connect_activate(move |item| {
                if !item.is_active() {
                    return;
                }
                let s = st.borrow();
                if s.initializing {
                    return;
                }
                s.settings.write().unwrap().selected_language = "auto".into();
                update_language_label(&s, "auto");
                drop(s);
                rebuild_model_menu(&st);
                save_settings(&st);
                crate::vtt_log!("Language changed to Multilingual");
            });
        }

        // Logging toggle
        {
            let st = state.clone();
            logging_item.connect_activate(move |_| {
                let s = st.borrow();
                let enabled = !s.settings.read().unwrap().logging_enabled;
                s.settings.write().unwrap().logging_enabled = enabled;
                logging::set_enabled(enabled);
                s.logging_item.set_label(if enabled {
                    "Logging: On"
                } else {
                    "Logging: Off"
                });
                crate::vtt_log!("Logging {}", if enabled { "enabled" } else { "disabled" });
            });
        }

        // Hotkey
        {
            let st = state.clone();
            hotkey_item.connect_activate(move |_| {
                show_hotkey_dialog(&st);
            });
        }

        // Settings dialog
        {
            let st = state.clone();
            prompt_item.connect_activate(move |_| {
                show_settings_dialog(&st);
            });
        }

        // Copy last transcription — safe no-op (with a log line) when nothing
        // has been transcribed yet this run.
        {
            let lt = last_transcription.clone();
            copy_last_item.connect_activate(move |_| {
                let text = lt.lock().unwrap().clone();
                match text {
                    Some(text) => {
                        crate::typing::set_clipboard_text(&text);
                        crate::vtt_log!(
                            "Copied last transcription to clipboard ({} bytes)",
                            text.len()
                        );
                    }
                    None => {
                        crate::vtt_log!("Copy last transcription: nothing transcribed yet this run")
                    }
                }
            });
        }

        // Re-transcribe last recording — asks the worker to re-run whisper on
        // the newest archived WAV and re-type it (FEAT-VTT039). The worker
        // locates/decodes the file and handles the empty-dir no-op, so this
        // handler just fires the signal.
        {
            let work_tx = work_tx.clone();
            retranscribe_item.connect_activate(move |_| {
                if work_tx.send(crate::WorkItem::RetranscribeLast).is_err() {
                    crate::vtt_log!("Re-transcribe: worker channel closed");
                }
            });
        }

        // Update available (TASK-VTT095) — the URL arrives later over
        // UiMessage::UpdateAvailable, so it's held in a cell shared with the
        // poll closure below rather than captured at item-creation time.
        let update_url: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        {
            let update_url = update_url.clone();
            update_item.connect_activate(move |_| {
                if let Some(url) = update_url.borrow().clone() {
                    open_file(&url);
                }
            });
        }

        // About
        about_item.connect_activate(|_| {
            show_about_dialog();
        });

        // Quit — shared helper stops the systemd unit when one manages us,
        // otherwise exits the process directly (TASK-VTT122).
        quit_item.connect_activate(|_| {
            super::quit::quit();
        });

        // Microphone label auto-refresh
        glib::timeout_add_seconds_local(
            3,
            clone!(@weak mic_item => @default-return glib::ControlFlow::Break, move || {
                if let Some(desc) = get_default_mic_description() {
                    mic_item.set_label(&format!("Microphone: {}", desc));
                }
                glib::ControlFlow::Continue
            }),
        );
        // Initial mic label
        if let Some(desc) = get_default_mic_description() {
            mic_item.set_label(&format!("Microphone: {}", desc));
        }

        // Logs submenu auto-refresh — picks up a new day's log, a purge, and
        // the Today/Yesterday labels rolling over at midnight.
        glib::timeout_add_seconds_local(
            3,
            clone!(@weak logs_menu => @default-return glib::ControlFlow::Break, move || {
                refresh_logs_menu(&logs_menu, &shown_logs);
                glib::ControlFlow::Continue
            }),
        );

        // --- Cross-thread UI update channel (mpsc, polled via glib timeout) ---
        let (ui_tx, ui_rx) = std::sync::mpsc::channel::<UiMessage>();

        let status_item_clone = status_item.clone();
        let indicator_clone = indicator.clone();
        let backend_item_clone = backend_item.clone();
        let update_item_clone = update_item.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            while let Ok(msg) = ui_rx.try_recv() {
                match msg {
                    UiMessage::SetStatus(text) => {
                        status_item_clone.set_label(&format!("Status: {}", text));
                    }
                    UiMessage::SetIcon(icon) => {
                        let icon_name = match icon.as_str() {
                            "ready" => "audio-input-microphone",
                            "recording" => "media-record",
                            "processing" => "emblem-synchronizing",
                            _ => "audio-input-microphone",
                        };
                        indicator_clone
                            .borrow_mut()
                            .set_icon_full(icon_name, "status");
                    }
                    UiMessage::SetBackendLabel(label) => {
                        backend_item_clone.set_label(&format!("Backend: {}", label));
                    }
                    UiMessage::UpdateAvailable(version, url) => {
                        update_item_clone.set_label(&format!("Update available: {}", version));
                        *update_url.borrow_mut() = Some(url);
                        update_item_clone.show();
                    }
                }
            }
            glib::ControlFlow::Continue
        });

        Ok((
            Tray {
                _indicator: indicator,
            },
            ui_tx,
        ))
    }
}

// ─── Model menu ─────────────────────────────────────────────────

fn rebuild_model_menu(state: &Rc<RefCell<TrayState>>) {
    let s = state.borrow_mut();
    let is_english = s.settings.read().unwrap().selected_language == "en";
    let current_model = s.settings.read().unwrap().selected_model.clone();

    // Clear existing items
    for child in s.model_menu.children() {
        s.model_menu.remove(&child);
    }

    // Strip .en from current model if switching to multilingual
    if !is_english && current_model.contains(".en") {
        let new_model = current_model.replace(".en", "");
        crate::vtt_log!(
            "Auto-switching from {} to {} (multilingual)",
            current_model,
            new_model
        );
        s.settings.write().unwrap().selected_model = new_model.clone();
        s.model_item.set_label(&format!("Model: {}", new_model));
    }

    let current = s.settings.read().unwrap().selected_model.clone();
    let current_base = current.replace(".en", "");

    let model_entries = model_menu_entries();
    let models: Vec<(&str, &str)> = model_entries
        .iter()
        .map(|(label, key)| (label.as_str(), *key))
        .collect();
    let mut group: Option<gtk::RadioMenuItem> = None;

    // Normalise current selection down to a menu key, handling legacy values
    let legacy = ["CT2 ", "W "];
    let cleaned = legacy.iter().fold(current_base.clone(), |acc, p| {
        acc.trim_start_matches(*p).to_string()
    });
    let base_key = cleaned.trim_end_matches(".en");

    for &(label, key) in &models {
        let item = match &group {
            None => gtk::RadioMenuItem::with_label(label),
            Some(g) => gtk::RadioMenuItem::with_label_from_widget(g, Some(label)),
        };
        if group.is_none() {
            group = Some(item.clone());
        }

        // Match on the base key so english/multilingual mode doesn't affect selection visuals
        item.set_active(key == base_key);
        item.set_sensitive(true);

        let st = state.clone();
        let model_name = key.to_string();
        item.connect_activate(move |item| {
            if !item.is_active() {
                return;
            }
            on_model_selected(&st, &model_name);
        });

        s.model_menu.append(&item);
    }

    s.model_menu.show_all();
}

fn on_model_selected(state: &Rc<RefCell<TrayState>>, model: &str) {
    let s = state.borrow();
    if s.initializing {
        return;
    }
    crate::vtt_log!("Model selected: {}", model);
    s.settings.write().unwrap().selected_model = model.to_string();
    s.model_item.set_label(&format!("Model: {}", model));
    drop(s);
    save_settings(state);
}

fn update_language_label(state: &TrayState, lang: &str) {
    let display = if lang == "en" {
        "English only"
    } else {
        "Multilingual"
    };
    state
        .language_item
        .set_label(&format!("Language: {}", display));
}

/// Persist a model or language change and have the worker load the model it
/// resolves to now, rather than on the next key press.
fn save_settings(state: &Rc<RefCell<TrayState>>) {
    let s = state.borrow();
    let settings = s.settings.read().unwrap();
    if let Err(e) = settings.save() {
        crate::vtt_log!("Failed to save settings: {}", e);
    }
    drop(settings);
    request_preload(&s);
}

fn request_preload(state: &TrayState) {
    if state.work_tx.send(crate::WorkItem::Preload).is_err() {
        crate::vtt_log!("Model preload: worker channel closed");
    }
}

// ─── Logs submenu ───────────────────────────────────────────────

/// One Logs submenu row: its label, and the log it opens. A row with no path
/// is a disabled placeholder.
type LogRow = (String, Option<PathBuf>);

/// The Logs submenu's rows, newest first. Pure so the change check that
/// gates a refill is testable without GTK.
fn logs_menu_rows(
    files: Result<Vec<String>, String>,
    log_dir: &Path,
    today: &str,
    yesterday: &str,
) -> Vec<LogRow> {
    match files {
        Err(e) => vec![(format!("(log dir unreadable: {})", e), None)],
        Ok(f) if f.is_empty() => vec![("(no logs yet)".to_string(), None)],
        Ok(f) => f
            .iter()
            .map(|name| {
                (
                    logging::format_log_label(name, today, yesterday),
                    Some(log_dir.join(name)),
                )
            })
            .collect(),
    }
}

/// Refill the Logs submenu in place, only when its rows have changed — the
/// same remove-and-append shape `rebuild_model_menu` uses, which the D-Bus
/// menu export tracks.
fn refresh_logs_menu(menu: &gtk::Menu, shown: &RefCell<Vec<LogRow>>) {
    let (today, yesterday) = logging::today_and_yesterday();
    let rows = logs_menu_rows(
        logging::list_log_filenames(),
        &logging::get_dir(),
        &today,
        &yesterday,
    );
    if *shown.borrow() == rows {
        return;
    }

    for child in menu.children() {
        menu.remove(&child);
    }
    for (label, path) in &rows {
        let item = gtk::MenuItem::with_label(label);
        match path {
            Some(p) => {
                let full_path = p.to_string_lossy().to_string();
                item.connect_activate(move |_| {
                    open_file(&full_path);
                });
            }
            None => item.set_sensitive(false),
        }
        menu.append(&item);
    }
    menu.show_all();
    *shown.borrow_mut() = rows;
}

fn open_file(path: &str) {
    crate::vtt_log!("Opening file: {}", path);
    std::process::Command::new("xdg-open")
        .arg(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok();
}

// ─── Microphone detection ───────────────────────────────────────

fn get_default_mic_description() -> Option<String> {
    let output = std::process::Command::new("pactl")
        .args(["get-default-source"])
        .output()
        .ok()?;
    let source_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if source_name.is_empty() {
        return None;
    }

    let output = std::process::Command::new("pactl")
        .args(["list", "sources"])
        .output()
        .ok()?;
    let list = String::from_utf8_lossy(&output.stdout);

    let mut current_name = String::new();
    let mut matched = false;

    for line in list.lines() {
        if line.starts_with("Source #") {
            current_name.clear();
            matched = false;
        } else if let Some(name) = line.trim().strip_prefix("Name: ") {
            current_name = name.to_string();
            matched = current_name == source_name;
        } else if matched {
            if let Some(desc) = line.trim().strip_prefix("Description: ") {
                return Some(desc.to_string());
            }
        }
    }
    None
}

// ─── Dialogs ────────────────────────────────────────────────────

fn show_about_dialog() {
    let dialog = gtk::Dialog::with_buttons(
        Some("About Voice to Text"),
        None::<&gtk::Window>,
        gtk::DialogFlags::MODAL,
        &[("Close", gtk::ResponseType::Close)],
    );
    dialog.set_default_size(400, 200);

    let content = dialog.content_area();
    let about_text = format!(
        "Voice to Text Linux\n\n\
         Version {}\n\
         Free, open-source voice-to-text transcription\n\
         https://github.com/powell-clark/voice-to-text\n\n\
         Hold your hotkey (see tray menu) and speak.\n\
         Release to transcribe.\n\n\
         {}",
        env!("CARGO_PKG_VERSION"),
        engine_credits(crate::engine::version())
    );
    let label = gtk::Label::new(Some(&about_text));
    label.set_selectable(true);
    label.set_justify(gtk::Justification::Center);
    label.set_margin_top(20);
    label.set_margin_bottom(20);
    label.set_margin_start(20);
    label.set_margin_end(20);
    content.add(&label);

    dialog.show_all();
    dialog.run();
    unsafe {
        dialog.destroy();
    }
}

/// Default width of the settings dialog's content, in logical pixels.
const SETTINGS_DIALOG_WIDTH: i32 = 520;
/// Room left under the work area for the title bar and a margin, which the
/// window's default size does not include.
const SETTINGS_DIALOG_FRAME_ALLOWANCE: i32 = 80;
/// Work-area height assumed when GDK reports no monitor at all: a 768-pixel
/// laptop screen less its top panel.
const SETTINGS_DIALOG_FALLBACK_WORKAREA: i32 = 700;
/// The dialog never opens shorter than this, however small the screen.
const SETTINGS_DIALOG_MIN_HEIGHT: i32 = 240;

/// The settings dialog's opening height: tall enough for its tallest tab,
/// but never taller than the work area (the monitor less its panels and
/// dock) minus the title bar, so Save opens on screen (TASK-VTT204). GDK
/// reports the whole monitor where a backend has no work area; `None` means
/// it reported no monitor, and a laptop-sized screen is assumed.
fn settings_dialog_height(natural: i32, workarea_height: Option<i32>) -> i32 {
    let workarea = workarea_height
        .filter(|h| *h > 0)
        .unwrap_or(SETTINGS_DIALOG_FALLBACK_WORKAREA);
    let cap = (workarea - SETTINGS_DIALOG_FRAME_ALLOWANCE).max(SETTINGS_DIALOG_MIN_HEIGHT);
    natural.clamp(SETTINGS_DIALOG_MIN_HEIGHT, cap)
}

/// Work-area height of the monitor under the pointer, which is the monitor
/// the dialog opens centred on, falling back to the primary monitor and then
/// the first. `None` when GDK reports no monitor.
fn pointer_monitor_workarea_height() -> Option<i32> {
    let display = gtk::gdk::Display::default()?;
    let at_pointer = display
        .default_seat()
        .and_then(|seat| seat.pointer())
        .and_then(|pointer| {
            let (_, x, y) = pointer.position();
            display.monitor_at_point(x, y)
        });
    let monitor = at_pointer
        .or_else(|| display.primary_monitor())
        .or_else(|| display.monitor(0))?;
    Some(monitor.workarea().height())
}

/// The menu key a saved model name selects: ".en" and the legacy "CT2 " and
/// "W " prefixes stripped, the same normalisation the tray's model submenu
/// applies.
fn model_base_key(selected_model: &str) -> String {
    let legacy = ["CT2 ", "W "];
    let cleaned = legacy
        .iter()
        .fold(selected_model.replace(".en", ""), |acc, p| {
            acc.trim_start_matches(*p).to_string()
        });
    cleaned.trim_end_matches(".en").to_string()
}

/// Every value the settings dialog shows and saves, as plain data. The
/// dialog fills its widgets from one of these and reads one back on Save, so
/// the mapping to and from `Settings` is a pure function a test can call
/// (TASK-VTT204). `show_form` and `read_form` name every field, so a widget
/// dropped from a tab page fails to compile rather than silently keeping its
/// old saved value.
#[derive(Debug, Clone, PartialEq)]
struct SettingsForm {
    /// "English only" is chosen; otherwise "Multilingual".
    language_en: bool,
    /// Key of the chosen model row, from `model_menu_entries`.
    model_key: Option<String>,
    /// Input-device row: 0 is "Default", row i is device ordinal i-1.
    device_row: u32,
    voice_prefix: String,
    initial_prompt: String,
    /// The corrections box, one `misheard => correct` pair per line.
    corrections_text: String,
    append_newline: bool,
    shift_return: bool,
    archive_recordings: bool,
    denoise: bool,
}

impl SettingsForm {
    /// What the dialog shows for `settings`, given the model rows in order
    /// and the number of listed input devices.
    fn from_settings(settings: &Settings, model_keys: &[&str], device_count: usize) -> Self {
        let base_key = model_base_key(&settings.selected_model);
        // A radio group always has one row chosen, and the first stays chosen
        // until another is, so an unlisted saved model shows as the first row.
        let model_key = model_keys
            .iter()
            .find(|k| **k == base_key)
            .or_else(|| model_keys.first())
            .map(|k| k.to_string());
        let device_row = if settings.selected_device_index >= 0
            && (settings.selected_device_index as usize) < device_count
        {
            settings.selected_device_index as u32 + 1
        } else {
            0
        };
        SettingsForm {
            language_en: settings.selected_language == "en",
            model_key,
            device_row,
            voice_prefix: settings.voice_prefix.clone(),
            initial_prompt: settings.initial_prompt.clone(),
            corrections_text: corrections::format_pairs(&settings.corrections),
            append_newline: settings.append_newline,
            shift_return: settings.newline_type == NewlineType::ShiftReturn,
            archive_recordings: settings.archive_recordings,
            denoise: settings.denoise,
        }
    }

    /// Write the form into `settings`. Fields the dialog does not show (the
    /// hotkey, which its own dialog saves, logging, the archive path and cap,
    /// the backend and the text clean-up switches) are left as they are.
    fn apply_to(&self, settings: &mut Settings, device_count: usize) {
        settings.selected_language = if self.language_en { "en" } else { "auto" }.to_string();
        if let Some(key) = &self.model_key {
            settings.selected_model = key.clone();
        }
        // Row 0 is "Default" (-1); row i is device ordinal i-1, matching the
        // order list_input_device_names() and resolve_device_ordinal use.
        settings.selected_device_index = match self.device_row {
            0 => -1,
            row if (row as usize) <= device_count => row as i32 - 1,
            _ => -1,
        };
        settings.voice_prefix = self.voice_prefix.clone();
        settings.initial_prompt = self.initial_prompt.clone();
        // An emptied box means "remove them all", so this assigns
        // unconditionally rather than skipping on empty (TASK-VTT144).
        settings.corrections = corrections::parse_pairs(&self.corrections_text);
        settings.append_newline = self.append_newline;
        settings.newline_type = if self.shift_return {
            NewlineType::ShiftReturn
        } else {
            NewlineType::PlainReturn
        };
        settings.archive_recordings = self.archive_recordings;
        settings.denoise = self.denoise;
    }

    /// Reset Default: the text boxes and switches go back to the shipped
    /// defaults, archiving and the rumble filter off among them; language,
    /// model and input device keep the user's choice.
    fn reset_to_defaults(&mut self) {
        let shipped = SettingsForm::from_settings(&Settings::default(), &[], 0);
        self.voice_prefix = shipped.voice_prefix;
        self.initial_prompt = shipped.initial_prompt;
        self.corrections_text = shipped.corrections_text;
        self.append_newline = shipped.append_newline;
        self.shift_return = shipped.shift_return;
        self.archive_recordings = shipped.archive_recordings;
        self.denoise = shipped.denoise;
    }
}

/// The settings dialog's window and the widgets its buttons act on.
struct SettingsDialog {
    window: gtk::Window,
    hotkey_current: gtk::Label,
    hotkey_change: gtk::Button,
    lang_en: gtk::RadioButton,
    lang_multi: gtk::RadioButton,
    model_buttons: Vec<(gtk::RadioButton, &'static str)>,
    device_combo: gtk::ComboBoxText,
    device_count: usize,
    prefix_entry: gtk::Entry,
    prompt_buffer: gtk::TextBuffer,
    corrections_buffer: gtk::TextBuffer,
    newline_toggle: gtk::CheckButton,
    plain_radio: gtk::RadioButton,
    shift_radio: gtk::RadioButton,
    archive_toggle: gtk::CheckButton,
    denoise_toggle: gtk::CheckButton,
    reset_btn: gtk::Button,
    cancel_btn: gtk::Button,
    save_btn: gtk::Button,
}

impl SettingsDialog {
    fn show_form(&self, form: &SettingsForm) {
        let SettingsForm {
            language_en,
            model_key,
            device_row,
            voice_prefix,
            initial_prompt,
            corrections_text,
            append_newline,
            shift_return,
            archive_recordings,
            denoise,
        } = form;
        if *language_en {
            self.lang_en.set_active(true);
        } else {
            self.lang_multi.set_active(true);
        }
        for (button, key) in &self.model_buttons {
            if model_key.as_deref() == Some(*key) {
                button.set_active(true);
            }
        }
        self.device_combo.set_active(Some(*device_row));
        self.prefix_entry.set_text(voice_prefix);
        self.prompt_buffer.set_text(initial_prompt);
        self.corrections_buffer.set_text(corrections_text);
        self.newline_toggle.set_active(*append_newline);
        if *shift_return {
            self.shift_radio.set_active(true);
        } else {
            self.plain_radio.set_active(true);
        }
        self.archive_toggle.set_active(*archive_recordings);
        self.denoise_toggle.set_active(*denoise);
    }

    fn read_form(&self) -> SettingsForm {
        SettingsForm {
            language_en: self.lang_en.is_active(),
            model_key: self
                .model_buttons
                .iter()
                .find(|(button, _)| button.is_active())
                .map(|(_, key)| key.to_string()),
            device_row: self.device_combo.active().unwrap_or(0),
            voice_prefix: self.prefix_entry.text().to_string(),
            initial_prompt: buffer_text(&self.prompt_buffer),
            corrections_text: buffer_text(&self.corrections_buffer),
            append_newline: self.newline_toggle.is_active(),
            shift_return: self.shift_radio.is_active(),
            archive_recordings: self.archive_toggle.is_active(),
            denoise: self.denoise_toggle.is_active(),
        }
    }
}

/// One tab of the settings dialog. The page sits in its own scroller, so a
/// window resized shorter than the page scrolls the page while the button
/// row, outside the notebook, stays put (TASK-VTT204).
fn settings_page(notebook: &gtk::Notebook, title: &str) -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 6);
    page.set_border_width(12);
    let scroller = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    // The page's natural height reaches the window, so it opens tall enough
    // for the tallest tab; its minimum does not, so it can still be shortened.
    scroller.set_propagate_natural_height(true);
    scroller.add(&page);
    notebook.append_page(&scroller, Some(&gtk::Label::new(Some(title))));
    page
}

/// A bold section heading on a settings page; `gap` puts space above it.
fn section_heading(page: &gtk::Box, markup: &str, gap: bool) {
    let label = gtk::Label::new(None);
    label.set_markup(markup);
    label.set_halign(gtk::Align::Start);
    label.set_xalign(0.0);
    label.set_line_wrap(true);
    if gap {
        label.set_margin_top(12);
    }
    page.pack_start(&label, false, false, 0);
}

/// A plain, wrapping note under a setting.
fn section_note(page: &gtk::Box, text: &str) {
    let label = gtk::Label::new(Some(text));
    label.set_halign(gtk::Align::Start);
    label.set_xalign(0.0);
    label.set_line_wrap(true);
    label.set_line_wrap_mode(gtk::pango::WrapMode::WordChar);
    page.pack_start(&label, false, false, 0);
}

/// Build the settings window for `settings`, sized to the monitor's work
/// area, without showing it. Nothing here reads or writes the live settings;
/// `show_settings_dialog` connects the hotkey, Save and Cancel buttons.
fn build_settings_dialog(
    settings: &Settings,
    data_dir: &Path,
    device_names: &[String],
) -> SettingsDialog {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Settings");
    window.set_resizable(true);
    window.set_position(gtk::WindowPosition::Center);
    window.set_border_width(12);

    // The notebook takes the spare height; the button row below it sits
    // outside the notebook and every scroller, so no page or window height
    // can push Save out of view (TASK-VTT204 pre-mortem).
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 12);
    window.add(&outer);
    let notebook = gtk::Notebook::new();
    outer.pack_start(&notebook, true, true, 0);

    // ── General: hotkey, language, model, input device ──
    let general = settings_page(&notebook, "General");

    // Hotkey — the Change... button opens show_hotkey_dialog, which captures
    // and saves the key itself; show_settings_dialog re-reads it afterwards.
    section_heading(&general, "<b>Push-to-talk hotkey:</b>", false);
    let hotkey_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let hotkey_current = gtk::Label::new(Some(&hotkey::get_key_name(settings.hotkey_keycode)));
    hotkey_current.set_halign(gtk::Align::Start);
    hotkey_row.pack_start(&hotkey_current, true, true, 0);
    let hotkey_change = gtk::Button::with_label("Change...");
    hotkey_row.pack_start(&hotkey_change, false, false, 0);
    general.pack_start(&hotkey_row, false, false, 0);

    section_heading(&general, "<b>Language:</b>", true);
    let lang_en = gtk::RadioButton::with_label("English only (fastest)");
    let lang_multi =
        gtk::RadioButton::with_label_from_widget(&lang_en, "Multilingual (99 languages)");
    general.pack_start(&lang_en, false, false, 0);
    general.pack_start(&lang_multi, false, false, 0);

    // Model — the same catalogue the tray's own submenu lists.
    section_heading(&general, "<b>Model:</b>", true);
    let model_entries = model_menu_entries();
    let mut model_group: Option<gtk::RadioButton> = None;
    let mut model_buttons: Vec<(gtk::RadioButton, &'static str)> = Vec::new();
    for (label, key) in &model_entries {
        let button = match &model_group {
            None => gtk::RadioButton::with_label(label.as_str()),
            Some(group) => gtk::RadioButton::with_label_from_widget(group, label.as_str()),
        };
        if model_group.is_none() {
            model_group = Some(button.clone());
        }
        general.pack_start(&button, false, false, 0);
        model_buttons.push((button, *key));
    }

    // Input device — a basic picker that saves the ordinal; the tray submenu
    // with hot-plug refresh is TASK-VTT129's scope, not duplicated here.
    section_heading(
        &general,
        "<b>Input device (restart required to take effect):</b>",
        true,
    );
    let device_combo = gtk::ComboBoxText::new();
    device_combo.append_text("Default");
    for name in device_names {
        device_combo.append_text(name);
    }
    general.pack_start(&device_combo, false, false, 0);

    // ── Transcription: voice prefix, initial prompt, corrections ──
    let transcription = settings_page(&notebook, "Transcription");

    section_heading(
        &transcription,
        "<b>Voice Prefix (prepended to every transcription):</b>",
        false,
    );
    let prefix_entry = gtk::Entry::new();
    prefix_entry.set_placeholder_text(Some("e.g., [voice] "));
    transcription.pack_start(&prefix_entry, false, false, 0);

    section_heading(
        &transcription,
        "<b>Initial Prompt (helps Whisper recognize your voice, max 240 chars):</b>",
        true,
    );
    let prompt_scroller =
        gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    prompt_scroller.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    prompt_scroller.set_size_request(-1, 70);
    transcription.pack_start(&prompt_scroller, true, true, 0);
    let prompt_view = gtk::TextView::new();
    prompt_view.set_wrap_mode(gtk::WrapMode::WordChar);
    let prompt_buffer = prompt_view.buffer().unwrap();
    prompt_scroller.add(&prompt_view);
    let counter = gtk::Label::new(None);
    counter.set_halign(gtk::Align::End);
    transcription.pack_start(&counter, false, false, 0);

    // Corrections. The format is stated in the heading because `parse_pairs`
    // drops a malformed row silently — a user who types `->` instead of `=>`
    // would otherwise watch their rule vanish on save (TASK-VTT144).
    section_heading(
        &transcription,
        "<b>Corrections (one per line, <tt>misheard =&gt; correct</tt>):</b>",
        true,
    );
    let corrections_scroller =
        gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    corrections_scroller.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    corrections_scroller.set_size_request(-1, 110);
    transcription.pack_start(&corrections_scroller, true, true, 0);
    let corrections_view = gtk::TextView::new();
    corrections_view.set_wrap_mode(gtk::WrapMode::WordChar);
    let corrections_buffer = corrections_view.buffer().unwrap();
    corrections_scroller.add(&corrections_view);
    section_note(
        &transcription,
        "Whole words only, case-insensitive. Empty the box to remove them all.",
    );

    // ── Output: newline behaviour ──
    let output = settings_page(&notebook, "Output");
    let newline_toggle = gtk::CheckButton::with_label("Insert newline between transcriptions");
    output.pack_start(&newline_toggle, false, false, 0);
    section_heading(&output, "<b>Newline key behavior:</b>", true);
    let plain_radio = gtk::RadioButton::with_label("Plain Return (may send messages in chat apps)");
    output.pack_start(&plain_radio, false, false, 0);
    let shift_radio = gtk::RadioButton::with_label_from_widget(
        &plain_radio,
        "Shift+Return (safer, won't send messages)",
    );
    output.pack_start(&shift_radio, false, false, 0);

    // ── Audio and privacy: archive, rumble filter ──
    // Both were settings-file only before, and the file most people would
    // edit by hand is the wrong one — the app reads the data directory, not
    // ~/.config (TASK-VTT154).
    let audio = settings_page(&notebook, "Audio and privacy");
    let archive_toggle = gtk::CheckButton::with_label(
        "Archive my recordings — saves my voice AND what I said, to disk",
    );
    audio.pack_start(&archive_toggle, false, false, 0);
    section_note(
        &audio,
        &format!(
            "Saved to {}. Nothing is uploaded. See the README before enabling.",
            crate::archive::resolve_archive_dir(&settings.archive_dir, data_dir).display()
        ),
    );
    let denoise_toggle =
        gtk::CheckButton::with_label("Filter low-frequency rumble before transcribing");
    denoise_toggle.set_margin_top(12);
    audio.pack_start(&denoise_toggle, false, false, 0);
    section_note(
        &audio,
        "These two apply on restart: systemctl --user restart vtt.service",
    );

    // ── Button row, outside the notebook ──
    let button_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    outer.pack_start(&button_row, false, false, 0);
    let reset_btn = gtk::Button::with_label("Reset Default");
    reset_btn.set_size_request(120, -1);
    button_row.pack_start(&reset_btn, false, false, 0);
    let cancel_btn = gtk::Button::with_label("Cancel");
    cancel_btn.set_size_request(80, -1);
    let save_btn = gtk::Button::with_label("Save");
    save_btn.set_size_request(80, -1);
    button_row.pack_end(&save_btn, false, false, 0);
    button_row.pack_end(&cancel_btn, false, false, 0);

    let dialog = SettingsDialog {
        window,
        hotkey_current,
        hotkey_change,
        lang_en,
        lang_multi,
        model_buttons,
        device_combo,
        device_count: device_names.len(),
        prefix_entry,
        prompt_buffer,
        corrections_buffer,
        newline_toggle,
        plain_radio,
        shift_radio,
        archive_toggle,
        denoise_toggle,
        reset_btn,
        cancel_btn,
        save_btn,
    };

    // Fill the widgets before the 240-character limit is connected, so a
    // longer hand-edited prompt opens whole, as it always has.
    let model_keys: Vec<&str> = model_entries.iter().map(|(_, key)| *key).collect();
    dialog.show_form(&SettingsForm::from_settings(
        settings,
        &model_keys,
        device_names.len(),
    ));
    update_char_counter(&dialog.prompt_buffer, &counter);
    dialog.prompt_buffer.connect_changed(move |buf| {
        if buf.char_count() > 240 {
            let start = buf.start_iter();
            let mut end = buf.start_iter();
            end.set_offset(240);
            if let Some(text) = buf.text(&start, &end, false) {
                buf.set_text(&text);
            }
        }
        update_char_counter(buf, &counter);
    });

    // Open as tall as the tallest tab wants, capped to the work area.
    outer.show_all();
    let content_width = SETTINGS_DIALOG_WIDTH - 2 * dialog.window.border_width() as i32;
    let (_, natural) = outer.preferred_height_for_width(content_width);
    let natural = natural + 2 * dialog.window.border_width() as i32;
    dialog.window.set_default_size(
        SETTINGS_DIALOG_WIDTH,
        settings_dialog_height(natural, pointer_monitor_workarea_height()),
    );
    dialog
}

fn show_settings_dialog(state: &Rc<RefCell<TrayState>>) {
    let settings = state.borrow().settings.read().unwrap().clone();

    // Resolve the archive destination for display: an empty archive_dir means
    // "the default", which is exactly the value a user cannot interpret.
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("voice-to-text");

    let device_names = crate::audio::list_input_device_names();
    let dialog = Rc::new(build_settings_dialog(&settings, &data_dir, &device_names));

    // Hotkey: show_hotkey_dialog saves on release; re-read so this dialog's
    // own label doesn't go stale while it stays open.
    {
        let st = state.clone();
        let hotkey_current = dialog.hotkey_current.clone();
        dialog.hotkey_change.connect_clicked(move |_| {
            show_hotkey_dialog(&st);
            let code = st.borrow().settings.read().unwrap().hotkey_keycode;
            hotkey_current.set_text(&hotkey::get_key_name(code));
        });
    }

    // Reset
    {
        let d = dialog.clone();
        dialog.reset_btn.connect_clicked(move |_| {
            let mut form = d.read_form();
            form.reset_to_defaults();
            d.show_form(&form);
        });
    }

    // Cancel
    {
        let d = dialog.clone();
        dialog.cancel_btn.connect_clicked(move |_| unsafe {
            d.window.destroy();
        });
    }

    // Save
    {
        let st = state.clone();
        let d = dialog.clone();
        dialog.save_btn.connect_clicked(move |_| {
            let form = d.read_form();
            let s = st.borrow();
            let mut settings = s.settings.write().unwrap();
            form.apply_to(&mut settings, d.device_count);
            if let Some(key) = &form.model_key {
                s.model_item.set_label(&format!("Model: {}", key));
            }
            update_language_label(&s, &settings.selected_language);

            if let Err(e) = settings.save() {
                crate::vtt_log!("Failed to save settings: {}", e);
            }

            crate::vtt_log!(
                "Settings updated: prefix='{}', prompt='{}', corrections={}, archive={}, denoise={}, \
                 language='{}', model='{}', device_index={}",
                settings.voice_prefix,
                settings.initial_prompt,
                settings.corrections.len(),
                settings.archive_recordings,
                settings.denoise,
                settings.selected_language,
                settings.selected_model,
                settings.selected_device_index
            );

            drop(settings);
            request_preload(&s);
            drop(s);
            rebuild_model_menu(&st);
            unsafe {
                d.window.destroy();
            }
        });
    }

    dialog.window.show_all();
}

/// The whole contents of a `TextBuffer` as a String, empty when the buffer
/// cannot produce text. Both multi-line fields in this dialog need it.
fn buffer_text(buffer: &gtk::TextBuffer) -> String {
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    buffer
        .text(&start, &end, false)
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn update_char_counter(buffer: &gtk::TextBuffer, label: &gtk::Label) {
    let count = buffer.char_count() as usize;
    let (markup_or_text, is_markup) = char_counter_markup(count);
    if is_markup {
        label.set_markup(&markup_or_text);
    } else {
        label.set_text(&markup_or_text);
    }
}

/// Pure: format the character-count label for the initial-prompt dialog.
/// Returns `(content, is_markup)` — when `is_markup` is true the caller
/// should use `set_markup` (Pango coloring), otherwise `set_text` plain.
///
/// Thresholds:
/// - < 200: plain text (no warning)
/// - 200..=229: orange (approaching limit)
/// - >= 230: red (almost at limit; limit is 240)
fn char_counter_markup(count: usize) -> (String, bool) {
    let text = format!("{} / 240 characters", count);
    if count >= 230 {
        (format!("<span color='red'>{}</span>", text), true)
    } else if count >= 200 {
        (format!("<span color='orange'>{}</span>", text), true)
    } else {
        (text, false)
    }
}

fn show_hotkey_dialog(state: &Rc<RefCell<TrayState>>) {
    let dialog = gtk::Dialog::with_buttons(
        Some("Customize Hotkey"),
        None::<&gtk::Window>,
        gtk::DialogFlags::MODAL,
        &[("Cancel", gtk::ResponseType::Cancel)],
    );
    dialog.set_default_size(400, 180);
    dialog.set_resizable(false);
    dialog.set_position(gtk::WindowPosition::Center);

    let content = dialog.content_area();
    content.set_border_width(20);

    let label = gtk::Label::new(Some(
        "Press and hold the key you want to use...\n\nWaiting for key press...",
    ));
    label.set_justify(gtk::Justification::Center);
    content.pack_start(&label, true, true, 10);

    let captured: Rc<RefCell<Option<u16>>> = Rc::new(RefCell::new(None));
    let key_pressed = Rc::new(RefCell::new(false));

    // Key press
    {
        let label = label.clone();
        let captured = captured.clone();
        let key_pressed = key_pressed.clone();
        dialog.connect_key_press_event(move |_, event| {
            let keycode = event.hardware_keycode();
            // X11 hardware keycodes are 8..=255. Anything outside that range
            // won't survive the u16→u8 cast that hotkey_keycode expects, so
            // reject early and tell the user rather than silently capturing
            // the wrong key.
            if !(8..=255).contains(&keycode) {
                label.set_text(&format!(
                    "Key {} is outside the valid X11 keycode range (8-255).\n\n\
                     Try a different key.",
                    keycode
                ));
                return glib::Propagation::Stop;
            }
            // Push-to-talk grabs its key globally, so the grab swallows every
            // press before the focused window sees it. Binding a key the user
            // types with removes that character from every application —
            // binding space cost the operator every space they typed, and
            // fired a recording on each one (TASK-VTT147). Refuse rather than
            // warn: the damage is silent and the only way back is hand-editing
            // settings.conf. Space and Return are also exactly the keys that
            // activate a focused widget, so they are easy to capture here by
            // accident.
            if hotkey::keycode_is_typing(keycode as u8) {
                let name = hotkey::get_key_name(keycode as u8);
                label.set_text(&format!(
                    "{} is a key you type with.\n\n\
                     Push-to-talk holds its key globally, so binding this would \
                     stop it working in every other application.\n\n\
                     Try Scroll Lock, Pause, or a function key.",
                    name
                ));
                return glib::Propagation::Stop;
            }

            *captured.borrow_mut() = Some(keycode);
            *key_pressed.borrow_mut() = true;

            let name = hotkey::get_key_name(keycode as u8);
            label.set_text(&format!(
                "Press and hold the key you want to use...\n\n\
                 Key detected: {}\n\nRelease the key to confirm.",
                name
            ));
            glib::Propagation::Stop
        });
    }

    // Key release
    {
        let st = state.clone();
        let captured = captured.clone();
        let key_pressed = key_pressed.clone();
        let dialog_weak = dialog.downgrade();
        dialog.connect_key_release_event(move |_, _| {
            if !*key_pressed.borrow() {
                return glib::Propagation::Stop;
            }
            // u16 → u8 would silently truncate keycodes > 255 (not valid X11
            // hardware keycodes but possible on exotic keyboards or via
            // emulated input). Reject them explicitly instead.
            let keycode = match *captured.borrow() {
                Some(kc) if (8..=255).contains(&kc) => kc as u8,
                _ => return glib::Propagation::Stop,
            };

            let name = hotkey::get_key_name(keycode);
            crate::vtt_log!("Hotkey changed to: {} (keycode {})", name, keycode);

            let s = st.borrow();
            s.settings.write().unwrap().hotkey_keycode = keycode;
            s.hotkey_item.set_label(&format!("Hotkey: {}", name));

            if let Some(ref tx) = s.hotkey_cmd_tx {
                tx.send(HotkeyCmd::SetKeycode(keycode)).ok();
            }

            if let Err(e) = s.settings.read().unwrap().save() {
                crate::vtt_log!("Failed to save hotkey: {}", e);
            }

            drop(s);

            if let Some(d) = dialog_weak.upgrade() {
                d.response(gtk::ResponseType::Ok);
            }

            glib::Propagation::Stop
        });
    }

    dialog.show_all();
    dialog.run();
    unsafe {
        dialog.destroy();
    }
}

// ─── Pure helpers (testable without GTK) ──────────────────────────

/// The About dialog's engine and model credits. Parakeet's weights are
/// CC-BY-4.0, which asks for attribution wherever they are used (ADR-0011).
fn engine_credits(whisper_cpp_version: &str) -> String {
    format!(
        "Speech engines: whisper.cpp {whisper_cpp_version} (MIT)\n\
         Whisper models by OpenAI (MIT)\n\
         Parakeet TDT 0.6B v3 by NVIDIA, licensed CC-BY-4.0\n\
         https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3"
    )
}

/// The model menu's rows as `(label, settings key)`, shared by the tray
/// submenu and the settings dialog so the two always list the same models.
/// Derived from models::MODELS instead of a hardcoded list: each multilingual
/// entry becomes one row (an English toggle swaps in .en variants at
/// transcription time via resolve_variant), and a model added to MODELS —
/// Parakeet included (ADR-0011) — appears in the tray with no change here.
fn model_menu_entries() -> Vec<(String, &'static str)> {
    crate::models::MODELS
        .iter()
        .filter(|m| m.multilingual)
        .map(|m| (display_name_for_model(m.name), m.name))
        .collect()
}

/// Turn a models::MODELS name like "small" or "large-v3-turbo" into the
/// title-cased display label shown in the tray ("Small" / "Large-v3-turbo").
///
/// Rules:
/// - Capitalize the first character only.
/// - Leave the rest intact (preserves version numbers like "v3" and
///   hyphenated suffixes like "-turbo").
fn display_name_for_model(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log_files(names: &[&str]) -> Result<Vec<String>, String> {
        Ok(names.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn logs_menu_rows_label_newest_first_with_paths() {
        let dir = Path::new("/logs");
        let rows = logs_menu_rows(
            log_files(&["vtt-2026-10-04.log", "vtt-2026-10-03.log"]),
            dir,
            "2026-10-04",
            "2026-10-03",
        );
        assert_eq!(
            rows,
            vec![
                (
                    "Today (10-04)".to_string(),
                    Some(dir.join("vtt-2026-10-04.log"))
                ),
                (
                    "Yesterday (10-03)".to_string(),
                    Some(dir.join("vtt-2026-10-03.log"))
                ),
            ]
        );
    }

    #[test]
    fn logs_menu_rows_unchanged_inputs_skip_the_refill() {
        let files = &["vtt-2026-10-04.log", "vtt-2026-10-03.log"];
        let dir = Path::new("/logs");
        let a = logs_menu_rows(log_files(files), dir, "2026-10-04", "2026-10-03");
        let b = logs_menu_rows(log_files(files), dir, "2026-10-04", "2026-10-03");
        assert_eq!(a, b, "same files and date must compare equal");
    }

    #[test]
    fn logs_menu_rows_change_when_a_new_day_log_appears() {
        let dir = Path::new("/logs");
        let before = logs_menu_rows(
            log_files(&["vtt-2026-10-01.log"]),
            dir,
            "2026-10-01",
            "2026-09-30",
        );
        let after = logs_menu_rows(
            log_files(&["vtt-2026-10-02.log", "vtt-2026-10-01.log"]),
            dir,
            "2026-10-02",
            "2026-10-01",
        );
        assert_ne!(before, after);
        assert_eq!(after[0].0, "Today (10-02)");
    }

    #[test]
    fn logs_menu_rows_change_at_midnight_with_the_same_files() {
        // Labels roll over before the new day's log is written; the refill
        // must still fire so "Today" never points at yesterday's file.
        let files = &["vtt-2026-10-01.log"];
        let dir = Path::new("/logs");
        let before = logs_menu_rows(log_files(files), dir, "2026-10-01", "2026-09-30");
        let after = logs_menu_rows(log_files(files), dir, "2026-10-02", "2026-10-01");
        assert_ne!(before, after);
        assert_eq!(after[0].0, "Yesterday (10-01)");
    }

    #[test]
    fn logs_menu_rows_placeholders_have_no_path() {
        let dir = Path::new("/logs");
        let empty = logs_menu_rows(log_files(&[]), dir, "2026-10-04", "2026-10-03");
        assert_eq!(empty, vec![("(no logs yet)".to_string(), None)]);

        let err = logs_menu_rows(Err("denied".into()), dir, "2026-10-04", "2026-10-03");
        assert_eq!(
            err,
            vec![("(log dir unreadable: denied)".to_string(), None)]
        );
    }

    #[test]
    fn char_counter_markup_plain_below_200() {
        let (content, is_markup) = char_counter_markup(0);
        assert_eq!(content, "0 / 240 characters");
        assert!(!is_markup);

        let (content, is_markup) = char_counter_markup(199);
        assert_eq!(content, "199 / 240 characters");
        assert!(!is_markup);
    }

    #[test]
    fn char_counter_markup_orange_200_to_229() {
        let (content, is_markup) = char_counter_markup(200);
        assert!(is_markup);
        assert!(content.contains("color='orange'"));
        assert!(content.contains("200 / 240 characters"));

        let (content, is_markup) = char_counter_markup(229);
        assert!(is_markup);
        assert!(content.contains("color='orange'"));
    }

    #[test]
    fn char_counter_markup_red_at_230_plus() {
        let (content, is_markup) = char_counter_markup(230);
        assert!(is_markup);
        assert!(content.contains("color='red'"));

        let (content, is_markup) = char_counter_markup(240);
        assert!(is_markup);
        assert!(content.contains("color='red'"));
        assert!(content.contains("240 / 240 characters"));
    }

    #[test]
    fn char_counter_markup_boundary_exact() {
        // Exact boundary values — 199 is plain, 200 is orange, 229 is orange, 230 is red.
        assert!(!char_counter_markup(199).1);
        assert!(char_counter_markup(200).0.contains("orange"));
        assert!(char_counter_markup(229).0.contains("orange"));
        assert!(char_counter_markup(230).0.contains("red"));
    }

    #[test]
    fn display_name_for_model_capitalises_first_char_only() {
        assert_eq!(display_name_for_model("small"), "Small");
        assert_eq!(display_name_for_model("medium"), "Medium");
        assert_eq!(display_name_for_model("large-v3-turbo"), "Large-v3-turbo");
        assert_eq!(display_name_for_model("large-v3"), "Large-v3");
    }

    #[test]
    fn about_credits_name_the_engine_and_parakeet_licence() {
        let credits = engine_credits("1.9.5");
        assert!(credits.contains("whisper.cpp 1.9.5"), "{credits}");
        assert!(credits.contains("NVIDIA"), "{credits}");
        assert!(credits.contains("CC-BY-4.0"), "{credits}");
    }

    #[test]
    fn model_menu_lists_parakeet_beside_the_whisper_models() {
        let rows = model_menu_entries();
        assert!(
            rows.contains(&("Parakeet-v3".to_string(), "parakeet-v3")),
            "{rows:?}"
        );
        assert!(
            rows.contains(&("Large-v3-turbo".to_string(), "large-v3-turbo")),
            "{rows:?}"
        );
        // English-only variants are reached through the language toggle, not
        // listed as rows of their own.
        assert!(
            rows.iter().all(|(_, key)| !key.ends_with(".en")),
            "{rows:?}"
        );
    }

    #[test]
    fn display_name_for_model_handles_empty_string() {
        assert_eq!(display_name_for_model(""), "");
    }

    #[test]
    fn display_name_for_model_leaves_version_suffixes_intact() {
        // Hypothetical future models — the helper shouldn't mangle them.
        assert_eq!(display_name_for_model("large-v4"), "Large-v4");
        assert_eq!(display_name_for_model("distil-v3.5"), "Distil-v3.5");
    }

    #[test]
    fn display_name_for_model_handles_unicode_first_char() {
        // Defensive — if a model name starts with a non-ASCII letter (unlikely
        // but possible), we use Unicode to_uppercase, not ASCII.
        assert_eq!(display_name_for_model("étude"), "Étude");
    }

    // ── Settings dialog (TASK-VTT204) ──

    const MODEL_KEYS: &[&str] = &["small", "medium", "large-v3-turbo", "parakeet-v3"];

    /// Settings with every field the dialog shows away from its default, and
    /// the fields it does not show set too, so a test can see both kept.
    fn edited_settings(dir: &Path) -> Settings {
        let mut s = Settings::load(dir);
        s.selected_language = "auto".into();
        s.selected_model = "large-v3-turbo".into();
        s.selected_device_index = 2;
        s.voice_prefix = "[Speech] ".into();
        s.initial_prompt = "Amala Vijnana, Kanjin, café".into();
        s.corrections = vec![
            ("ard".into(), "odd".into()),
            ("amala vajrayana".into(), "Amala Vijnana".into()),
        ];
        s.append_newline = false;
        s.newline_type = NewlineType::PlainReturn;
        s.archive_recordings = true;
        s.denoise = true;
        // Not shown in the dialog; Save must leave them as they are.
        s.hotkey_keycode = 78;
        s.money_figures = false;
        s.accents = false;
        s.logging_enabled = false;
        s.archive_dir = "/mnt/voice-archive".into();
        s.archive_max_files = 123;
        s.backend = "ct2".into();
        s
    }

    #[test]
    fn settings_dialog_round_trips_every_field_through_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let original = edited_settings(dir.path());
        let form = SettingsForm::from_settings(&original, MODEL_KEYS, 3);

        // The dialog saves onto a fresh copy loaded from disk, then reloads.
        let mut saved = Settings::load(dir.path());
        form.apply_to(&mut saved, 3);
        saved.save().unwrap();
        let loaded = Settings::load(dir.path());

        assert_eq!(loaded.selected_language, original.selected_language);
        assert_eq!(loaded.selected_model, original.selected_model);
        assert_eq!(loaded.selected_device_index, original.selected_device_index);
        assert_eq!(loaded.voice_prefix, original.voice_prefix);
        assert_eq!(loaded.initial_prompt, original.initial_prompt);
        assert_eq!(loaded.corrections, original.corrections);
        assert_eq!(loaded.append_newline, original.append_newline);
        assert_eq!(loaded.newline_type, original.newline_type);
        assert_eq!(loaded.archive_recordings, original.archive_recordings);
        assert_eq!(loaded.denoise, original.denoise);
        // And the loaded file shows the same form the user saved.
        assert_eq!(SettingsForm::from_settings(&loaded, MODEL_KEYS, 3), form);
    }

    #[test]
    fn settings_dialog_round_trips_the_default_side_of_every_switch() {
        // The edited case proves each switch saves "on"; this proves "off"
        // and the other radio of each pair.
        let dir = tempfile::tempdir().unwrap();
        let mut original = Settings::load(dir.path());
        original.selected_language = "en".into();
        original.selected_model = "small".into();
        original.selected_device_index = -1;
        original.append_newline = true;
        original.newline_type = NewlineType::ShiftReturn;
        original.archive_recordings = false;
        original.denoise = false;
        let form = SettingsForm::from_settings(&original, MODEL_KEYS, 3);

        let mut saved = edited_settings(dir.path());
        form.apply_to(&mut saved, 3);
        saved.save().unwrap();
        let loaded = Settings::load(dir.path());
        assert_eq!(SettingsForm::from_settings(&loaded, MODEL_KEYS, 3), form);
        assert_eq!(loaded.selected_language, "en");
        assert_eq!(loaded.selected_model, "small");
        assert_eq!(loaded.selected_device_index, -1);
        assert!(loaded.append_newline);
        assert_eq!(loaded.newline_type, NewlineType::ShiftReturn);
        assert!(!loaded.archive_recordings);
        assert!(!loaded.denoise);
    }

    #[test]
    fn settings_dialog_save_leaves_fields_it_does_not_show() {
        let dir = tempfile::tempdir().unwrap();
        let live = edited_settings(dir.path());
        let form = SettingsForm::from_settings(&Settings::load(dir.path()), MODEL_KEYS, 3);
        let mut saved = live.clone();
        form.apply_to(&mut saved, 3);
        assert_eq!(saved.hotkey_keycode, live.hotkey_keycode);
        assert_eq!(saved.money_figures, live.money_figures);
        assert_eq!(saved.accents, live.accents);
        assert_eq!(saved.logging_enabled, live.logging_enabled);
        assert_eq!(saved.autostart_initialized, live.autostart_initialized);
        assert_eq!(saved.archive_dir, live.archive_dir);
        assert_eq!(saved.archive_max_files, live.archive_max_files);
        assert_eq!(saved.backend, live.backend);
    }

    #[test]
    fn settings_dialog_shows_an_english_model_as_its_menu_row() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Settings::load(dir.path());
        s.selected_model = "medium.en".into();
        let form = SettingsForm::from_settings(&s, MODEL_KEYS, 0);
        assert_eq!(form.model_key.as_deref(), Some("medium"));
        s.selected_model = "CT2 large-v3-turbo".into();
        let form = SettingsForm::from_settings(&s, MODEL_KEYS, 0);
        assert_eq!(form.model_key.as_deref(), Some("large-v3-turbo"));
    }

    #[test]
    fn settings_dialog_device_rows_map_to_ordinals() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Settings::load(dir.path());
        for (index, row) in [(-1, 0), (0, 1), (2, 3)] {
            s.selected_device_index = index;
            let form = SettingsForm::from_settings(&s, MODEL_KEYS, 3);
            assert_eq!(form.device_row, row, "device {index}");
            let mut back = Settings::load(dir.path());
            form.apply_to(&mut back, 3);
            assert_eq!(back.selected_device_index, index);
        }
        // A saved device that is no longer listed shows, and saves, as Default.
        s.selected_device_index = 5;
        let form = SettingsForm::from_settings(&s, MODEL_KEYS, 3);
        assert_eq!(form.device_row, 0);
    }

    #[test]
    fn settings_dialog_reset_restores_defaults_but_keeps_language_model_device() {
        let dir = tempfile::tempdir().unwrap();
        let mut form = SettingsForm::from_settings(&edited_settings(dir.path()), MODEL_KEYS, 3);
        let before = form.clone();
        form.reset_to_defaults();
        assert_eq!(form.voice_prefix, "[Voice] ");
        assert_eq!(form.initial_prompt, "");
        assert_eq!(form.corrections_text, "");
        assert!(form.append_newline);
        assert!(form.shift_return);
        assert!(
            !form.archive_recordings,
            "Reset must not leave archiving on"
        );
        assert!(!form.denoise);
        assert_eq!(form.language_en, before.language_en);
        assert_eq!(form.model_key, before.model_key);
        assert_eq!(form.device_row, before.device_row);
    }

    #[test]
    fn settings_dialog_height_fits_the_tallest_tab_when_there_is_room() {
        assert_eq!(settings_dialog_height(600, Some(984)), 600);
    }

    #[test]
    fn settings_dialog_height_is_capped_to_the_work_area() {
        // 1080p less a 32 px top bar and a 64 px dock.
        assert_eq!(settings_dialog_height(1200, Some(984)), 984 - 80);
        // A short laptop screen.
        assert_eq!(settings_dialog_height(900, Some(700)), 620);
    }

    #[test]
    fn settings_dialog_height_without_a_monitor_assumes_a_laptop_screen() {
        assert_eq!(settings_dialog_height(1200, None), 620);
        assert_eq!(settings_dialog_height(1200, Some(0)), 620);
    }

    #[test]
    fn settings_dialog_height_never_goes_below_its_floor() {
        assert_eq!(settings_dialog_height(1200, Some(200)), 240);
        assert_eq!(settings_dialog_height(100, Some(984)), 240);
    }

    /// Run GTK's pending events for about `ms` milliseconds.
    fn pump_gtk(ms: u64) {
        let until = std::time::Instant::now() + std::time::Duration::from_millis(ms);
        for _ in 0..10_000 {
            if std::time::Instant::now() >= until {
                break;
            }
            for _ in 0..1_000 {
                if !gtk::events_pending() {
                    break;
                }
                gtk::main_iteration();
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// Hold the window on screen for an outside screenshot when asked.
    fn hold_for_screenshot(stage: &str) {
        if let Some(ms) = std::env::var("VTT_DIALOG_HOLD_MS")
            .ok()
            .and_then(|v| v.parse().ok())
        {
            eprintln!("STAGE {stage}");
            pump_gtk(ms);
        }
    }

    fn assert_fully_inside(widget: &gtk::Button, window: &gtk::Window, at: &str) {
        let (_, window_height) = window.size();
        let (x, y) = widget
            .translate_coordinates(window, 0, 0)
            .expect("button is inside the window");
        let bottom = y + widget.allocated_height();
        assert!(widget.is_mapped(), "{at}: button not mapped");
        assert!(
            x >= 0 && y >= 0 && bottom <= window_height,
            "{at}: button spans y {y}..{bottom}, window is {window_height} high"
        );
    }

    /// Opens the real dialog, so it needs an X display:
    /// `xvfb-run -s "-screen 0 1920x1080x24" cargo test -- --ignored settings_dialog_on_screen`
    #[test]
    #[ignore = "needs an X display; run under xvfb-run"]
    fn settings_dialog_on_screen_keeps_save_visible_at_every_height() {
        gtk::init().expect("an X display: run under xvfb-run");
        let dir = tempfile::tempdir().unwrap();
        let settings = edited_settings(dir.path());
        let devices = vec![
            "Built-in Audio Analog Stereo".to_string(),
            "USB Microphone".to_string(),
            "Webcam Microphone".to_string(),
        ];
        let dialog = build_settings_dialog(&settings, dir.path(), &devices);
        dialog.window.show_all();
        pump_gtk(400);

        let workarea = pointer_monitor_workarea_height().expect("a monitor");
        let (_, opened) = dialog.window.size();
        eprintln!("workarea {workarea}, dialog opened {opened} high");
        assert!(
            opened <= workarea - SETTINGS_DIALOG_FRAME_ALLOWANCE,
            "dialog {opened} high on a {workarea} work area"
        );
        assert_fully_inside(&dialog.save_btn, &dialog.window, "opened");
        assert_fully_inside(&dialog.cancel_btn, &dialog.window, "opened");

        // Where the screen has room, no tab needs scrolling at the opening size.
        let outer = dialog
            .window
            .child()
            .unwrap()
            .downcast::<gtk::Box>()
            .unwrap();
        let notebook = outer.children()[0]
            .clone()
            .downcast::<gtk::Notebook>()
            .unwrap();
        if workarea >= 900 {
            for page in 0..notebook.n_pages() {
                notebook.set_current_page(Some(page));
                pump_gtk(150);
                let scroller = notebook
                    .nth_page(Some(page))
                    .unwrap()
                    .downcast::<gtk::ScrolledWindow>()
                    .unwrap();
                let adj = scroller.vadjustment();
                eprintln!(
                    "tab {page}: content {} in view {}",
                    adj.upper(),
                    adj.page_size()
                );
                assert!(
                    adj.upper() <= adj.page_size() + 1.0,
                    "tab {page} needs scrolling: content {} in view {}",
                    adj.upper(),
                    adj.page_size()
                );
            }
            notebook.set_current_page(Some(0));
            pump_gtk(150);
        }
        hold_for_screenshot("opened");

        // Shorter than it opened: the tab scrolls, the button row stays.
        dialog.window.resize(SETTINGS_DIALOG_WIDTH, 260);
        pump_gtk(400);
        let (_, short) = dialog.window.size();
        eprintln!("dialog resized to {short} high");
        assert!(short < opened, "window would not shrink below {opened}");
        assert_fully_inside(&dialog.save_btn, &dialog.window, "shortened");
        assert_fully_inside(&dialog.cancel_btn, &dialog.window, "shortened");
        hold_for_screenshot("shortened");
    }
}
