mod audio;
mod hotkey;
mod logging;
mod settings;
mod transcribe;
mod tray;
mod typing;

use audio::RecordingResult;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Audio file sent to the transcription worker
enum WorkItem {
    Audio(PathBuf),
    Truncated(PathBuf),
}

fn main() -> anyhow::Result<()> {
    // Platform-specific init
    #[cfg(target_os = "linux")]
    {
        unsafe { x11::xlib::XInitThreads(); }
        gtk::init()?;
    }

    // Config directory
    let config_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("voice-to-text");

    // Initialize logging
    logging::init(&config_dir);
    vtt_log!("===========================================");
    vtt_log!("Voice to Text - Starting (Rust 2.0)");
    vtt_log!("===========================================");

    // Singleton lock
    let _lock_fd = singleton_lock(&config_dir)?;

    // Signal handler
    let running = Arc::new(AtomicBool::new(true));
    {
        let r = running.clone();
        ctrlc_handler(move || {
            eprintln!("Signal received, shutting down");
            r.store(false, Ordering::SeqCst);
            #[cfg(target_os = "linux")]
            glib::idle_add_once(gtk::main_quit);
        });
    }

    // Clean up old WAV files from /tmp
    cleanup_old_wavs();

    // Load settings
    let settings = Arc::new(RwLock::new(settings::Settings::load(&config_dir)));

    // Shared state
    let recording = Arc::new(AtomicBool::new(false));
    let typing_active = Arc::new(AtomicBool::new(false));
    let typing_has_output = Arc::new(AtomicBool::new(false));

    // Initialize audio
    let audio = Arc::new(audio::Audio::new()?);

    // Buffer full notification
    audio.set_buffer_full_callback(|| {
        let show_notification = || {
            if let Err(e) = notify_rust::Notification::new()
                .summary("Voice to Text")
                .body("Recording limit reached - release key to transcribe")
                .icon("dialog-information")
                .timeout(3000)
                .show()
            {
                crate::vtt_log!("Notification failed: {}", e);
            }
        };
        #[cfg(target_os = "linux")]
        glib::idle_add_once(show_notification);
        #[cfg(not(target_os = "linux"))]
        show_notification();
    });

    // Transcription work channel
    let (work_tx, work_rx) = mpsc::channel::<WorkItem>();

    // Create tray (returns UI message sender for cross-thread updates)
    let (tray, ui_tx) = tray::Tray::new(settings.clone(), &config_dir)?;
    let _ = tray; // keep alive

    // Worker thread — transcribes audio and types result
    let worker_settings = settings.clone();
    let worker_typing_active = typing_active.clone();
    let worker_typing_has_output = typing_has_output.clone();
    let worker_running = running.clone();
    let worker_ui_tx = ui_tx.clone();
    let worker_config_dir = config_dir.clone();

    thread::Builder::new()
        .name("transcription-worker".into())
        .spawn(move || {
            transcription_worker(
                work_rx,
                worker_settings,
                worker_typing_active,
                worker_typing_has_output,
                worker_running,
                worker_ui_tx,
                worker_config_dir,
            );
        })?;

    // Hotkey monitor
    let hk_keycode = settings.read().unwrap().hotkey_keycode;
    let hk_recording = recording.clone();
    let hk_audio = audio.clone();
    let hk_typing_active = typing_active.clone();
    let hk_work_tx = work_tx.clone();
    let hk_ui_tx = ui_tx.clone();
    let hk_recording_start = Arc::new(std::sync::Mutex::new(Instant::now()));

    let hotkey_tx = hotkey::start_monitor(hk_keycode, move |event| {
        match event {
            hotkey::KeyEvent::Down => {
                if hk_recording.load(Ordering::SeqCst) {
                    return;
                }

                // Wait for typing to finish (max 30s)
                let start = Instant::now();
                while hk_typing_active.load(Ordering::Relaxed)
                    && start.elapsed() < Duration::from_secs(30)
                {
                    thread::sleep(Duration::from_millis(1));
                }

                vtt_log!("Key pressed - starting recording");
                *hk_recording_start.lock().unwrap() = Instant::now();
                hk_recording.store(true, Ordering::SeqCst);
                hk_audio.start_recording();
                hk_ui_tx.send(tray::UiMessage::SetStatus("Recording...".into())).ok();
                hk_ui_tx.send(tray::UiMessage::SetIcon("recording".into())).ok();
            }
            hotkey::KeyEvent::Up => {
                if !hk_recording.load(Ordering::SeqCst) {
                    return;
                }

                // Guard against stale releases (< 150ms after start)
                let elapsed = hk_recording_start.lock().unwrap().elapsed();
                if elapsed < Duration::from_millis(150) {
                    vtt_log!("Ignoring stale KeyRelease ({}ms)", elapsed.as_millis());
                    return;
                }

                vtt_log!("Key released - stopping recording");
                hk_recording.store(false, Ordering::SeqCst);

                match hk_audio.stop_recording() {
                    Some(RecordingResult::Audio(path)) => {
                        vtt_log!("Recording saved: {}", path.display());
                        hk_ui_tx.send(tray::UiMessage::SetStatus("Loading model...".into())).ok();
                        hk_ui_tx.send(tray::UiMessage::SetIcon("processing".into())).ok();
                        hk_work_tx.send(WorkItem::Audio(path)).ok();
                    }
                    Some(RecordingResult::MaxLength(path)) => {
                        vtt_log!("Max recording length reached");
                        hk_ui_tx.send(tray::UiMessage::SetStatus("Loading model...".into())).ok();
                        hk_ui_tx.send(tray::UiMessage::SetIcon("processing".into())).ok();
                        hk_work_tx.send(WorkItem::Truncated(path)).ok();
                    }
                    Some(RecordingResult::TooShort(_)) => {
                        hk_ui_tx.send(tray::UiMessage::SetStatus("Ready".into())).ok();
                        hk_ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();
                    }
                    Some(RecordingResult::TooQuiet(_)) => {
                        hk_ui_tx.send(tray::UiMessage::SetStatus("Ready".into())).ok();
                        hk_ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();
                    }
                    None => {
                        vtt_log!("Recording returned None");
                        hk_ui_tx.send(tray::UiMessage::SetStatus("Ready".into())).ok();
                        hk_ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();
                    }
                }
            }
        }
    })?;

    // Apply loaded hotkey
    if hk_keycode >= 8 {
        vtt_log!("Applied custom hotkey from settings: keycode {}", hk_keycode);
    }

    vtt_log!("All systems initialized");
    ui_tx.send(tray::UiMessage::SetStatus("Ready".into())).ok();
    ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();

    // Run platform event loop (blocks until quit)
    #[cfg(target_os = "linux")]
    gtk::main();

    #[cfg(not(target_os = "linux"))]
    {
        // On macOS/Windows, block main thread until signal
        loop {
            thread::sleep(Duration::from_millis(100));
            if !running.load(Ordering::Relaxed) {
                break;
            }
        }
    }

    // Shutdown
    vtt_log!("Shutting down...");
    running.store(false, Ordering::SeqCst);
    hotkey_tx.send(hotkey::HotkeyCmd::Stop).ok();
    drop(work_tx); // Close channel to unblock worker

    vtt_log!("Shutdown complete");
    Ok(())
}

// ─── Transcription worker ───────────────────────────────────────

fn transcription_worker(
    rx: mpsc::Receiver<WorkItem>,
    settings: Arc<RwLock<settings::Settings>>,
    typing_active: Arc<AtomicBool>,
    typing_has_output: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    ui_tx: tray::UiSender,
    config_dir: PathBuf,
) {
    vtt_log!("Transcription worker started");
    let typer = match typing::Typer::new() {
        Ok(t) => t,
        Err(e) => {
            vtt_log!("Failed to init typer: {}", e);
            return;
        }
    };

    while running.load(Ordering::Relaxed) {
        let item = match rx.recv() {
            Ok(item) => item,
            Err(_) => break, // Channel closed
        };

        let (audio_path, is_truncated) = match &item {
            WorkItem::Audio(p) => (p.clone(), false),
            WorkItem::Truncated(p) => (p.clone(), true),
        };

        vtt_log!(
            "Processing: {}{}",
            audio_path.display(),
            if is_truncated { " (TRUNCATED)" } else { "" }
        );
        ui_tx.send(tray::UiMessage::SetStatus("Transcribing...".into())).ok();
        ui_tx.send(tray::UiMessage::SetIcon("processing".into())).ok();

        // Read settings snapshot
        let s = settings.read().unwrap();
        let model = s.selected_model.clone();
        let language = s.selected_language.clone();
        let prompt = s.initial_prompt.clone();
        let prefix = s.voice_prefix.clone();
        let append_newline = s.append_newline;
        let newline_type = s.newline_type;
        drop(s);

        // Transcribe
        let text = transcribe::transcribe_audio(&audio_path, &model, &language, &prompt);

        if let Some(text) = text {
            let trimmed = text.trim();

            // Skip blank/music markers
            if trimmed.is_empty()
                || trimmed.eq_ignore_ascii_case("[BLANK_AUDIO]")
                || trimmed.eq_ignore_ascii_case("[MUSIC PLAYING]")
            {
                vtt_log!("Skipping blank transcription");
            } else if trimmed.chars().any(|c| c.is_alphanumeric()) {
                vtt_log!("Transcription: {}", trimmed);

                // Build final text with prefix
                let final_text = if is_truncated {
                    format!("[Truncated] {}{}", prefix, trimmed)
                } else if !trimmed.starts_with(&prefix) {
                    format!("{}{}", prefix, trimmed)
                } else {
                    trimmed.to_string()
                };

                // Type the result
                typing_active.store(true, Ordering::SeqCst);
                if append_newline && typing_has_output.load(Ordering::Relaxed) {
                    typer.type_text("\n", newline_type);
                }
                typer.type_text(&final_text, newline_type);
                typing_active.store(false, Ordering::SeqCst);
                typing_has_output.store(true, Ordering::Relaxed);
            } else {
                vtt_log!("Skipping punctuation-only transcription: {}", trimmed);
            }
        }

        // Save recording backup and clean up
        save_and_cleanup(&audio_path, &config_dir);

        ui_tx.send(tray::UiMessage::SetStatus("Ready".into())).ok();
        ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();
    }

    vtt_log!("Transcription worker stopped");
}

fn save_and_cleanup(audio_path: &std::path::Path, config_dir: &std::path::Path) {
    let recordings_dir = config_dir.join("recordings");
    std::fs::create_dir_all(&recordings_dir).ok();

    if let Some(filename) = audio_path.file_name() {
        let dest = recordings_dir.join(filename);
        if std::fs::copy(audio_path, &dest).is_ok() {
            std::fs::remove_file(audio_path).ok();
            vtt_log!("Saved recording to {}", dest.display());
            prune_recordings(&recordings_dir, 20);
            return;
        }
    }
    // Fallback: just delete
    std::fs::remove_file(audio_path).ok();
}

fn prune_recordings(dir: &std::path::Path, max: usize) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with(".wav")
        })
        .filter_map(|e| {
            let mtime = e.metadata().ok()?.modified().ok()?;
            Some((e.path(), mtime))
        })
        .collect();

    if entries.len() <= max {
        return;
    }

    entries.sort_by(|a, b| b.1.cmp(&a.1)); // newest first
    for (path, _) in &entries[max..] {
        if std::fs::remove_file(path).is_ok() {
            vtt_log!("Pruned old recording: {}", path.display());
        }
    }
}

// ─── Utilities ──────────────────────────────────────────────────

fn singleton_lock(config_dir: &std::path::Path) -> anyhow::Result<std::fs::File> {
    use std::os::unix::io::AsRawFd;
    std::fs::create_dir_all(config_dir)?;
    let lock_path = config_dir.join("vtt-linux.lock");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)?;

    // flock(LOCK_EX | LOCK_NB)
    let ret = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if ret != 0 {
        anyhow::bail!("Another instance of vtt-linux is already running");
    }
    Ok(file)
}

fn cleanup_old_wavs() {
    let cutoff = std::time::SystemTime::now() - Duration::from_secs(3600);

    if let Ok(entries) = std::fs::read_dir("/tmp") {
        let mut cleaned = 0;
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("vtt_recording_") || !name.ends_with(".wav") {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified < cutoff {
                        if std::fs::remove_file(entry.path()).is_ok() {
                            cleaned += 1;
                        }
                    }
                }
            }
        }
        if cleaned > 0 {
            vtt_log!("Cleaned up {} old WAV files from /tmp", cleaned);
        }
    }
}

fn ctrlc_handler<F: Fn() + Send + 'static>(f: F) {
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_DFL);
        libc::signal(libc::SIGTERM, libc::SIG_DFL);
    }
    // Use a simple thread that blocks on signal
    thread::Builder::new()
        .name("signal-handler".into())
        .spawn(move || unsafe {
            let mut set: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut set);
            libc::sigaddset(&mut set, libc::SIGINT);
            libc::sigaddset(&mut set, libc::SIGTERM);
            libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
            let mut sig: i32 = 0;
            libc::sigwait(&set, &mut sig);
            f();
        })
        .ok();
}

