#include "audio.h"
#include "keyboard.h"
#include "typing.h"
#include "transcribe.h"
#include "gui.h"
#include "../common/logging.h"
#include "../common/queue.h"
#include <pthread.h>
#include <signal.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

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

// Worker thread for transcription
static void *transcription_worker(void *arg) {
    vtt_app_t *app = (vtt_app_t *)arg;

    vtt_log("Transcription worker started");

    while (app->running) {
        char *audio_file = vtt_queue_pop(&app->queue);
        if (!audio_file) {
            break; // Queue shutdown
        }

        vtt_log("Processing: %s", audio_file);
        vtt_gui_set_status(&app->gui, "Transcribing...");

        // Transcribe
        char *text = vtt_transcribe_audio(audio_file);

        if (text && strlen(text) > 0) {
            vtt_log("Transcription: %s", text);

            // Add voice prefix if not already present
            char final_text[8192];
            if (strstr(text, "[Voice]") == NULL) {
                snprintf(final_text, sizeof(final_text), "[Voice] %s", text);
            } else {
                strncpy(final_text, text, sizeof(final_text) - 1);
                final_text[sizeof(final_text) - 1] = '\0';
            }

            // Type the text
            vtt_typing_type_text(&app->typing, final_text);

            free(text);
        }

        free(audio_file);

        vtt_gui_set_status(&app->gui, "Ready");
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

        if (vtt_audio_start_recording(&g_app->audio) != 0) {
            vtt_log("Failed to start recording");
            g_app->recording = false;
            vtt_gui_set_status(&g_app->gui, "Error: Recording failed");
        }

    } else if (event == VTT_KEY_UP && g_app->recording) {
        // Stop recording
        vtt_log("Key released - stopping recording");
        g_app->recording = false;

        char *audio_file = vtt_audio_stop_recording(&g_app->audio);
        if (audio_file) {
            vtt_log("Recording saved: %s", audio_file);
            vtt_queue_push(&g_app->queue, audio_file);
            free(audio_file);
        } else {
            vtt_log("Recording rejected (too short or quiet)");
            vtt_gui_set_status(&g_app->gui, "Ready");
        }
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

    // Set up signal handlers
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);

    // Initialize subsystems
    if (vtt_audio_init(&app.audio) != 0) {
        vtt_log("Failed to initialize audio");
        return 1;
    }

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
    if (vtt_gui_init(&app.gui, &app) != 0) {
        vtt_log("Failed to initialize GUI");
        app.running = false;
        vtt_queue_shutdown(&app.queue);
        pthread_join(app.worker_thread, NULL);
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

    vtt_log("Shutdown complete");
    return 0;
}
