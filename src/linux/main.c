#include "audio.h"
#include "keyboard.h"
#include "typing.h"
#include "transcribe.h"
#include "gui.h"
#include "../common/logging.h"
#include "../common/queue.h"
#include "../common/settings.h"
#include <pthread.h>
#include <signal.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <stdio.h>
#include <ctype.h>
#include <fcntl.h>
#include <sys/file.h>
#include <sys/stat.h>
#include <libnotify/notify.h>
#include <glib.h>

typedef struct {
    vtt_audio_t audio;
    vtt_keyboard_t keyboard;
    vtt_typing_t typing;
    vtt_gui_t gui;
    vtt_queue_t queue;
    pthread_t worker_thread;
    bool running;
    bool recording;
} vtt_app_t;

static vtt_app_t *g_app = NULL;

// Notification data for GLib idle callback
typedef struct {
    char message[256];
} notification_data_t;

// GLib idle callback to show notification from main thread
static gboolean show_notification_idle(gpointer user_data) {
    notification_data_t *data = (notification_data_t *)user_data;

    NotifyNotification *notification = notify_notification_new(
        "Voice to Text",
        data->message,
        "dialog-information"
    );

    notify_notification_set_timeout(notification, 3000);  // 3 seconds
    notify_notification_show(notification, NULL);
    g_object_unref(notification);

    free(data);
    return G_SOURCE_REMOVE;
}

// Audio buffer full callback (called from audio thread)
static void on_buffer_full(void *user_data) {
    (void)user_data;

    // Post notification to main GTK thread using g_idle_add
    notification_data_t *data = malloc(sizeof(notification_data_t));
    snprintf(data->message, sizeof(data->message),
             "Recording limit reached (%ds) - release key to transcribe", 120);

    g_idle_add(show_notification_idle, data);
}

// Worker thread for transcription
static void *transcription_worker(void *arg) {
    vtt_app_t *app = (vtt_app_t *)arg;

    vtt_log("Transcription worker started");

    while (app->running) {
        char *audio_file = vtt_queue_pop(&app->queue);
        if (!audio_file) {
            break; // Queue shutdown
        }

        // Check if this recording was truncated at max length
        bool is_truncated = (strncmp(audio_file, "TRUNCATED:", 10) == 0);
        const char *actual_filename = is_truncated ? (audio_file + 10) : audio_file;

        vtt_log("Processing: %s%s", actual_filename, is_truncated ? " (TRUNCATED)" : "");
        vtt_gui_set_status(&app->gui, "Transcribing...");
        vtt_gui_set_icon(&app->gui, "processing");

        // Transcribe with selected model and language
        char *text = vtt_transcribe_audio(actual_filename, app->gui.selected_model, app->gui.selected_language);

        if (text && strlen(text) > 0) {
            // Check if text contains at least some alphanumeric content (not just punctuation/brackets)
            int has_content = 0;
            for (const char *p = text; *p; p++) {
                if (isalnum(*p)) {
                    has_content = 1;
                    break;
                }
            }

            if (has_content) {
                vtt_log("Transcription: %s", text);

                // Add voice prefix if not already present (using custom prefix from GUI)
                const char *prefix = app->gui.voice_prefix ? app->gui.voice_prefix : "[Voice] ";
                char final_text[8192];

                if (is_truncated) {
                    // Add truncation indicator before voice prefix
                    snprintf(final_text, sizeof(final_text), "[Truncated - 120s limit] %s%s", prefix, text);
                } else if (strstr(text, prefix) == NULL) {
                    snprintf(final_text, sizeof(final_text), "%s%s", prefix, text);
                } else {
                    strncpy(final_text, text, sizeof(final_text) - 1);
                    final_text[sizeof(final_text) - 1] = '\0';
                }

                // Type the text
                vtt_typing_type_text(&app->typing, final_text);
            } else {
                vtt_log("Skipping empty/punctuation-only transcription: %s", text);
            }

            free(text);
        }

        // Clean up WAV file after processing
        if (remove(actual_filename) == 0) {
            vtt_log("Cleaned up audio file: %s", actual_filename);
        }

        free(audio_file);

        vtt_gui_set_status(&app->gui, "Ready");
        vtt_gui_set_icon(&app->gui, "ready");
    }

    vtt_log("Transcription worker stopped");
    return NULL;
}

// Keyboard callback
static void on_key_event(vtt_key_event_t event) {
    if (!g_app) return;

    if (event == VTT_KEY_DOWN && !g_app->recording) {
        // Start recording
        vtt_log("Key pressed - starting recording");
        g_app->recording = true;
        vtt_gui_set_status(&g_app->gui, "Recording...");
        vtt_gui_set_icon(&g_app->gui, "recording");

        if (vtt_audio_start_recording(&g_app->audio) != 0) {
            vtt_log("Failed to start recording");
            g_app->recording = false;
            vtt_gui_set_status(&g_app->gui, "Error: Recording failed");
            vtt_gui_set_icon(&g_app->gui, "ready");
        }

    } else if (event == VTT_KEY_UP && g_app->recording) {
        // Stop recording
        vtt_log("Key released - stopping recording");
        g_app->recording = false;

        char *audio_file = vtt_audio_stop_recording(&g_app->audio);
        if (audio_file) {
            // Check if this is a rejection marker
            if (strncmp(audio_file, "REJECTED:", 9) == 0) {
                // Handle rejection - type descriptive message acknowledging the keypress
                char message[256];

                if (strstr(audio_file, "TOO_SHORT")) {
                    snprintf(message, sizeof(message), "[Transcription activated: audio too short]");
                } else if (strstr(audio_file, "TOO_QUIET")) {
                    snprintf(message, sizeof(message), "[Transcription activated: no audio detected]");
                } else {
                    // Should never happen - log for debugging
                    vtt_log("Unknown rejection type: %s", audio_file);
                    snprintf(message, sizeof(message), "[Transcription activated]");
                }

                vtt_typing_type_text(&g_app->typing, message);
                vtt_gui_set_status(&g_app->gui, "Ready");
                vtt_gui_set_icon(&g_app->gui, "ready");
                free(audio_file);
            } else if (strncmp(audio_file, "MAX_LENGTH_REACHED:", 19) == 0) {
                // Handle max length reached - system notification already shown, just transcribe
                vtt_log("Max recording length reached");

                // Extract actual filename after the marker
                const char *actual_filename = audio_file + 19; // Skip "MAX_LENGTH_REACHED:"
                vtt_log("Recording saved: %s", actual_filename);

                // Queue for transcription with TRUNCATED marker so transcription worker knows to add prefix
                vtt_gui_set_status(&g_app->gui, "Loading model...");
                vtt_gui_set_icon(&g_app->gui, "processing");

                char *marked_filename = malloc(strlen(actual_filename) + 32);
                snprintf(marked_filename, strlen(actual_filename) + 32, "TRUNCATED:%s", actual_filename);
                vtt_queue_push(&g_app->queue, marked_filename);
                free(marked_filename);
                free(audio_file);
            } else {
                // Normal recording - queue for transcription
                vtt_log("Recording saved: %s", audio_file);

                // Immediately show loading/processing state
                vtt_gui_set_status(&g_app->gui, "Loading model...");
                vtt_gui_set_icon(&g_app->gui, "processing");

                vtt_queue_push(&g_app->queue, audio_file);
                free(audio_file);
            }
        } else {
            // Should never happen now, but handle gracefully
            vtt_log("Recording returned NULL");
            vtt_gui_set_status(&g_app->gui, "Ready");
            vtt_gui_set_icon(&g_app->gui, "ready");
        }
    }
}

// Cleanup old WAV files from /tmp
static void cleanup_old_wav_files(void) {
    char command[256];
    snprintf(command, sizeof(command), "find /tmp -name 'vtt_recording_*.wav' -mmin +60 -delete 2>/dev/null");
    int result = system(command);
    if (result == 0) {
        vtt_log("Cleaned up old WAV files from /tmp");
    }
}

// Signal handler
static void signal_handler(int sig) {
    if (g_app) {
        vtt_log("Signal %d received, shutting down", sig);
        g_app->running = false;
        vtt_queue_shutdown(&g_app->queue);
    }
    exit(0);
}

int main(int argc, char *argv[]) {
    // Singleton lock: ensure only one instance runs
    char lockfile[512];
    snprintf(lockfile, sizeof(lockfile), "%s/.local/share/voice-to-text/vtt-linux.lock", getenv("HOME"));

    int lockfd = open(lockfile, O_CREAT | O_RDWR, 0666);
    if (lockfd == -1) {
        fprintf(stderr, "Error: Cannot create lock file\n");
        return 1;
    }

    if (flock(lockfd, LOCK_EX | LOCK_NB) == -1) {
        fprintf(stderr, "Error: Another instance of vtt-linux is already running\n");
        close(lockfd);
        return 1;
    }

    vtt_app_t app;
    memset(&app, 0, sizeof(app));
    app.running = true;
    app.recording = false;
    g_app = &app;

    // Initialize logging
    char log_dir[512];
    snprintf(log_dir, sizeof(log_dir), "%s/.local/share/voice-to-text", getenv("HOME"));
    vtt_log_init(log_dir);

    vtt_log("===========================================");
    vtt_log("Voice to Text - Starting");
    vtt_log("===========================================");

    // Initialize libnotify
    if (!notify_init("Voice to Text")) {
        vtt_log("Failed to initialize libnotify");
        return 1;
    }

    // Clean up any orphaned WAV files from previous runs (older than 60 minutes)
    cleanup_old_wav_files();

    // Set up signal handlers
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);

    // Initialize subsystems
    if (vtt_audio_init(&app.audio) != 0) {
        vtt_log("Failed to initialize audio");
        notify_uninit();
        return 1;
    }

    // Register buffer full callback for notifications
    vtt_audio_set_buffer_full_callback(&app.audio, on_buffer_full, &app);

    if (vtt_keyboard_init(&app.keyboard, on_key_event) != 0) {
        vtt_log("Failed to initialize keyboard hook");
        vtt_audio_cleanup(&app.audio);
        return 1;
    }

    if (vtt_typing_init(&app.typing) != 0) {
        vtt_log("Failed to initialize typing");
        vtt_keyboard_cleanup(&app.keyboard);
        vtt_audio_cleanup(&app.audio);
        return 1;
    }

    // Initialize queue
    vtt_queue_init(&app.queue);

    // Start worker thread
    if (pthread_create(&app.worker_thread, NULL, transcription_worker, &app) != 0) {
        vtt_log("Failed to create worker thread");
        vtt_typing_cleanup(&app.typing);
        vtt_keyboard_cleanup(&app.keyboard);
        vtt_audio_cleanup(&app.audio);
        return 1;
    }

    // Initialize GUI
    if (vtt_gui_init(&app.gui, &app, log_dir) != 0) {
        vtt_log("Failed to initialize GUI");
        app.running = false;
        vtt_queue_shutdown(&app.queue);
        pthread_join(app.worker_thread, NULL);
        vtt_typing_cleanup(&app.typing);
        vtt_keyboard_cleanup(&app.keyboard);
        vtt_audio_cleanup(&app.audio);
        return 1;
    }

    // Load hotkey from settings and apply it
    vtt_settings_t hotkey_settings;
    vtt_settings_init(&hotkey_settings);
    if (vtt_settings_load(&hotkey_settings, log_dir) == 0 && hotkey_settings.hotkey_keycode != 0) {
        vtt_keyboard_set_hotkey(&app.keyboard, hotkey_settings.hotkey_keycode);
        vtt_log("Applied custom hotkey from settings: keycode %d", hotkey_settings.hotkey_keycode);
    }
    vtt_settings_cleanup(&hotkey_settings);

    // Populate microphone menu
    vtt_gui_update_microphones(&app.gui);

    // Open audio stream now that GUI is initialized (eliminates latency on key press)
    if (vtt_audio_open_stream(&app.audio) != 0) {
        vtt_log("Failed to open audio stream");
        app.running = false;
        vtt_queue_shutdown(&app.queue);
        pthread_join(app.worker_thread, NULL);
        vtt_gui_cleanup(&app.gui);
        vtt_typing_cleanup(&app.typing);
        vtt_keyboard_cleanup(&app.keyboard);
        vtt_audio_cleanup(&app.audio);
        return 1;
    }

    // Start keyboard monitoring
    if (vtt_keyboard_start(&app.keyboard) != 0) {
        vtt_log("Failed to start keyboard monitoring");
        app.running = false;
        vtt_queue_shutdown(&app.queue);
        pthread_join(app.worker_thread, NULL);
        vtt_gui_cleanup(&app.gui);
        vtt_typing_cleanup(&app.typing);
        vtt_keyboard_cleanup(&app.keyboard);
        vtt_audio_cleanup(&app.audio);
        return 1;
    }

    vtt_log("All systems initialized");
    vtt_gui_set_status(&app.gui, "Ready");
    vtt_gui_set_icon(&app.gui, "ready");

    // Run GUI main loop (blocks until quit)
    vtt_gui_run(&app.gui);

    // Cleanup
    vtt_log("Shutting down...");
    app.running = false;
    vtt_queue_shutdown(&app.queue);
    pthread_join(app.worker_thread, NULL);

    vtt_keyboard_stop(&app.keyboard);
    vtt_keyboard_cleanup(&app.keyboard);
    vtt_typing_cleanup(&app.typing);
    vtt_audio_cleanup(&app.audio);
    vtt_queue_destroy(&app.queue);
    vtt_gui_cleanup(&app.gui);

    notify_uninit();

    vtt_log("Shutdown complete");
    return 0;
}
