/// Portable tray implementation using tray-icon + muda (macOS + Windows)
use super::{UiMessage, UiSender};
use crate::hotkey;
use crate::logging;
use crate::settings::Settings;
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};

use muda::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{TrayIcon, TrayIconBuilder};

pub struct Tray {
    _tray_icon: TrayIcon,
}

struct MenuIds {
    quit: MenuItem,
    about: MenuItem,
    logging: MenuItem,
    models: Vec<(CheckMenuItem, String)>,
    lang_en: CheckMenuItem,
    lang_multi: CheckMenuItem,
}

impl Tray {
    pub fn new(
        settings: Arc<RwLock<Settings>>,
        _config_dir: &Path,
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

        // Models
        let model_sub = Submenu::new(&format!("Model: {}", current_model), true);
        let model_names = [
            "W base",
            "W small",
            "W medium",
            "W large",
            "CT2 base",
            "CT2 small",
            "CT2 distil-large-v3.5",
            "CT2 large-v3-turbo",
        ];
        let mut models = Vec::new();
        for &name in &model_names {
            let checked = name == current_model;
            let item = CheckMenuItem::new(name, true, checked, None);
            model_sub.append(&item)?;
            models.push((item, name.to_string()));
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

        // About
        let about = MenuItem::new("About Voice to Text", true, None);
        menu.append(&about)?;
        menu.append(&PredefinedMenuItem::separator())?;

        // Quit
        let quit = MenuItem::new("Quit", true, None);
        menu.append(&quit)?;

        // Create tray icon (simple colored dot)
        let icon = create_icon(0, 180, 0); // Green = ready
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Voice to Text")
            .with_icon(icon)
            .build()?;

        // UI channel
        let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>();

        // Gather menu item IDs
        let ids = MenuIds {
            quit,
            about,
            logging,
            models,
            lang_en,
            lang_multi,
        };

        // Menu event handler thread
        let menu_rx = MenuEvent::receiver().clone();
        let settings_clone = settings.clone();
        std::thread::Builder::new()
            .name("menu-events".into())
            .spawn(move || {
                handle_menu_events(&menu_rx, ids, settings_clone, status, lang_sub, model_sub);
            })?;

        // UI update handler (polls mpsc in a thread, updates tray)
        // Note: on macOS/Windows, tray icon updates must happen from the right thread.
        // For now, status updates are logged since tray-icon doesn't support
        // dynamic label changes easily after creation.
        std::thread::Builder::new()
            .name("ui-updates".into())
            .spawn(move || {
                while let Ok(msg) = ui_rx.recv() {
                    match msg {
                        UiMessage::SetStatus(text) => {
                            crate::vtt_log!("[UI] Status: {}", text);
                        }
                        UiMessage::SetIcon(icon) => {
                            crate::vtt_log!("[UI] Icon: {}", icon);
                        }
                    }
                }
            })?;

        Ok((
            Tray {
                _tray_icon: tray_icon,
            },
            ui_tx,
        ))
    }
}

fn handle_menu_events(
    menu_rx: &crossbeam_channel::Receiver<MenuEvent>,
    ids: MenuIds,
    settings: Arc<RwLock<Settings>>,
    _status: MenuItem,
    _lang_sub: Submenu,
    _model_sub: Submenu,
) {
    loop {
        if let Ok(event) = menu_rx.recv() {
            let id = event.id().clone();

            // Quit
            if id == *ids.quit.id() {
                crate::vtt_log!("Quit requested");
                std::process::exit(0);
            }

            // About
            if id == *ids.about.id() {
                crate::vtt_log!("About: Voice to Text 2.0 (Rust) — https://github.com/powell-clark/voice-to-text");
            }

            // Logging toggle
            if id == *ids.logging.id() {
                let mut s = settings.write().unwrap();
                s.logging_enabled = !s.logging_enabled;
                logging::set_enabled(s.logging_enabled);
                ids.logging.set_text(if s.logging_enabled {
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

            // Language
            if id == *ids.lang_en.id() {
                settings.write().unwrap().selected_language = "en".into();
                ids.lang_en.set_checked(true);
                ids.lang_multi.set_checked(false);
                settings.read().unwrap().save().ok();
                crate::vtt_log!("Language changed to English");
            }
            if id == *ids.lang_multi.id() {
                settings.write().unwrap().selected_language = "auto".into();
                ids.lang_en.set_checked(false);
                ids.lang_multi.set_checked(true);
                settings.read().unwrap().save().ok();
                crate::vtt_log!("Language changed to Multilingual");
            }

            // Model selection
            for (item, name) in &ids.models {
                if id == *item.id() {
                    settings.write().unwrap().selected_model = name.clone();
                    // Uncheck all, check selected
                    for (other, _) in &ids.models {
                        other.set_checked(false);
                    }
                    item.set_checked(true);
                    settings.read().unwrap().save().ok();
                    crate::vtt_log!("Model changed to {}", name);
                    break;
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
