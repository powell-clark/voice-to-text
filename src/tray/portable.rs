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
}

struct MenuIds {
    status: MenuItem,
    quit: MenuItem,
    about: MenuItem,
    logging: MenuItem,
    autostart: Option<CheckMenuItem>,
    copy_last: MenuItem,
    models: Vec<(CheckMenuItem, String)>,
    lang_en: CheckMenuItem,
    lang_multi: CheckMenuItem,
}

// Commands from the event-matching thread (Send-safe) to the main thread.
enum MenuCmd {
    Quit,
    About,
    LoggingToggle,
    AutostartToggle,
    CopyLastTranscription,
    LanguageSel(String),
    ModelSel(String),
}

impl Tray {
    pub fn new(
        settings: Arc<RwLock<Settings>>,
        _config_dir: &Path,
        last_transcription: LastTranscription,
    ) -> anyhow::Result<(Self, UiSender)> {
        let s = settings.read().unwrap();
        let current_model = s.selected_model.clone();
        let current_lang = s.selected_language.clone();
        let logging_enabled = s.logging_enabled;
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
            models,
            lang_en,
            lang_multi,
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
        let lang_en_id = ids.lang_en.id().clone();
        let lang_multi_id = ids.lang_multi.id().clone();
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
                    } else if id == lang_en_id {
                        Some(MenuCmd::LanguageSel("en".into()))
                    } else if id == lang_multi_id {
                        Some(MenuCmd::LanguageSel("auto".into()))
                    } else {
                        model_ids
                            .iter()
                            .find(|(mid, _)| *mid == id)
                            .map(|(_, name)| MenuCmd::ModelSel(name.clone()))
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
            },
            ui_tx,
        ))
    }

    /// Process pending menu commands. Call from the main event loop at ~10 Hz.
    pub fn poll_menu(&mut self) {
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
            }
        }
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
