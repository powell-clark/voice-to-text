mod audio;
mod hotkey;
mod logging;
mod models;
mod settings;
mod transcribe;
mod tray;
mod typing;
mod whisper;

use audio::RecordingResult;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Work units sent to the transcription worker.
/// v2.0 uses in-memory f32 samples; Truncated marks buffer-full recordings so
/// the user sees `[Truncated]` prefix on the typed output.
enum WorkItem {
    Audio {
        samples: Vec<f32>,
        archive_path: PathBuf,
    },
    Truncated {
        samples: Vec<f32>,
        archive_path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    // Simple arg handling — no clap dep just for --version / --help.
    // Hold a hotkey to speak; the tray is the real UI. These flags only
    // exist so users can confirm what's installed without launching GTK.
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("voice-to-text {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "voice-to-text {} — push-to-talk offline transcription\n\n\
             Usage: vtt-linux [options]\n\n\
             Options:\n  \
             -V, --version    Print version and exit\n  \
             -h, --help       Print this help and exit\n\n\
             With no options, launches the tray icon. Hold the configured\n\
             hotkey (default: Scroll Lock) and speak. Release to transcribe.\n\n\
             Config:   ~/.local/share/voice-to-text/settings.conf\n\
             Logs:     ~/.local/share/voice-to-text/vtt-YYYY-MM-DD.log\n\
             Models:   /usr/share/voice-to-text/models/ggml-*.bin",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }

    // Platform-specific init
    #[cfg(target_os = "linux")]
    {
        unsafe {
            x11::xlib::XInitThreads();
        }
        gtk::init()?;
    }

    // Config directory
    let config_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("voice-to-text");

    // Singleton lock BEFORE logging::init so failed start attempts (another
    // instance already running) don't pollute the daily log with banner
    // lines. Error surfaces on stderr via anyhow's Display impl, which is
    // what the user sees when launching from a terminal anyway.
    // (Unix only — Windows needs CreateMutexW, see TASK-VTT044)
    #[cfg(unix)]
    let _lock_fd = singleton_lock(&config_dir)?;

    // Initialize logging (now we know we're the only instance)
    logging::init(&config_dir);
    vtt_log!("===========================================");
    vtt_log!(
        "Voice to Text {} — Starting (Rust)",
        env!("CARGO_PKG_VERSION")
    );
    vtt_log!("===========================================");

    // Silence whisper.cpp and ggml internal C-level stdout/stderr chatter.
    // Without `log_backend` / `tracing_backend` features, these hooks discard
    // the noisy model-init and per-transcription log lines whisper.cpp emits.
    whisper_rs::install_logging_hooks();

    // Signal handler (Unix only — Windows needs SetConsoleCtrlHandler, see TASK-VTT045)
    let running = Arc::new(AtomicBool::new(true));
    #[cfg(unix)]
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

    // Buffer full notification (shells to notify-send on Linux; libnotify-bin
    // is a runtime dependency. This avoids the notify-rust crate which pulls
    // zbus 5.x which uses edition 2024 — Ubuntu Noble cargo 1.75 can't parse).
    audio.set_buffer_full_callback(|| {
        let show_notification = || {
            #[cfg(target_os = "linux")]
            {
                let result = std::process::Command::new("notify-send")
                    .args([
                        "--icon=dialog-information",
                        "--expire-time=3000",
                        "Voice to Text",
                        "Recording limit reached — release key to transcribe",
                    ])
                    .status();
                if let Err(e) = result {
                    crate::vtt_log!("notify-send failed: {}", e);
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                // TODO: macOS/Windows notifications via osascript / winrt API in
                // STORY-VTT012 / STORY-VTT013.
                crate::vtt_log!("Recording limit reached");
            }
        };
        #[cfg(target_os = "linux")]
        glib::idle_add_once(show_notification);
        #[cfg(not(target_os = "linux"))]
        show_notification();
    });

    // Transcription work channel
    let (work_tx, work_rx) = mpsc::channel::<WorkItem>();

    // Create tray (returns UI message sender for cross-thread updates).
    // The returned Tray wraps the AppIndicator Rc but we immediately discard
    // it — the indicator survives because Tray::new also installed a
    // glib::timeout_add_local closure that holds its own clone of the Rc.
    // Keeping this discard explicit so future readers don't `?` or `_` the
    // struct away and wonder why the tray disappears.
    let (_tray, ui_tx) = tray::Tray::new(settings.clone(), &config_dir)?;

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
                hk_ui_tx
                    .send(tray::UiMessage::SetStatus("Recording...".into()))
                    .ok();
                hk_ui_tx
                    .send(tray::UiMessage::SetIcon("recording".into()))
                    .ok();
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
                    Some(RecordingResult::Audio { samples, path }) => {
                        vtt_log!("Recording saved: {}", path.display());
                        hk_ui_tx
                            .send(tray::UiMessage::SetStatus("Transcribing...".into()))
                            .ok();
                        hk_ui_tx
                            .send(tray::UiMessage::SetIcon("processing".into()))
                            .ok();
                        if let Err(e) = hk_work_tx.send(WorkItem::Audio {
                            samples,
                            archive_path: path,
                        }) {
                            vtt_log!(
                                "Transcription worker channel closed — worker thread died: {}",
                                e
                            );
                            hk_ui_tx
                                .send(tray::UiMessage::SetStatus(
                                    "Worker died — restart required".into(),
                                ))
                                .ok();
                            hk_ui_tx.send(tray::UiMessage::SetIcon("error".into())).ok();
                        }
                    }
                    Some(RecordingResult::MaxLength { samples, path }) => {
                        vtt_log!("Max recording length reached");
                        hk_ui_tx
                            .send(tray::UiMessage::SetStatus("Transcribing...".into()))
                            .ok();
                        hk_ui_tx
                            .send(tray::UiMessage::SetIcon("processing".into()))
                            .ok();
                        if let Err(e) = hk_work_tx.send(WorkItem::Truncated {
                            samples,
                            archive_path: path,
                        }) {
                            vtt_log!(
                                "Transcription worker channel closed — worker thread died: {}",
                                e
                            );
                            hk_ui_tx
                                .send(tray::UiMessage::SetStatus(
                                    "Worker died — restart required".into(),
                                ))
                                .ok();
                            hk_ui_tx.send(tray::UiMessage::SetIcon("error".into())).ok();
                        }
                    }
                    Some(RecordingResult::TooShort) | Some(RecordingResult::TooQuiet) => {
                        hk_ui_tx
                            .send(tray::UiMessage::SetStatus("Ready".into()))
                            .ok();
                        hk_ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();
                    }
                    None => {
                        vtt_log!("Recording returned None");
                        hk_ui_tx
                            .send(tray::UiMessage::SetStatus("Ready".into()))
                            .ok();
                        hk_ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();
                    }
                }
            }
        }
    })?;

    // Apply loaded hotkey
    if hk_keycode >= 8 {
        vtt_log!(
            "Applied custom hotkey from settings: keycode {}",
            hk_keycode
        );
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

    // Load initial engine at startup. Model comes from settings; language switches
    // to .en variant for multilingual-capable models.
    let (init_model, init_lang) = {
        let s = settings.read().unwrap();
        (s.selected_model.clone(), s.selected_language.clone())
    };
    let mut engine = match load_engine(&init_model, &init_lang, &ui_tx) {
        Some(e) => {
            ui_tx.send(tray::UiMessage::SetStatus("Ready".into())).ok();
            ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();
            Some(e)
        }
        None => {
            ui_tx
                .send(tray::UiMessage::SetStatus(
                    "No model — select one in the menu".into(),
                ))
                .ok();
            None
        }
    };

    while running.load(Ordering::Relaxed) {
        let item = match rx.recv() {
            Ok(item) => item,
            Err(_) => break, // Channel closed
        };

        let (samples, archive_path, is_truncated) = match item {
            WorkItem::Audio {
                samples,
                archive_path,
            } => (samples, archive_path, false),
            WorkItem::Truncated {
                samples,
                archive_path,
            } => (samples, archive_path, true),
        };

        // Detect tray-driven settings changes: if the selected model or language
        // no longer matches the loaded engine, reload transparently.
        let (want_model_raw, want_lang) = {
            let s = settings.read().unwrap();
            (s.selected_model.clone(), s.selected_language.clone())
        };
        let want_model =
            models::resolve_variant(&migrate_legacy_model_name(&want_model_raw), &want_lang);
        let needs_reload = engine
            .as_ref()
            .map(|e| e.model_name() != want_model)
            .unwrap_or(true);
        if needs_reload {
            drop(engine.take()); // free old engine before loading replacement
            engine = load_engine(&want_model_raw, &want_lang, &ui_tx);
        }

        let engine_ref = match engine.as_ref() {
            Some(e) => e,
            None => {
                vtt_log!("Transcription skipped — no model loaded");
                ui_tx
                    .send(tray::UiMessage::SetStatus("No model loaded".into()))
                    .ok();
                continue;
            }
        };

        ui_tx
            .send(tray::UiMessage::SetStatus("Transcribing...".into()))
            .ok();
        ui_tx
            .send(tray::UiMessage::SetIcon("processing".into()))
            .ok();

        // Read settings snapshot
        let s = settings.read().unwrap();
        let language = s.selected_language.clone();
        let prompt = s.initial_prompt.clone();
        let prefix = s.voice_prefix.clone();
        let append_newline = s.append_newline;
        let newline_type = s.newline_type;
        drop(s);

        vtt_log!(
            "Transcribing: model={}, lang={}, samples={}{}",
            engine_ref.model_name(),
            language,
            samples.len(),
            if is_truncated { " (TRUNCATED)" } else { "" }
        );
        let t0 = Instant::now();
        let text = transcribe::transcribe_samples(engine_ref, &samples, &language, &prompt);
        let elapsed = t0.elapsed();

        if let Some(text) = text {
            vtt_log!("Transcribed in {:.2}s", elapsed.as_secs_f64());
            let trimmed = text.trim();

            if is_whisper_filler(trimmed) {
                vtt_log!("Skipping blank transcription");
            } else if trimmed.chars().any(|c| c.is_alphanumeric()) {
                vtt_log!("Transcription: {}", trimmed);

                let final_text = compose_final_text(is_truncated, &prefix, trimmed);

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
        } else {
            vtt_log!("Transcription failed after {:.2}s", elapsed.as_secs_f64());
        }

        if !archive_path.as_os_str().is_empty() {
            save_and_cleanup(&archive_path, &config_dir);
        }

        ui_tx.send(tray::UiMessage::SetStatus("Ready".into())).ok();
        ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();
    }

    vtt_log!("Transcription worker stopped");
}

/// Load the GGML model identified by `menu_name` + `language`. Downloads the
/// file on first use and emits progress to the tray. Returns None on failure so
/// the caller can surface an explanatory status.
fn load_engine(
    menu_name: &str,
    language: &str,
    ui_tx: &tray::UiSender,
) -> Option<whisper::WhisperEngine> {
    let migrated = migrate_legacy_model_name(menu_name);
    let resolved = models::resolve_variant(&migrated, language);
    let info = match models::find(&resolved).or_else(|| models::find(&migrated)) {
        Some(i) => i,
        None => {
            vtt_log!("Unknown model '{}'; falling back to small.en", menu_name);
            models::find("small.en")?
        }
    };

    ui_tx
        .send(tray::UiMessage::SetStatus(format!(
            "Loading {}...",
            info.name
        )))
        .ok();
    ui_tx
        .send(tray::UiMessage::SetIcon("processing".into()))
        .ok();

    let ui_tx_clone = ui_tx.clone();
    let name_for_progress = info.name.to_string();
    let path = match models::ensure(info, move |done, total| {
        let pct = (done * 100).checked_div(total).unwrap_or(0);
        ui_tx_clone
            .send(tray::UiMessage::SetStatus(format!(
                "Downloading {}... {}%",
                name_for_progress, pct
            )))
            .ok();
    }) {
        Ok(p) => p,
        Err(e) => {
            vtt_log!("Model ensure failed for {}: {}", info.name, e);
            ui_tx
                .send(tray::UiMessage::SetStatus("Model download failed".into()))
                .ok();
            return None;
        }
    };

    match whisper::WhisperEngine::new(&path, info.name) {
        Ok(engine) => Some(engine),
        Err(e) => {
            vtt_log!("Engine load failed: {}", e);
            ui_tx
                .send(tray::UiMessage::SetStatus("Model load failed".into()))
                .ok();
            None
        }
    }
}

/// Translate legacy settings.conf model names ("CT2 large-v3-turbo", "W medium")
/// into the v2.0.0 flat namespace. Unknown values fall through unchanged so
/// `load_engine` can apply its default-to-small.en fallback.
fn migrate_legacy_model_name(raw: &str) -> String {
    let stripped = raw
        .strip_prefix("CT2 ")
        .or_else(|| raw.strip_prefix("W "))
        .unwrap_or(raw)
        .trim()
        .to_string();
    match stripped.as_str() {
        "" | "tiny" | "tiny.en" | "base" | "base.en" => "small.en".to_string(),
        "distil-large-v3" | "distil-large-v3.5" => "large-v3-turbo".to_string(),
        "large" => "large-v3".to_string(),
        _ => stripped,
    }
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
        .filter(|e| e.file_name().to_string_lossy().ends_with(".wav"))
        .filter_map(|e| {
            let mtime = e.metadata().ok()?.modified().ok()?;
            Some((e.path(), mtime))
        })
        .collect();

    if entries.len() <= max {
        return;
    }

    entries.sort_by_key(|e| std::cmp::Reverse(e.1)); // newest first
    for (path, _) in &entries[max..] {
        if std::fs::remove_file(path).is_ok() {
            vtt_log!("Pruned old recording: {}", path.display());
        }
    }
}

// ─── Utilities ──────────────────────────────────────────────────

#[cfg(unix)]
fn singleton_lock(config_dir: &std::path::Path) -> anyhow::Result<std::fs::File> {
    use std::io::Write;
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
        // Read the existing PID from the lock file (written by the holder below)
        // so the user knows which process to kill or check.
        let existing_pid = std::fs::read_to_string(&lock_path)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());
        match existing_pid {
            Some(pid) => anyhow::bail!(
                "Another instance of vtt-linux is already running (PID {}). \
                 Stop it with `kill {}` or `systemctl --user stop vtt.service`.",
                pid,
                pid
            ),
            None => anyhow::bail!(
                "Another instance of vtt-linux is already running. \
                 Find it with `pgrep -x vtt-linux` and stop it."
            ),
        }
    }

    // We hold the lock — write our PID into the lock file so a future failed
    // lock attempt can report which process to kill. Truncate first so the
    // file doesn't retain the previous holder's longer PID if this one is
    // shorter (e.g. 12345 replacing 456789).
    let mut file_mut = &file;
    file_mut.set_len(0).ok();
    writeln!(&mut file_mut, "{}", std::process::id()).ok();

    Ok(file)
}

fn cleanup_old_wavs() {
    let cutoff = std::time::SystemTime::now() - Duration::from_secs(3600);
    let cleaned = cleanup_old_wavs_in(std::path::Path::new("/tmp"), cutoff);
    if cleaned > 0 {
        vtt_log!("Cleaned up {} old WAV files from /tmp", cleaned);
    }
}

/// Pure-ish: delete `vtt_recording_*.wav` files in `dir` older than `cutoff`.
/// Returns the count actually deleted. Extracted so the filtering logic is
/// testable with a tempdir — the production entry point `cleanup_old_wavs()`
/// wraps this with the hardcoded `/tmp` and a 1-hour cutoff.
fn cleanup_old_wavs_in(dir: &std::path::Path, cutoff: std::time::SystemTime) -> usize {
    let mut cleaned = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("vtt_recording_") || !name.ends_with(".wav") {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified < cutoff && std::fs::remove_file(entry.path()).is_ok() {
                        cleaned += 1;
                    }
                }
            }
        }
    }
    cleaned
}

#[cfg(unix)]
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

/// Pure: build the final text that gets typed, given the transcription.
///
/// - If `is_truncated`, prepend `[Truncated] ` before the prefix.
/// - Else if the transcription doesn't already start with the prefix,
///   prepend the prefix (so the user isn't double-prefixed when Whisper
///   echoes the prefix back from the prompt).
/// - Else return the trimmed text unchanged.
fn compose_final_text(is_truncated: bool, prefix: &str, trimmed: &str) -> String {
    if is_truncated {
        format!("[Truncated] {}{}", prefix, trimmed)
    } else if !trimmed.starts_with(prefix) {
        format!("{}{}", prefix, trimmed)
    } else {
        trimmed.to_string()
    }
}

/// Is the given transcription a whisper "filler" (empty, bracketed marker,
/// or punctuation-only) that we should drop instead of typing?
///
/// Whisper models emit bracketed markers when they hear non-speech audio —
/// silence, music, applause, coughing, etc. These get typed literally if
/// we don't filter them, which is always surprising and never useful.
///
/// Pure function — no I/O, no globals, easy to extend and test.
fn is_whisper_filler(trimmed: &str) -> bool {
    if trimmed.is_empty() {
        return true;
    }

    // Strip a leading/trailing bracket pair and compare the inner token
    // case-insensitively against the known filler list. Covers [BLANK_AUDIO],
    // [MUSIC PLAYING], [Music], [Applause], [Silence], (silence), etc.
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .or_else(|| trimmed.strip_prefix('(').and_then(|s| s.strip_suffix(')')));

    if let Some(tok) = inner {
        let upper = tok.trim().to_ascii_uppercase();
        // Normalise whitespace and underscores so "MUSIC PLAYING" == "MUSIC_PLAYING".
        let norm = upper.replace('_', " ");
        let norm = norm.split_whitespace().collect::<Vec<_>>().join(" ");
        return matches!(
            norm.as_str(),
            "BLANK AUDIO"
                | "MUSIC PLAYING"
                | "MUSIC"
                | "APPLAUSE"
                | "SILENCE"
                | "NO AUDIO"
                | "LAUGHTER"
                | "COUGHING"
                | "INAUDIBLE"
                | "NOISE"
        );
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_final_text_prepends_prefix_when_missing() {
        let s = compose_final_text(false, "[Voice] ", "hello world");
        assert_eq!(s, "[Voice] hello world");
    }

    #[test]
    fn compose_final_text_does_not_double_prefix() {
        let s = compose_final_text(false, "[Voice] ", "[Voice] hello world");
        assert_eq!(s, "[Voice] hello world");
    }

    #[test]
    fn compose_final_text_handles_truncated_flag() {
        let s = compose_final_text(true, "[Voice] ", "hello world");
        assert_eq!(s, "[Truncated] [Voice] hello world");
    }

    #[test]
    fn compose_final_text_truncated_wins_over_prefix_check() {
        // Even when the text already starts with prefix, truncated still prepends.
        let s = compose_final_text(true, "[Voice] ", "[Voice] hello");
        assert_eq!(s, "[Truncated] [Voice] [Voice] hello");
    }

    #[test]
    fn compose_final_text_empty_prefix_returns_trimmed_unchanged() {
        let s = compose_final_text(false, "", "hello");
        assert_eq!(
            s, "hello",
            "empty prefix means every string already 'starts with' it"
        );
    }

    #[test]
    fn compose_final_text_unicode_prefix_and_body_roundtrip() {
        let s = compose_final_text(false, "[£ é] ", "£100 an hour");
        assert_eq!(s, "[£ é] £100 an hour");
    }

    #[test]
    fn is_whisper_filler_drops_empty_and_known_bracketed_markers() {
        for filler in [
            "",
            "[BLANK_AUDIO]",
            "[blank_audio]",
            "[MUSIC PLAYING]",
            "[music playing]",
            "[Music]",
            "[Applause]",
            "[Silence]",
            "[NO AUDIO]",
            "[no_audio]",
            "(silence)",
            "[LAUGHTER]",
            "[Coughing]",
            "[INAUDIBLE]",
            "[noise]",
        ] {
            assert!(
                is_whisper_filler(filler),
                "expected {:?} to be treated as filler",
                filler
            );
        }
    }

    #[test]
    fn is_whisper_filler_does_not_drop_real_speech() {
        for text in [
            "Hello world",
            "I was about to say [music] playing in the background",
            "[something custom]",
            "[deprecated]",
            "£100 an hour",
            "Music is great",
        ] {
            assert!(
                !is_whisper_filler(text),
                "expected {:?} to pass through (real speech, not filler)",
                text
            );
        }
    }

    #[test]
    fn is_whisper_filler_handles_case_and_whitespace_variants() {
        assert!(is_whisper_filler("[  MUSIC  PLAYING  ]"));
        assert!(is_whisper_filler("[MUSIC_PLAYING]"));
        assert!(is_whisper_filler("[music    playing]"));
        assert!(is_whisper_filler("[ Blank_Audio ]"));
    }

    #[test]
    fn migrate_legacy_model_name_strips_ct2_and_w_prefixes() {
        assert_eq!(migrate_legacy_model_name("CT2 small"), "small");
        assert_eq!(migrate_legacy_model_name("W small.en"), "small.en");
        assert_eq!(migrate_legacy_model_name("CT2 medium"), "medium");
    }

    #[test]
    fn migrate_legacy_model_name_maps_retired_models_to_supported_ones() {
        // tiny and base were removed in the v2.0 model trim — migrate to small.en
        // so users with stale settings.conf don't error out on startup.
        assert_eq!(migrate_legacy_model_name("tiny"), "small.en");
        assert_eq!(migrate_legacy_model_name("tiny.en"), "small.en");
        assert_eq!(migrate_legacy_model_name("base"), "small.en");
        assert_eq!(migrate_legacy_model_name("base.en"), "small.en");
        assert_eq!(migrate_legacy_model_name(""), "small.en");
    }

    #[test]
    fn migrate_legacy_model_name_maps_distil_to_large_v3_turbo() {
        // distil models were replaced by large-v3-turbo — same speed, better quality.
        assert_eq!(
            migrate_legacy_model_name("distil-large-v3"),
            "large-v3-turbo"
        );
        assert_eq!(
            migrate_legacy_model_name("distil-large-v3.5"),
            "large-v3-turbo"
        );
    }

    #[test]
    fn migrate_legacy_model_name_maps_bare_large_to_large_v3() {
        assert_eq!(migrate_legacy_model_name("large"), "large-v3");
    }

    #[test]
    fn migrate_legacy_model_name_passes_current_names_through_unchanged() {
        assert_eq!(migrate_legacy_model_name("small"), "small");
        assert_eq!(migrate_legacy_model_name("small.en"), "small.en");
        assert_eq!(
            migrate_legacy_model_name("large-v3-turbo"),
            "large-v3-turbo"
        );
        assert_eq!(migrate_legacy_model_name("large-v3"), "large-v3");
    }

    #[test]
    fn migrate_legacy_model_name_handles_whitespace() {
        assert_eq!(migrate_legacy_model_name("  small  "), "small");
        assert_eq!(migrate_legacy_model_name("CT2  small"), "small");
    }

    #[test]
    fn prune_recordings_deletes_oldest_when_over_cap() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Create 5 .wav files with ascending mtimes (old..new). Sleep between
        // writes so the filesystem's mtime resolution (typically 1ms on ext4,
        // but up to 1s on older filesystems) can distinguish them.
        for i in 0..5 {
            let path = root.join(format!("vtt_recording_{:03}.wav", i));
            std::fs::write(&path, b"fake wav bytes").unwrap();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        prune_recordings(root, 2);

        // After pruning, only the 2 newest should remain.
        let remaining: Vec<String> = std::fs::read_dir(root)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(remaining.len(), 2, "got {:?}", remaining);
        // Files 003 and 004 are newest (written last, so most recent mtimes).
        assert!(remaining.iter().any(|n| n.contains("003")));
        assert!(remaining.iter().any(|n| n.contains("004")));
    }

    #[test]
    fn prune_recordings_noop_when_at_or_under_cap() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for i in 0..3 {
            std::fs::write(root.join(format!("vtt_recording_{:03}.wav", i)), b"fake").unwrap();
        }
        prune_recordings(root, 5);
        assert_eq!(
            std::fs::read_dir(root).unwrap().count(),
            3,
            "pruning under cap should touch nothing"
        );
    }

    #[test]
    fn prune_recordings_ignores_non_wav_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(root.join("vtt_recording_1.wav"), b"fake").unwrap();
        std::fs::write(root.join("vtt_recording_2.wav"), b"fake").unwrap();
        std::fs::write(root.join("vtt_recording_3.wav"), b"fake").unwrap();
        std::fs::write(root.join("notes.txt"), b"user notes").unwrap();
        std::fs::write(root.join("README"), b"keep").unwrap();

        prune_recordings(root, 1);

        // Two .wav files deleted, non-wav files untouched.
        let remaining: Vec<String> = std::fs::read_dir(root)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(remaining.contains(&"notes.txt".to_string()));
        assert!(remaining.contains(&"README".to_string()));
        let wav_count = remaining.iter().filter(|n| n.ends_with(".wav")).count();
        assert_eq!(
            wav_count, 1,
            "only 1 .wav should remain, got: {:?}",
            remaining
        );
    }

    #[test]
    fn prune_recordings_missing_directory_is_safe_noop() {
        // Should not panic or error if the directory doesn't exist yet.
        let dir = tempfile::tempdir().unwrap();
        let nonexistent = dir.path().join("not-a-dir");
        prune_recordings(&nonexistent, 10);
        // No assertion needed — we just want to confirm it doesn't panic.
    }

    #[test]
    fn cleanup_old_wavs_in_deletes_only_old_matching_files() {
        use std::time::{Duration, SystemTime};
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Mix of files:
        //  - vtt_recording_fresh.wav (current — should survive)
        //  - vtt_recording_old.wav (cutoff rewound 2h — should be deleted)
        //  - other-app.wav (not ours — must survive regardless of age)
        //  - vtt_recording_note.txt (wrong extension — survives)
        std::fs::write(root.join("vtt_recording_fresh.wav"), b"fresh").unwrap();
        std::fs::write(root.join("vtt_recording_old.wav"), b"old").unwrap();
        std::fs::write(root.join("other-app.wav"), b"other").unwrap();
        std::fs::write(root.join("vtt_recording_note.txt"), b"note").unwrap();

        // Cutoff = "now + 1 second" makes every existing file appear "old"
        // relative to the cutoff, except we want to preserve "fresh" — so
        // we instead use a cutoff in the PAST (1 second ago). Only files
        // with mtime < cutoff are deleted. Since all files were just created
        // with mtime == now, NONE are older than 1-second-ago.
        let past_cutoff = SystemTime::now() - Duration::from_secs(1);
        let deleted = cleanup_old_wavs_in(root, past_cutoff);
        assert_eq!(
            deleted, 0,
            "with past cutoff, nothing should be deleted (all files are fresh)"
        );

        // Now use a cutoff in the future — every file is "older than the future".
        // But only vtt_recording_*.wav should be deleted.
        let future_cutoff = SystemTime::now() + Duration::from_secs(60);
        let deleted = cleanup_old_wavs_in(root, future_cutoff);
        assert_eq!(
            deleted, 2,
            "both vtt_recording_*.wav files should be deleted, non-matching preserved"
        );

        let remaining: Vec<String> = std::fs::read_dir(root)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(remaining.contains(&"other-app.wav".to_string()));
        assert!(remaining.contains(&"vtt_recording_note.txt".to_string()));
        assert!(!remaining.contains(&"vtt_recording_fresh.wav".to_string()));
        assert!(!remaining.contains(&"vtt_recording_old.wav".to_string()));
    }

    #[test]
    fn cleanup_old_wavs_in_missing_dir_is_safe_noop() {
        let dir = tempfile::tempdir().unwrap();
        let nonexistent = dir.path().join("nonexistent");
        let deleted = cleanup_old_wavs_in(&nonexistent, std::time::SystemTime::now());
        assert_eq!(deleted, 0);
    }

    #[test]
    fn cleanup_old_wavs_in_empty_dir_returns_zero() {
        let dir = tempfile::tempdir().unwrap();
        let deleted = cleanup_old_wavs_in(dir.path(), std::time::SystemTime::now());
        assert_eq!(deleted, 0);
    }
}
