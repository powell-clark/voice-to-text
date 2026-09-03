use super::{LastTranscription, UiMessage};
use crate::corrections;
use crate::hotkey::{self, HotkeyCmd};
use crate::logging;
use crate::settings::{NewlineType, Settings};
use glib::clone;
use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};
use std::cell::RefCell;
use std::path::Path;
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
    initializing: bool,
}

impl Tray {
    /// Build the tray icon and menu. Returns the Tray and a glib::Sender for
    /// cross-thread UI updates.
    pub fn new(
        settings: Arc<RwLock<Settings>>,
        _config_dir: &Path,
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

        // --- Hotkey ---
        let hk_code = settings.read().unwrap().hotkey_keycode;
        let hk_name = hotkey::get_key_name(hk_code);
        let hotkey_item = gtk::MenuItem::with_label(&format!("Hotkey: {}", hk_name));
        menu.append(&hotkey_item);

        // --- Customize Transcription Settings ---
        let prompt_item = gtk::MenuItem::with_label("Customize Transcription Settings...");
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
        // Rebuilt when the parent tray menu is shown (via menu.connect_show below),
        // not on logs_item's connect_activate — that fires AFTER GTK has already
        // revealed the submenu, so the first open sees stale content.
        let logs_item = gtk::MenuItem::with_label("Logs");
        logs_item.set_submenu(Some(&build_logs_menu()));
        menu.append(&logs_item);

        menu.append(&gtk::SeparatorMenuItem::new());

        // --- About ---
        let about_item = gtk::MenuItem::with_label("About Voice to Text");
        menu.append(&about_item);
        menu.append(&gtk::SeparatorMenuItem::new());

        // --- Quit ---
        let quit_item = gtk::MenuItem::with_label("Quit");
        menu.append(&quit_item);

        // Rebuild Logs submenu every time the tray menu is shown — catches new
        // log files written since the last open. This fires before GTK reveals
        // any child submenu, so first-hover already sees fresh contents.
        {
            let logs_item_ref = logs_item.clone();
            menu.connect_show(move |_| {
                let fresh = build_logs_menu();
                fresh.show_all();
                logs_item_ref.set_submenu(Some(&fresh));
            });
        }

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

        // Prompt settings
        {
            let st = state.clone();
            prompt_item.connect_activate(move |_| {
                show_prompt_dialog(&st);
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

        // --- Cross-thread UI update channel (mpsc, polled via glib timeout) ---
        let (ui_tx, ui_rx) = std::sync::mpsc::channel::<UiMessage>();

        let status_item_clone = status_item.clone();
        let indicator_clone = indicator.clone();
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

    // Derive the menu from models::MODELS instead of a hardcoded list.
    // Each multilingual entry becomes one menu row (with an English toggle
    // swapping .en variants at transcription time via resolve_variant).
    // Adding a model to MODELS now automatically adds it to the tray.
    let model_entries: Vec<(String, &str)> = crate::models::MODELS
        .iter()
        .filter(|m| m.multilingual)
        .map(|m| (display_name_for_model(m.name), m.name))
        .collect();
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

fn save_settings(state: &Rc<RefCell<TrayState>>) {
    let s = state.borrow();
    let settings = s.settings.read().unwrap();
    if let Err(e) = settings.save() {
        crate::vtt_log!("Failed to save settings: {}", e);
    }
    drop(settings);
}

// ─── Logs submenu ───────────────────────────────────────────────

fn build_logs_menu() -> gtk::Menu {
    let menu = gtk::Menu::new();
    let log_dir = logging::get_dir();

    let entries = match std::fs::read_dir(&log_dir) {
        Ok(e) => e,
        Err(e) => {
            let msg = format!("(log dir unreadable: {})", e);
            let item = gtk::MenuItem::with_label(&msg);
            item.set_sensitive(false);
            menu.append(&item);
            return menu;
        }
    };

    let mut files: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("vtt-") && name.ends_with(".log") {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    files.sort_unstable();
    files.reverse(); // Newest first (filenames are ISO dates, lexicographic sort reversed == newest first)

    if files.is_empty() {
        let empty = gtk::MenuItem::with_label("(no logs yet)");
        empty.set_sensitive(false);
        menu.append(&empty);
        return menu;
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    for filename in &files {
        let label = format_log_label(filename, &today, &yesterday);

        let item = gtk::MenuItem::with_label(&label);
        let full_path = log_dir.join(filename).to_string_lossy().to_string();
        item.connect_activate(move |_| {
            open_file(&full_path);
        });
        menu.append(&item);
    }

    menu
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
         Release to transcribe.",
        env!("CARGO_PKG_VERSION")
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

fn show_prompt_dialog(state: &Rc<RefCell<TrayState>>) {
    let s = state.borrow();
    let settings = s.settings.read().unwrap().clone();
    drop(s);

    let dialog = gtk::Window::new(gtk::WindowType::Toplevel);
    dialog.set_title("Customize Transcription Settings");
    // Six sections now. The old fixed 500x340 fitted five and clipped the
    // button row when the corrections editor landed (TASK-VTT144), so the
    // window grows and is allowed to resize rather than hiding Save.
    dialog.set_default_size(520, 560);
    dialog.set_resizable(true);
    dialog.set_position(gtk::WindowPosition::Center);
    dialog.set_border_width(20);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    dialog.add(&vbox);

    // Voice prefix
    let prefix_label = gtk::Label::new(None);
    prefix_label.set_markup("<b>Voice Prefix (prepended to every transcription):</b>");
    prefix_label.set_halign(gtk::Align::Start);
    vbox.pack_start(&prefix_label, false, false, 0);

    let prefix_entry = gtk::Entry::new();
    prefix_entry.set_text(&settings.voice_prefix);
    prefix_entry.set_placeholder_text(Some("e.g., [voice] "));
    vbox.pack_start(&prefix_entry, false, false, 0);

    vbox.pack_start(&gtk::Label::new(Some("")), false, false, 0); // spacer

    // Initial prompt
    let prompt_label = gtk::Label::new(None);
    prompt_label
        .set_markup("<b>Initial Prompt (helps Whisper recognize your voice, max 240 chars):</b>");
    prompt_label.set_halign(gtk::Align::Start);
    vbox.pack_start(&prompt_label, false, false, 0);

    let scrolled = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scrolled.set_size_request(-1, 70);
    vbox.pack_start(&scrolled, true, true, 0);

    let text_view = gtk::TextView::new();
    text_view.set_wrap_mode(gtk::WrapMode::WordChar);
    let buffer = text_view.buffer().unwrap();
    buffer.set_text(&settings.initial_prompt);
    scrolled.add(&text_view);

    // Character counter
    let counter = gtk::Label::new(None);
    counter.set_halign(gtk::Align::End);
    update_char_counter(&buffer, &counter);
    vbox.pack_start(&counter, false, false, 0);

    let counter_clone = counter.clone();
    buffer.connect_changed(move |buf| {
        // Enforce 240 char limit
        let count = buf.char_count();
        if count > 240 {
            let start = buf.start_iter();
            let mut end = buf.start_iter();
            end.set_offset(240);
            if let Some(text) = buf.text(&start, &end, false) {
                buf.set_text(&text);
            }
        }
        update_char_counter(buf, &counter_clone);
    });

    vbox.pack_start(&gtk::Label::new(Some("")), false, false, 0); // spacer

    // Corrections dictionary. The format is stated in the label because
    // `parse_pairs` drops a malformed row silently — a user who types `->`
    // instead of `=>` would otherwise watch their rule vanish on save with no
    // explanation (TASK-VTT144 pre-mortem).
    let corrections_label = gtk::Label::new(None);
    corrections_label
        .set_markup("<b>Corrections (one per line, <tt>misheard =&gt; correct</tt>):</b>");
    corrections_label.set_halign(gtk::Align::Start);
    vbox.pack_start(&corrections_label, false, false, 0);

    let corrections_scrolled =
        gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    corrections_scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    corrections_scrolled.set_size_request(-1, 110);
    vbox.pack_start(&corrections_scrolled, true, true, 0);

    let corrections_view = gtk::TextView::new();
    corrections_view.set_wrap_mode(gtk::WrapMode::WordChar);
    let corrections_buffer = corrections_view.buffer().unwrap();
    corrections_buffer.set_text(&corrections::format_pairs(&settings.corrections));
    corrections_scrolled.add(&corrections_view);

    let corrections_hint = gtk::Label::new(Some(
        "Whole words only, case-insensitive. Empty the box to remove them all.",
    ));
    corrections_hint.set_halign(gtk::Align::Start);
    vbox.pack_start(&corrections_hint, false, false, 0);

    vbox.pack_start(&gtk::Label::new(Some("")), false, false, 0); // spacer

    // Newline toggle
    let newline_toggle = gtk::CheckButton::with_label("Insert newline between transcriptions");
    newline_toggle.set_active(settings.append_newline);
    vbox.pack_start(&newline_toggle, false, false, 0);

    // Newline type radios
    let nl_label = gtk::Label::new(Some("Newline key behavior:"));
    nl_label.set_halign(gtk::Align::Start);
    vbox.pack_start(&nl_label, false, false, 5);

    let plain_radio = gtk::RadioButton::with_label("Plain Return (may send messages in chat apps)");
    vbox.pack_start(&plain_radio, false, false, 0);

    let shift_radio = gtk::RadioButton::with_label_from_widget(
        &plain_radio,
        "Shift+Return (safer, won't send messages)",
    );
    vbox.pack_start(&shift_radio, false, false, 0);

    if settings.newline_type == NewlineType::ShiftReturn {
        shift_radio.set_active(true);
    } else {
        plain_radio.set_active(true);
    }

    // Button row
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    vbox.pack_start(&btn_box, false, false, 0);

    let reset_btn = gtk::Button::with_label("Reset Default");
    reset_btn.set_size_request(120, -1);
    btn_box.pack_start(&reset_btn, false, false, 0);

    btn_box.pack_start(&gtk::Label::new(Some("")), true, true, 0); // spacer

    let cancel_btn = gtk::Button::with_label("Cancel");
    cancel_btn.set_size_request(80, -1);
    btn_box.pack_start(&cancel_btn, false, false, 0);

    let save_btn = gtk::Button::with_label("Save");
    save_btn.set_size_request(80, -1);
    btn_box.pack_start(&save_btn, false, false, 0);

    // Reset
    {
        let pe = prefix_entry.clone();
        let buf = text_view.buffer().unwrap();
        let cb = corrections_buffer.clone();
        let nt = newline_toggle.clone();
        let sr = shift_radio.clone();
        reset_btn.connect_clicked(move |_| {
            pe.set_text("[Voice] ");
            buf.set_text("");
            cb.set_text("");
            nt.set_active(true);
            sr.set_active(true);
        });
    }

    // Cancel
    {
        let d = dialog.clone();
        cancel_btn.connect_clicked(move |_| unsafe {
            d.destroy();
        });
    }

    // Save
    {
        let st = state.clone();
        let d = dialog.clone();
        let pe = prefix_entry.clone();
        let buf = text_view.buffer().unwrap();
        let cb = corrections_buffer;
        let nt = newline_toggle;
        let sr = shift_radio;
        save_btn.connect_clicked(move |_| {
            let s = st.borrow();
            let mut settings = s.settings.write().unwrap();

            settings.voice_prefix = pe.text().to_string();

            settings.initial_prompt = buffer_text(&buf);

            // An emptied box means "remove them all", so this assigns
            // unconditionally rather than skipping on empty (TASK-VTT144).
            settings.corrections = corrections::parse_pairs(&buffer_text(&cb));

            settings.append_newline = nt.is_active();
            settings.newline_type = if sr.is_active() {
                NewlineType::ShiftReturn
            } else {
                NewlineType::PlainReturn
            };

            if let Err(e) = settings.save() {
                crate::vtt_log!("Failed to save settings: {}", e);
            }

            crate::vtt_log!(
                "Settings updated: prefix='{}', prompt='{}', corrections={}",
                settings.voice_prefix,
                settings.initial_prompt,
                settings.corrections.len()
            );

            drop(settings);
            drop(s);
            unsafe {
                d.destroy();
            }
        });
    }

    dialog.show_all();
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

/// Turn a log filename like `vtt-2026-04-20.log` into a menu label.
/// - Today's file → `Today (MM-DD)`
/// - Yesterday's → `Yesterday (MM-DD)`
/// - Older       → `YYYY-MM-DD`
/// - Malformed   → the filename itself (fallback)
///
/// Pure function — no I/O, no GTK, trivially testable.
fn format_log_label(filename: &str, today: &str, yesterday: &str) -> String {
    let date = filename
        .strip_prefix("vtt-")
        .and_then(|s| s.strip_suffix(".log"))
        .unwrap_or(filename);

    if date == today && date.len() >= 7 {
        format!("Today ({})", &date[5..])
    } else if date == yesterday && date.len() >= 7 {
        format!("Yesterday ({})", &date[5..])
    } else {
        date.to_string()
    }
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

    #[test]
    fn format_log_label_today_gives_friendly_label() {
        let s = format_log_label("vtt-2026-04-20.log", "2026-04-20", "2026-04-19");
        assert_eq!(s, "Today (04-20)");
    }

    #[test]
    fn format_log_label_yesterday_gives_friendly_label() {
        let s = format_log_label("vtt-2026-04-19.log", "2026-04-20", "2026-04-19");
        assert_eq!(s, "Yesterday (04-19)");
    }

    #[test]
    fn format_log_label_older_returns_full_iso_date() {
        let s = format_log_label("vtt-2026-01-15.log", "2026-04-20", "2026-04-19");
        assert_eq!(s, "2026-01-15");
    }

    #[test]
    fn format_log_label_malformed_filename_returns_filename() {
        let s = format_log_label("not-a-log-file.txt", "2026-04-20", "2026-04-19");
        assert_eq!(s, "not-a-log-file.txt");
    }

    #[test]
    fn format_log_label_wrong_prefix_returns_filename() {
        let s = format_log_label("vtt2026-04-20.log", "2026-04-20", "2026-04-19");
        assert_eq!(
            s, "vtt2026-04-20.log",
            "filename without proper vtt- prefix falls through"
        );
    }

    #[test]
    fn format_log_label_short_dates_dont_panic() {
        // Sanity check the bounds guard — a "today" of "2026" (len 4) must not panic
        // from string slicing at [5..].
        let s = format_log_label("vtt-2026.log", "2026", "2025");
        assert_eq!(
            s, "2026",
            "short dates bypass the friendly label and fall through"
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
}
