// On Windows, build as a GUI (windowed) binary so launching the tray app does
// not pop a console window — the system tray icon is the UI. Logs still go to
// the daily log file; stdout is simply unused on a normal launch. CLI flags
// (--version/--help) re-attach to the launching terminal (see main). On
// Linux/macOS this attribute is a no-op.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod archive;
mod audio;
mod corrections;
mod ct2_client;
mod denoise;
mod doctor;
// autostart is only consumed by the portable tray (Windows + macOS); compiling
// it on Linux makes it dead code, which fails the -D warnings release build.
#[cfg(any(target_os = "windows", target_os = "macos"))]
mod autostart;
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
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Work units sent to the transcription worker.
/// v2.0 uses in-memory f32 samples; Truncated marks buffer-full recordings so
/// the user sees `[Truncated]` prefix on the typed output.
pub(crate) enum WorkItem {
    Audio {
        samples: Vec<f32>,
        archive_path: PathBuf,
        /// The same recording at the device's own rate, carried only when the
        /// `archive` setting is on. The worker needs it here rather than in the
        /// audio layer because the sidecar records the transcript, and the
        /// transcript does not exist until after this item is worked.
        native: Option<audio::NativeCapture>,
    },
    Truncated {
        samples: Vec<f32>,
        archive_path: PathBuf,
        native: Option<audio::NativeCapture>,
    },
    /// Re-run transcription on the newest archived recording and re-type the
    /// result — the tray recovery net for lost output (FEAT-VTT039). Carries no
    /// audio: the worker locates and decodes the WAV itself, so the tray stays
    /// decoupled from decode/whisper and no file I/O lands on the UI thread.
    RetranscribeLast,
}

/// Re-attach the (windowed) process to its launching console so `--version` /
/// `--help` output is visible when run from a terminal. Best-effort: does
/// nothing if there is no parent console (e.g. launched from Explorer). Links
/// `AttachConsole` from kernel32 directly to avoid a winapi dependency just for
/// this one call.
#[cfg(target_os = "windows")]
fn attach_parent_console() {
    // ATTACH_PARENT_PROCESS = (DWORD)-1
    extern "system" {
        fn AttachConsole(dw_process_id: u32) -> i32;
    }
    unsafe {
        AttachConsole(0xFFFF_FFFF);
    }
}

fn main() -> anyhow::Result<()> {
    // Simple arg handling — no clap dep just for --version / --help.
    // Hold a hotkey to speak; the tray is the real UI. These flags only
    // exist so users can confirm what's installed without launching GTK.
    let args: Vec<String> = std::env::args().collect();
    let wants_version = args.iter().any(|a| a == "--version" || a == "-V");
    let wants_help = args.iter().any(|a| a == "--help" || a == "-h");
    // `--doctor` answers "is the app I am running the app I just installed?"
    // Three deployment failures in one morning all looked like success
    // (TASK-VTT153); this makes the question answerable on purpose.
    let wants_doctor = args.iter().any(|a| a == "--doctor");
    // Batch mode: `--file <PATH>` transcribes a 16 kHz WAV and prints the
    // transcript to stdout, then exits — no tray, no hotkey (TASK-VTT023).
    let wants_file = args.iter().any(|a| a == "--file" || a == "-f");
    let file_path = args
        .iter()
        .position(|a| a == "--file" || a == "-f")
        .and_then(|i| args.get(i + 1).cloned());
    // On Windows the binary is windowed (no console of its own), so re-attach to
    // the launching terminal before printing CLI output — otherwise --version /
    // --help / --file would silently produce nothing when run from a shell.
    #[cfg(target_os = "windows")]
    if wants_version || wants_help || wants_file || wants_doctor {
        attach_parent_console();
    }
    if wants_version {
        println!("voice-to-text {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if wants_help {
        println!(
            "voice-to-text {} — push-to-talk offline transcription\n\n\
             Usage: vtt-linux [options]\n\n\
             Options:\n  \
             -V, --version      Print version and exit\n  \
             -h, --help         Print this help and exit\n  \
             -f, --file <PATH>  Transcribe a 16 kHz WAV to stdout and exit\n  \
                 --doctor       Check the running app matches the installed one\n\n\
             With no options, launches the tray icon. Hold the configured\n\
             hotkey (default: Scroll Lock) and speak. Release to transcribe.\n\n\
             Config:   ~/.local/share/voice-to-text/settings.conf\n\
             Logs:     ~/.local/share/voice-to-text/vtt-YYYY-MM-DD.log\n\
             Models:   /usr/share/voice-to-text/models/ggml-*.bin",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    if wants_doctor {
        return run_doctor();
    }
    if wants_file {
        return run_file_mode(file_path.as_deref());
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
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("voice-to-text");

    // Singleton lock BEFORE logging::init so failed start attempts (another
    // instance already running) don't pollute the daily log with banner
    // lines. Error surfaces on stderr via anyhow's Display impl, which is
    // what the user sees when launching from a terminal anyway.
    #[cfg(unix)]
    let _lock_fd = singleton_lock(&data_dir)?;
    #[cfg(windows)]
    singleton_lock()?;

    // Initialize logging (now we know we're the only instance)
    logging::init(&data_dir);
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
    #[cfg(windows)]
    {
        let r = running.clone();
        ctrlc_handler(move || {
            eprintln!("Signal received, shutting down");
            r.store(false, Ordering::SeqCst);
        });
    }

    // Clean up old WAV files from /tmp
    cleanup_old_wavs();

    // Load settings
    let settings = Arc::new(RwLock::new(settings::Settings::load(&data_dir)));

    // First-run default: enable start-at-login once on Windows so VTT is there
    // after a reboot without the user having to find the tray toggle first
    // (TASK-VTT109). The tray "Start at login" item stays the off switch; the
    // persisted `autostart_initialized` marker means we apply this default
    // exactly once and never re-enable after the user opts out.
    #[cfg(target_os = "windows")]
    {
        let needs_init = !settings.read().unwrap().autostart_initialized;
        if needs_init {
            match autostart::enable() {
                Ok(()) => {
                    let mut s = settings.write().unwrap();
                    s.autostart_initialized = true;
                    if let Err(e) = s.save() {
                        vtt_log!("Failed to persist autostart marker: {}", e);
                    }
                    vtt_log!("First-run default: start-at-login enabled");
                }
                Err(e) => {
                    vtt_log!(
                        "First-run autostart enable failed (will retry next launch): {}",
                        e
                    )
                }
            }
        }
    }

    // Shared state
    // Push-to-talk state plus the instant the current hold began; the
    // watchdog thread reads both, so they outlive the hotkey closure.
    let gate = Arc::new(Mutex::new(hotkey::PushToTalk::new()));
    let recording_start = Arc::new(Mutex::new(Instant::now()));
    let typing_active = Arc::new(AtomicBool::new(false));
    let typing_has_output = Arc::new(AtomicBool::new(false));
    let last_transcription: tray::LastTranscription = Arc::new(Mutex::new(None));

    // Initialize audio against the user's saved input-device choice (settings
    // `device=N`); a negative value means "follow the system default".
    let selected_device_index =
        usize::try_from(settings.read().unwrap().selected_device_index).ok();
    let audio = Arc::new(audio::Audio::new(selected_device_index)?);
    // Retention of the native-rate capture is gated here, once, from the
    // setting — off by default, so nothing downstream ever sees a second copy
    // of the audio unless the user has asked for the archive (TASK-VTT150).
    {
        let s = settings.read().unwrap();
        audio.set_archive_enabled(s.archive_recordings);
        audio.set_denoise_enabled(s.denoise);
        // Say what these resolved to, at startup, every run. An archive-enabled
        // build that writes nothing is otherwise indistinguishable from a
        // working one: on 2026-09-03 a stale process kept answering the hotkey
        // for eighty minutes while transcription worked perfectly and no
        // archive appeared, and nothing in the log said why (TASK-VTT156).
        if s.archive_recordings {
            let resolved = archive::resolve_archive_dir_checked(&s.archive_dir, &data_dir);
            if let archive::ArchiveDir::Rejected { why, .. } = &resolved {
                // Loud, because the alternative is writing somewhere the
                // operator did not ask for and will not find (TASK-VTT161).
                vtt_log!("Archiving: archive_dir IGNORED — {}", why);
            }
            vtt_log!(
                "Archiving ON -> {} (cap {})",
                resolved.path().display(),
                if s.archive_max_files == 0 {
                    "unbounded".to_string()
                } else {
                    s.archive_max_files.to_string()
                }
            );
        } else {
            vtt_log!("Archiving off (set archive=1 in settings.conf to enable)");
        }
        vtt_log!("Rumble filtering {}", if s.denoise { "on" } else { "off" });
    }

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
    // On Linux, GTK owns the event loop so the Tray struct can be discarded
    // after construction. On macOS/Windows we must keep it alive and call
    // poll_menu() each tick so menu events are processed on the main thread
    // (muda uses Rc internally — menu items are !Send).
    #[cfg(target_os = "linux")]
    let (_tray, ui_tx) = tray::Tray::new(
        settings.clone(),
        &data_dir,
        last_transcription.clone(),
        work_tx.clone(),
    )?;
    #[cfg(not(target_os = "linux"))]
    let (mut tray, ui_tx) = tray::Tray::new(
        settings.clone(),
        &data_dir,
        last_transcription.clone(),
        work_tx.clone(),
    )?;

    // Worker thread — transcribes audio and types result
    let worker_settings = settings.clone();
    let worker_typing_active = typing_active.clone();
    let worker_typing_has_output = typing_has_output.clone();
    let worker_running = running.clone();
    let worker_ui_tx = ui_tx.clone();
    let worker_data_dir = data_dir.clone();
    let worker_last_transcription = last_transcription.clone();

    thread::Builder::new()
        .name("transcription-worker".into())
        .spawn(move || {
            transcription_worker(
                work_rx,
                WorkerCtx {
                    settings: worker_settings,
                    typing_active: worker_typing_active,
                    typing_has_output: worker_typing_has_output,
                    running: worker_running,
                    ui_tx: worker_ui_tx,
                    data_dir: worker_data_dir,
                    last_transcription: worker_last_transcription,
                },
            );
        })?;

    // Hotkey monitor
    let hk_keycode = settings.read().unwrap().hotkey_keycode;
    let hk_gate = gate.clone();
    let hk_audio = audio.clone();
    let hk_typing_active = typing_active.clone();
    let hk_work_tx = work_tx.clone();
    let hk_ui_tx = ui_tx.clone();
    let hk_recording_start = recording_start.clone();

    let hotkey_tx = hotkey::start_monitor(hk_keycode, move |event| {
        match event {
            hotkey::KeyEvent::Down => {
                if hk_gate.lock().unwrap().press() == hotkey::Action::Ignore {
                    return;
                }

                vtt_log!("Key pressed - starting recording");
                *hk_recording_start.lock().unwrap() = Instant::now();
                hk_ui_tx
                    .send(tray::UiMessage::SetStatus("Recording...".into()))
                    .ok();
                hk_ui_tx
                    .send(tray::UiMessage::SetIcon("recording".into()))
                    .ok();

                // Let the previous transcription finish typing before the mic
                // opens, but do the waiting on a worker thread. This callback
                // runs on the single X11 event thread that also delivers
                // KeyRelease, so blocking here queued the user's real release
                // and it arrived ~0ms later, where the old guard discarded it
                // as auto-repeat and left the mic open (TASK-VTT146).
                let starter_audio = hk_audio.clone();
                let starter_typing = hk_typing_active.clone();
                let starter_gate = hk_gate.clone();
                thread::spawn(move || {
                    let start = Instant::now();
                    while starter_typing.load(Ordering::Relaxed)
                        && start.elapsed() < Duration::from_secs(30)
                    {
                        thread::sleep(Duration::from_millis(1));
                    }
                    // Hold the gate across the check so a release landing right
                    // now waits for the stream to open rather than racing past
                    // it and leaving nothing to stop.
                    let gate = starter_gate.lock().unwrap();
                    if gate.is_recording() {
                        starter_audio.start_recording();
                    }
                });
            }
            hotkey::KeyEvent::Up => {
                let held_for = hk_recording_start.lock().unwrap().elapsed();
                if hk_gate.lock().unwrap().release(held_for) == hotkey::Action::Ignore {
                    return;
                }

                vtt_log!(
                    "Key released - stopping recording ({}ms)",
                    held_for.as_millis()
                );
                finish_recording(&hk_audio, &hk_work_tx, &hk_ui_tx);
            }
        }
    })?;

    // Watchdog. A KeyRelease lost anywhere below this layer must not be able to
    // strand the microphone, so an over-long hold is force-stopped rather than
    // trusted (TASK-VTT146).
    let wd_gate = gate.clone();
    let wd_audio = audio.clone();
    let wd_work_tx = work_tx.clone();
    let wd_ui_tx = ui_tx.clone();
    let wd_recording_start = recording_start.clone();
    thread::Builder::new()
        .name("recording-watchdog".into())
        .spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
            let held_for = wd_recording_start.lock().unwrap().elapsed();
            if wd_gate.lock().unwrap().poll(held_for) == hotkey::Action::Stop {
                vtt_log!(
                    "Watchdog: hold exceeded {}s with no KeyRelease - forcing stop",
                    hotkey::MAX_HOLD.as_secs()
                );
                finish_recording(&wd_audio, &wd_work_tx, &wd_ui_tx);
            }
        })?;

    // Apply loaded hotkey
    if hk_keycode >= 8 {
        vtt_log!(
            "Applied custom hotkey from settings: keycode {}",
            hk_keycode
        );
        // The dialog now refuses to bind a typing key, but a settings.conf
        // written before that check exists still holds one, and the symptom —
        // a character that has stopped working everywhere — reads as a broken
        // keyboard rather than a hotkey choice (TASK-VTT147).
        #[cfg(target_os = "linux")]
        if hotkey::keycode_is_typing(hk_keycode) {
            vtt_log!(
                "WARNING: hotkey keycode {} is '{}', a key you type with. \
                 Holding it globally means that character no longer reaches \
                 other applications. Change it from the tray's Hotkey item.",
                hk_keycode,
                hotkey::get_key_name(hk_keycode)
            );
        }
    }

    vtt_log!("All systems initialized");
    ui_tx.send(tray::UiMessage::SetStatus("Ready".into())).ok();
    ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();

    // Run platform event loop (blocks until quit)
    #[cfg(target_os = "linux")]
    gtk::main();

    // Windows: pump the Win32 message queue so the tray icon's hidden window
    // processes clicks and pops its context menu (TASK-VTT091), then drain muda
    // menu events on this thread (menu items are !Send / Rc-based).
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
        };
        let mut msg: MSG = unsafe { std::mem::zeroed() };
        loop {
            // Drain every pending message this tick.
            while unsafe { PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            tray.poll_menu();
            thread::sleep(Duration::from_millis(16));
            if !running.load(Ordering::Relaxed) {
                break;
            }
        }
    }

    // macOS: poll menu events on the main thread (separate run-loop mechanism).
    #[cfg(target_os = "macos")]
    {
        loop {
            thread::sleep(Duration::from_millis(100));
            tray.poll_menu();
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

/// Owns the transcription loop for the lifetime of the process — the extra
/// param over clippy's default threshold is independent cross-thread handles,
/// not a design smell to fix by bundling into a struct only this fn would use.
#[allow(clippy::too_many_arguments)]
/// Close the microphone and route the captured audio to the right place.
///
/// Shared by the KeyRelease handler and the watchdog so that a force-stop takes
/// exactly the same path as a normal release — the watchdog is worthless if it
/// clears the flag but leaves the capture stream running (TASK-VTT146).
fn finish_recording(
    audio: &audio::Audio,
    work_tx: &mpsc::Sender<WorkItem>,
    ui_tx: &tray::UiSender,
) {
    match audio.stop_recording() {
        Some(RecordingResult::Audio {
            samples,
            path,
            native,
        }) => {
            vtt_log!("Recording saved: {}", path.display());
            ui_tx
                .send(tray::UiMessage::SetStatus("Transcribing...".into()))
                .ok();
            ui_tx
                .send(tray::UiMessage::SetIcon("processing".into()))
                .ok();
            if let Err(e) = work_tx.send(WorkItem::Audio {
                samples,
                archive_path: path,
                native,
            }) {
                vtt_log!(
                    "Transcription worker channel closed — worker thread died: {}",
                    e
                );
                ui_tx
                    .send(tray::UiMessage::SetStatus(
                        "Worker died — restart required".into(),
                    ))
                    .ok();
                ui_tx.send(tray::UiMessage::SetIcon("error".into())).ok();
            }
        }
        Some(RecordingResult::MaxLength {
            samples,
            path,
            native,
        }) => {
            vtt_log!("Max recording length reached");
            ui_tx
                .send(tray::UiMessage::SetStatus("Transcribing...".into()))
                .ok();
            ui_tx
                .send(tray::UiMessage::SetIcon("processing".into()))
                .ok();
            if let Err(e) = work_tx.send(WorkItem::Truncated {
                samples,
                archive_path: path,
                native,
            }) {
                vtt_log!(
                    "Transcription worker channel closed — worker thread died: {}",
                    e
                );
                ui_tx
                    .send(tray::UiMessage::SetStatus(
                        "Worker died — restart required".into(),
                    ))
                    .ok();
                ui_tx.send(tray::UiMessage::SetIcon("error".into())).ok();
            }
        }
        Some(RecordingResult::NoAudioCaptured) => {
            // Dead stream detected and re-opened inside audio.rs;
            // tell the user honestly instead of the old misleading
            // "Recording too short (0.00s)" (TASK-VTT121).
            vtt_log!("No audio captured — mic changed/suspended; stream re-opened");
            ui_tx
                .send(tray::UiMessage::SetStatus(
                    "No audio — check microphone".into(),
                ))
                .ok();
            ui_tx.send(tray::UiMessage::SetIcon("error".into())).ok();
            #[cfg(target_os = "linux")]
            {
                let result = std::process::Command::new("notify-send")
                    .args([
                        "--icon=dialog-warning",
                        "--expire-time=5000",
                        "Voice to Text",
                        "No audio captured — check your microphone and try again",
                    ])
                    .status();
                if let Err(e) = result {
                    vtt_log!("notify-send failed: {}", e);
                }
            }
        }
        Some(RecordingResult::TooShort) | Some(RecordingResult::TooQuiet) => {
            ui_tx.send(tray::UiMessage::SetStatus("Ready".into())).ok();
            ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();
        }
        None => {
            vtt_log!("Recording returned None");
            ui_tx.send(tray::UiMessage::SetStatus("Ready".into())).ok();
            ui_tx.send(tray::UiMessage::SetIcon("ready".into())).ok();
        }
    }
}

/// Everything the transcription worker borrows from the main thread. Grouped
/// into one value so the worker can gain a dependency without the signature
/// growing another positional parameter.
struct WorkerCtx {
    settings: Arc<RwLock<settings::Settings>>,
    typing_active: Arc<AtomicBool>,
    typing_has_output: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    ui_tx: tray::UiSender,
    data_dir: PathBuf,
    last_transcription: tray::LastTranscription,
}

fn transcription_worker(rx: mpsc::Receiver<WorkItem>, ctx: WorkerCtx) {
    let WorkerCtx {
        settings,
        typing_active,
        typing_has_output,
        running,
        ui_tx,
        data_dir,
        last_transcription,
    } = ctx;

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

    // Optional CT2 daemon backend (FEAT-VTT034, TASK-VTT054). Spawned once at
    // startup, never restarted mid-session — any failure here or later falls
    // back to the whisper-rs engine already loaded above, silently to the
    // transcription result (the tray label is the only visible signal).
    let mut ct2: Option<ct2_client::Ct2Client> = None;
    if settings.read().unwrap().backend == "ct2" {
        let want_model =
            models::resolve_variant(&migrate_legacy_model_name(&init_model), &init_lang);
        match ct2_client::resolve_daemon_script() {
            Some(script) => {
                ui_tx
                    .send(tray::UiMessage::SetBackendLabel(format!(
                        "CT2 (starting {want_model})"
                    )))
                    .ok();
                match ct2_client::Ct2Client::spawn(&script, &want_model, "cpu", "int8") {
                    Some(client) => {
                        vtt_log!("CT2 daemon started with model {want_model}");
                        ui_tx
                            .send(tray::UiMessage::SetBackendLabel("CT2".into()))
                            .ok();
                        ct2 = Some(client);
                    }
                    None => {
                        vtt_log!("CT2 daemon failed to start; using native whisper-rs backend");
                        ui_tx
                            .send(tray::UiMessage::SetBackendLabel(
                                "Native (CT2 failed to start)".into(),
                            ))
                            .ok();
                    }
                }
            }
            None => {
                vtt_log!(
                    "backend=ct2 but transcribe_daemon.py could not be located; using native whisper-rs backend"
                );
                ui_tx
                    .send(tray::UiMessage::SetBackendLabel(
                        "Native (CT2 daemon not found)".into(),
                    ))
                    .ok();
            }
        }
    } else {
        ui_tx
            .send(tray::UiMessage::SetBackendLabel("Native".into()))
            .ok();
    }

    while running.load(Ordering::Relaxed) {
        let item = match rx.recv() {
            Ok(item) => item,
            Err(_) => break, // Channel closed
        };

        let (samples, archive_path, is_truncated, native) = match item {
            WorkItem::Audio {
                samples,
                archive_path,
                native,
            } => (samples, archive_path, false, native),
            WorkItem::Truncated {
                samples,
                archive_path,
                native,
            } => (samples, archive_path, true, native),
            // Re-transcribe the newest archived recording and re-type it. The
            // empty archive_path makes the save/prune step below a no-op so the
            // source WAV is not re-archived or self-copied (FEAT-VTT039).
            WorkItem::RetranscribeLast => {
                let recordings_dir = data_dir.join("recordings");
                match newest_wav(&recordings_dir) {
                    Some(path) => match whisper::decode_wav_to_samples(&path) {
                        Ok(s) => {
                            vtt_log!("Re-transcribing last recording: {}", path.display());
                            // Never re-archives: the source is already an
                            // archived 16 kHz wav, and a second copy would be
                            // a duplicate corpus row with a re-run transcript.
                            (s, PathBuf::new(), false, None)
                        }
                        Err(e) => {
                            vtt_log!("Re-transcribe: decode failed for {}: {}", path.display(), e);
                            continue;
                        }
                    },
                    None => {
                        vtt_log!(
                            "Re-transcribe: no recording found in {}",
                            recordings_dir.display()
                        );
                        ui_tx
                            .send(tray::UiMessage::SetStatus(
                                "No recording to re-transcribe".into(),
                            ))
                            .ok();
                        continue;
                    }
                }
            }
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
        let corrections = s.corrections.clone();
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
        // Try the CT2 daemon first when it's alive; any failure (including
        // this specific call) falls through to the whisper-rs engine already
        // loaded, exactly as if CT2 had never been requested (TASK-VTT054).
        let ct2_text = ct2.as_mut().filter(|c| c.is_alive()).and_then(|client| {
            match audio::write_wav(&samples, whisper::WHISPER_INPUT_RATE) {
                Ok(wav_path) => {
                    let result = client.transcribe(&wav_path, &language, &prompt);
                    let _ = std::fs::remove_file(&wav_path);
                    if !client.is_alive() {
                        ui_tx
                            .send(tray::UiMessage::SetBackendLabel(
                                "Native (CT2 daemon died)".into(),
                            ))
                            .ok();
                    }
                    result
                }
                Err(e) => {
                    vtt_log!("CT2: failed to write temp wav for the daemon: {e}");
                    None
                }
            }
        });
        let text = match ct2_text {
            Some(text) => Some(text),
            None => transcribe::transcribe_samples(engine_ref, &samples, &language, &prompt),
        };
        let elapsed = t0.elapsed();

        let mut archived_text: Option<String> = None;
        if let Some(text) = text {
            vtt_log!("Transcribed in {:.2}s", elapsed.as_secs_f64());
            let trimmed = text.trim();

            if is_whisper_filler(trimmed) {
                vtt_log!("Skipping blank transcription");
            } else if trimmed.chars().any(|c| c.is_alphanumeric()) {
                vtt_log!("Transcription: {}", trimmed);

                let corrected = corrections::apply(trimmed, &corrections);
                let final_text = compose_final_text(is_truncated, &prefix, &corrected);
                *last_transcription.lock().unwrap() = Some(final_text.clone());
                archived_text = Some(corrected.clone());

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

        // Archive the native-rate recording with its transcript (TASK-VTT150).
        // Only reached when the `archive` setting put a native buffer on this
        // work item, and only when there is a transcript to pair with it — a
        // wav whose text is empty or filler is not a corpus row.
        if let (Some(native), Some(text)) = (
            native,
            archived_text.filter(|t| {
                let keep = archive::should_archive(t);
                if !keep {
                    vtt_log!("Not archiving: empty transcript");
                }
                keep
            }),
        ) {
            let (dir_setting, max_files) = {
                let s = settings.read().unwrap();
                (s.archive_dir.clone(), s.archive_max_files)
            };
            let root = archive::resolve_archive_dir(&dir_setting, &data_dir);
            let now = chrono::Local::now();
            let meta = archive::Sidecar {
                id: now.format("%Y%m%dT%H%M%S_%3f").to_string(),
                recorded_at: now.to_rfc3339(),
                duration_s: native.samples.len() as f64 / native.sample_rate as f64,
                sample_rate: native.sample_rate,
                text,
                model: engine_ref.model_name().to_string(),
                language: language.clone(),
                app: None,
            };
            let date = now.format("%Y-%m-%d").to_string();
            match archive::write_archive(&root, &date, &native.samples, &meta) {
                Ok(path) => {
                    vtt_log!(
                        "Archived {:.2}s at {} Hz to {}",
                        meta.duration_s,
                        meta.sample_rate,
                        path.display()
                    );
                    let removed = archive::prune_archive(&root, max_files);
                    if removed > 0 {
                        vtt_log!("Pruned {} archived recording(s) over the cap", removed);
                    }
                }
                Err(e) => vtt_log!("Archive write failed (dictation unaffected): {}", e),
            }
        }

        if !archive_path.as_os_str().is_empty() {
            save_and_cleanup(&archive_path, &data_dir);
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

/// Batch transcription: decode a 16 kHz WAV and print the transcript to stdout,
/// reusing the tray app's model resolution + download (TASK-VTT023). Runs fully
/// headless — no GTK, no singleton lock, no tray — so it composes in shell
/// pipelines. Progress and diagnostics go to stderr/the log; only the transcript
/// goes to stdout.
fn run_file_mode(path: Option<&str>) -> anyhow::Result<()> {
    let path =
        path.ok_or_else(|| anyhow::anyhow!("--file needs a path, e.g. `--file clip.wav`"))?;

    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("voice-to-text");
    // Deliberately NOT calling logging::init here: the file logger also echoes
    // every line to stdout (harmless for the tray app), which would corrupt a
    // `--file … | …` pipe. With no init the logger's dir stays empty and every
    // vtt_log! is a no-op, so stdout carries only the transcript; errors and
    // progress go to stderr. Silence whisper.cpp/ggml C chatter for the same
    // reason.
    whisper_rs::install_logging_hooks();
    let settings = settings::Settings::load(&data_dir);

    // Decode first so a bad path/format fails fast, before any model download.
    let samples = whisper::decode_wav_to_samples(std::path::Path::new(path))?;
    if samples.is_empty() {
        anyhow::bail!("{}: decoded to zero samples", path);
    }
    // Batch mode exists to reproduce what the app does to a recording, so it
    // honours the same `denoise` setting the live path does. Without this,
    // `--file` would transcribe unfiltered audio and quietly disagree with the
    // hotkey it is meant to debug (TASK-VTT145).
    let samples = if settings.denoise {
        denoise::suppress_rumble(&samples, whisper::WHISPER_INPUT_RATE)
    } else {
        samples
    };

    let migrated = migrate_legacy_model_name(&settings.selected_model);
    let resolved = models::resolve_variant(&migrated, &settings.selected_language);
    let info = models::find(&resolved)
        .or_else(|| models::find(&migrated))
        .or_else(|| models::find("small.en"))
        .ok_or_else(|| anyhow::anyhow!("no usable model in the catalogue"))?;
    let model_path = models::ensure(info, |done, total| {
        let pct = (done * 100).checked_div(total).unwrap_or(0);
        eprint!("\rDownloading {}... {pct}%", info.name);
    })?;
    eprintln!();

    let engine = whisper::WhisperEngine::new(&model_path, info.name)?;
    // Pass the initial prompt and apply corrections, exactly as the live path
    // does. Batch mode exists to reproduce what the hotkey produces; dropping
    // the prompt made it silently disagree with the app it is meant to debug,
    // and made an A/B over settings return "no difference" for every prompt
    // change because neither side ever saw one (TASK-VTT158).
    let prompt = if settings.initial_prompt.trim().is_empty() {
        None
    } else {
        Some(settings.initial_prompt.as_str())
    };
    let text = engine.transcribe(&samples, &settings.selected_language, prompt)?;
    let text = corrections::apply(text.trim(), &settings.corrections);
    println!("{text}");
    Ok(())
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

/// `--doctor`: report whether the running app is the installed one, plus the
/// paths that were misread during the TASK-VTT150 deployment. Exits non-zero
/// when anything is wrong, so it is usable in a script.
fn run_doctor() -> anyhow::Result<()> {
    use doctor::{diagnose_running, format_report, parse_exe_link, Finding};

    let mut findings = vec![Finding::ok("version", env!("CARGO_PKG_VERSION"))];

    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("voice-to-text");
    findings.push(Finding::ok(
        "settings",
        data_dir.join("settings.conf").display().to_string(),
    ));

    // The stale ~/.config copy is inert but still on disk for anyone who ran a
    // pre-2.0 build, and editing it silently does nothing (TASK-VTT150).
    if let Some(cfg) = dirs::config_dir() {
        let stale = cfg.join("voice-to-text").join("settings.conf");
        if stale.exists() {
            findings.push(Finding::problem(
                "stale settings",
                format!(
                    "{} exists and is NOT read — edits there do nothing",
                    stale.display()
                ),
            ));
        }
    }

    let s = settings::Settings::load(&data_dir);
    if s.archive_recordings {
        let resolved = archive::resolve_archive_dir_checked(&s.archive_dir, &data_dir);
        match &resolved {
            archive::ArchiveDir::Rejected { used, why } => findings.push(Finding::problem(
                "archiving",
                format!(
                    "on, but archive_dir was IGNORED ({why}) — writing to {}",
                    used.display()
                ),
            )),
            _ => findings.push(Finding::ok(
                "archiving",
                format!("on -> {}", resolved.path().display()),
            )),
        }
    } else {
        findings.push(Finding::ok(
            "archiving",
            "off (archive=1 in settings.conf enables it)",
        ));
    }

    // Which binary is actually answering the hotkey.
    #[cfg(target_os = "linux")]
    {
        let installed = std::path::Path::new("/usr/bin/vtt-linux");
        let mut seen = false;
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for e in entries.flatten() {
                let name = e.file_name();
                let Some(pid) = name.to_str().and_then(|s| s.parse::<u32>().ok()) else {
                    continue;
                };
                let Ok(target) = std::fs::read_link(format!("/proc/{pid}/exe")) else {
                    continue;
                };
                let target = target.to_string_lossy().to_string();
                if !target.contains("vtt-linux") {
                    continue;
                }
                if pid == std::process::id() {
                    continue;
                }
                seen = true;
                findings.push(diagnose_running(pid, &parse_exe_link(&target), installed));
            }
        }
        if !seen {
            findings.push(Finding::problem(
                "running binary",
                "no vtt-linux process found — nothing is listening for the hotkey. \
                 Start it: `systemctl --user start vtt.service`",
            ));
        }
    }

    let (report, any_problem) = format_report(&findings);
    print!("{report}");
    if any_problem {
        std::process::exit(1);
    }
    Ok(())
}

fn save_and_cleanup(audio_path: &std::path::Path, data_dir: &std::path::Path) {
    let recordings_dir = data_dir.join("recordings");
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

/// The newest `.wav` in `dir` by modification time, or None if the directory is
/// missing/empty or holds no wav files. Backs the tray "Re-transcribe last
/// recording" recovery net (FEAT-VTT039); pruning always keeps the newest
/// recording, so this is a reliable last-recording source.
fn newest_wav(dir: &std::path::Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".wav"))
        .filter_map(|e| {
            let mtime = e.metadata().ok()?.modified().ok()?;
            Some((e.path(), mtime))
        })
        .max_by_key(|(_, mtime)| *mtime)
        .map(|(path, _)| path)
}

// ─── Utilities ──────────────────────────────────────────────────

#[cfg(unix)]
fn singleton_lock(data_dir: &std::path::Path) -> anyhow::Result<std::fs::File> {
    use std::io::Write;
    use std::os::unix::io::AsRawFd;
    std::fs::create_dir_all(data_dir)?;
    let lock_path = data_dir.join("vtt-linux.lock");
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

// TASK-VTT044: Windows singleton via named mutex (CreateMutexW).
// Named mutex in the "Local\" namespace persists for the lifetime of the
// process — no file descriptor to keep open, OS releases on exit.
#[cfg(windows)]
fn singleton_lock() -> anyhow::Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    extern "system" {
        fn CreateMutexW(
            lp_mutex_attributes: *const std::ffi::c_void,
            b_initial_owner: i32,
            lp_name: *const u16,
        ) -> isize;
        fn GetLastError() -> u32;
    }

    const ERROR_ALREADY_EXISTS: u32 = 183;

    let name: Vec<u16> = OsStr::new("Local\\vtt-singleton")
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();

    let handle = unsafe { CreateMutexW(std::ptr::null(), 1, name.as_ptr()) };
    if handle == 0 {
        anyhow::bail!("Failed to create Windows singleton mutex (CreateMutexW returned NULL)");
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        anyhow::bail!(
            "Another instance of vtt is already running. \
             Stop it via Task Manager or run `taskkill /IM vtt.exe /F`."
        );
    }
    // Intentionally never call CloseHandle — the open mutex IS the singleton
    // lock, and Windows releases it when the process exits. (handle is a bare
    // isize with no Drop, so there is nothing to forget; it simply leaks by
    // design.)
    let _ = handle;
    Ok(())
}

// TASK-VTT045: Windows signal handling via SetConsoleCtrlHandler.
// Ctrl+C (event 0), Ctrl+Break (1), and console close (2) all call our
// shutdown closure, matching the Unix SIGINT/SIGTERM behaviour.
#[cfg(windows)]
fn ctrlc_handler<F: Fn() + Send + Sync + 'static>(f: F) {
    use std::sync::OnceLock;

    static HANDLER: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

    extern "system" {
        fn SetConsoleCtrlHandler(
            handler_routine: Option<extern "system" fn(u32) -> i32>,
            add: i32,
        ) -> i32;
    }

    extern "system" fn ctrl_handler(ctrl_type: u32) -> i32 {
        if ctrl_type <= 2 {
            if let Some(h) = HANDLER.get() {
                h();
            }
        }
        0
    }

    HANDLER.get_or_init(|| Box::new(f));
    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), 1);
    }
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
    fn newest_wav_returns_most_recent_by_mtime() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for i in 0..4 {
            std::fs::write(root.join(format!("vtt_recording_{:03}.wav", i)), b"fake").unwrap();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let newest = newest_wav(root).expect("a wav should be found");
        assert!(
            newest
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains("003"),
            "expected the last-written file, got {:?}",
            newest
        );
    }

    #[test]
    fn newest_wav_ignores_non_wav_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(root.join("vtt_recording_only.wav"), b"fake").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        // Written last, but not a .wav — must be ignored.
        std::fs::write(root.join("notes.txt"), b"newer non-wav").unwrap();
        let newest = newest_wav(root).expect("the wav should be found");
        assert!(newest
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with(".wav"));
    }

    #[test]
    fn newest_wav_empty_or_missing_dir_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(newest_wav(dir.path()).is_none(), "empty dir → None");
        assert!(
            newest_wav(&dir.path().join("not-a-dir")).is_none(),
            "missing dir → None"
        );
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
