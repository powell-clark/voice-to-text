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
#include <strings.h>
#include <unistd.h>
#include <stdio.h>
#include <ctype.h>
#include <fcntl.h>
#include <sys/file.h>
#include <sys/stat.h>
#include <errno.h>
#include <dirent.h>
#include <limits.h>
#include <time.h>
#include <libnotify/notify.h>
#include <glib.h>
#include <X11/Xlib.h>

#define MAX_RECORDING_HISTORY 20

typedef struct {
    vtt_audio_t audio;
    vtt_keyboard_t keyboard;
    vtt_typing_t typing;
    vtt_gui_t gui;
    vtt_queue_t queue;
    pthread_t worker_thread;
    bool running;
    bool recording;
    volatile bool typing_active;
    bool typing_has_output;
    struct timespec recording_start_ts;  // Wall-clock time recording started
} vtt_app_t;

static vtt_app_t *g_app = NULL;
static void prune_recordings_directory(const char *recordings_dir, size_t max_files);

// Notification data for GLib idle callback
typedef struct {
    char message[256];
} notification_data_t;

static int copy_file_to(const char *src_path, const char *dst_path) {
    int src_fd = open(src_path, O_RDONLY);
    if (src_fd < 0) {
        vtt_log("Failed to open source recording %s: %s", src_path, strerror(errno));
        return -1;
    }

    int dst_fd = open(dst_path, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (dst_fd < 0) {
        vtt_log("Failed to open destination recording %s: %s", dst_path, strerror(errno));
        close(src_fd);
        return -1;
    }

    char buffer[8192];
    ssize_t bytes_read;
    int result = 0;

    while ((bytes_read = read(src_fd, buffer, sizeof(buffer))) > 0) {
        ssize_t total_written = 0;
        while (total_written < bytes_read) {
            ssize_t written = write(dst_fd, buffer + total_written, (size_t)(bytes_read - total_written));
            if (written < 0) {
                vtt_log("Failed writing recording to %s: %s", dst_path, strerror(errno));
                result = -1;
                break;
            }
            total_written += written;
        }
        if (result != 0) {
            break;
        }
    }

    if (bytes_read < 0) {
        vtt_log("Failed reading recording %s: %s", src_path, strerror(errno));
        result = -1;
    }

    close(src_fd);
    close(dst_fd);

    if (result != 0) {
        unlink(dst_path);
    }

    return result;
}

typedef struct {
    char path[PATH_MAX];
    time_t mtime;
} recording_entry_t;

static int compare_recordings_desc(const void *a, const void *b) {
    const recording_entry_t *ra = (const recording_entry_t *)a;
    const recording_entry_t *rb = (const recording_entry_t *)b;
    if (ra->mtime == rb->mtime) {
        return 0;
    }
    return (ra->mtime > rb->mtime) ? -1 : 1;
}

static void prune_recordings_directory(const char *recordings_dir, size_t max_files) {
    if (!recordings_dir || max_files == 0) {
        return;
    }

    DIR *dir = opendir(recordings_dir);
    if (!dir) {
        vtt_log("Unable to open recordings directory %s: %s", recordings_dir, strerror(errno));
        return;
    }

    recording_entry_t *entries = NULL;
    size_t count = 0;

    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL) {
        if (entry->d_name[0] == '.') {
            continue;
        }

        size_t name_len = strlen(entry->d_name);
        if (name_len < 4 || strcmp(entry->d_name + name_len - 4, ".wav") != 0) {
            continue;
        }

        char full_path[PATH_MAX];
        if ((snprintf(full_path, sizeof(full_path), "%s/%s", recordings_dir, entry->d_name)) >= (int)sizeof(full_path)) {
            continue;
        }

        struct stat st;
        if (stat(full_path, &st) != 0 || !S_ISREG(st.st_mode)) {
            continue;
        }

        recording_entry_t *tmp = realloc(entries, sizeof(recording_entry_t) * (count + 1));
        if (!tmp) {
            vtt_log("Failed to allocate recording list while pruning");
            free(entries);
            closedir(dir);
            return;
        }
        entries = tmp;
        strncpy(entries[count].path, full_path, sizeof(entries[count].path) - 1);
        entries[count].path[sizeof(entries[count].path) - 1] = '\0';
        entries[count].mtime = st.st_mtime;
        count++;
    }

    closedir(dir);

    if (count <= max_files) {
        free(entries);
        return;
    }

    qsort(entries, count, sizeof(recording_entry_t), compare_recordings_desc);

    for (size_t i = max_files; i < count; i++) {
        if (unlink(entries[i].path) == 0) {
            vtt_log("Pruned old recording: %s", entries[i].path);
        }
    }

    free(entries);
}

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
             "Recording limit reached - release key to transcribe");

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
        char *text = vtt_transcribe_audio(actual_filename, app->gui.selected_model, app->gui.selected_language, app->gui.initial_prompt);

        if (text && strlen(text) > 0) {
            char *trimmed = text;
            while (*trimmed && isspace((unsigned char)*trimmed)) {
                trimmed++;
            }

            // Trim trailing whitespace in-place
            char *trim_end = trimmed + strlen(trimmed);
            while (trim_end > trimmed && isspace((unsigned char)trim_end[-1])) {
                *--trim_end = '\0';
            }

            if (*trimmed == '\0' ||
                strcasecmp(trimmed, "[BLANK_AUDIO]") == 0 ||
                strcasecmp(trimmed, "[MUSIC PLAYING]") == 0) {
                vtt_log("Skipping blank transcription result");
                free(text);
                text = NULL;
                goto post_transcription;
            }

            // Check if text contains at least some alphanumeric content (not just punctuation/brackets)
            int has_content = 0;
            for (const char *p = trimmed; *p; p++) {
                if (isalnum(*p)) {
                    has_content = 1;
                    break;
                }
            }

            if (has_content) {
                vtt_log("Transcription: %s", trimmed);

                // Add voice prefix if not already present (using custom prefix from GUI)
                const char *prefix = app->gui.voice_prefix ? app->gui.voice_prefix : "[Voice] ";
                char final_text[8192];
                size_t prefix_len = strlen(prefix);
                bool has_prefix = false;

                if (prefix_len > 0 && strncasecmp(trimmed, prefix, prefix_len) == 0) {
                    has_prefix = true;
                }

                if (is_truncated) {
                    // Add truncation indicator before voice prefix
                    snprintf(final_text, sizeof(final_text), "[Truncated] %s%s", prefix, trimmed);
                } else if (!has_prefix) {
                    snprintf(final_text, sizeof(final_text), "%s%s", prefix, trimmed);
                } else {
                    strncpy(final_text, trimmed, sizeof(final_text) - 1);
                    final_text[sizeof(final_text) - 1] = '\0';
                }

                // Type the text (optionally prepend newline between messages)
                app->typing_active = true;
                // Sync newline type setting
                app->typing.newline_type = app->gui.newline_type;
                if (app->gui.append_newline && app->typing_has_output) {
                    vtt_typing_type_text(&app->typing, "\n");
                }
                vtt_typing_type_text(&app->typing, final_text);
                app->typing_active = false;
                app->typing_has_output = true;
            } else {
                vtt_log("Skipping empty/punctuation-only transcription: %s", trimmed);
            }

            free(text);
            text = NULL;
        }

post_transcription:
        // Clean up WAV file after processing
        const char *log_dir = vtt_log_get_path();
        if (log_dir && *log_dir) {
            char backup_dir[512];
            strncpy(backup_dir, log_dir, sizeof(backup_dir) - 1);
            backup_dir[sizeof(backup_dir) - 1] = '\0';

            char *slash = strrchr(backup_dir, '/');
            if (slash) {
                *slash = '\0';
            }

            char recordings_dir[512];
            int dir_result = snprintf(recordings_dir, sizeof(recordings_dir), "%s/recordings", backup_dir);
            if (dir_result > 0 && dir_result < (int)sizeof(recordings_dir)) {
                mkdir(recordings_dir, 0755);

                const char *filename_only = strrchr(actual_filename, '/');
                filename_only = filename_only ? filename_only + 1 : actual_filename;

                char backup_file[1024];
                int file_result = snprintf(backup_file, sizeof(backup_file), "%s/%s", recordings_dir, filename_only);

                if (file_result > 0 && file_result < (int)sizeof(backup_file)) {
                    if (copy_file_to(actual_filename, backup_file) == 0) {
                        unlink(actual_filename);
                        vtt_log("Saved recording to %s", backup_file);
                        prune_recordings_directory(recordings_dir, MAX_RECORDING_HISTORY);
                        goto after_cleanup;
                    }
                }
            }

            vtt_log("Cleaning up temporary recording: %s", actual_filename);
            remove(actual_filename);
        } else {
            if (remove(actual_filename) == 0) {
                vtt_log("Cleaned up audio file: %s", actual_filename);
            }
        }

after_cleanup:

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
        while (g_app->typing_active) {
            usleep(1000);
        }

        // Start recording
        vtt_log("Key pressed - starting recording");
        clock_gettime(CLOCK_MONOTONIC, &g_app->recording_start_ts);
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
        // Guard against stale KeyRelease events that queued while we were
        // busy-waiting on typing_active. If the recording just started
        // (< 150ms ago), this is almost certainly a stale event — ignore it.
        struct timespec now;
        clock_gettime(CLOCK_MONOTONIC, &now);
        long elapsed_ms = (now.tv_sec - g_app->recording_start_ts.tv_sec) * 1000 +
                          (now.tv_nsec - g_app->recording_start_ts.tv_nsec) / 1000000;
        if (elapsed_ms < 150) {
            vtt_log("Ignoring stale KeyRelease (%ldms after recording start)", elapsed_ms);
            return;
        }

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
    // Initialize Xlib threading support (must be first Xlib call)
    XInitThreads();

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
    if (vtt_gui_init(&app.gui, &app.audio, &app.keyboard, &app.recording, log_dir) != 0) {
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
    if (vtt_settings_load(&hotkey_settings, log_dir) == 0 &&
        hotkey_settings.hotkey_keycode >= 8 && hotkey_settings.hotkey_keycode <= 255) {
        if (vtt_keyboard_set_hotkey(&app.keyboard, hotkey_settings.hotkey_keycode) == 0) {
            vtt_log("Applied custom hotkey from settings: keycode %d", hotkey_settings.hotkey_keycode);
        } else {
            vtt_log("Failed to apply saved hotkey %d, reverting to default", hotkey_settings.hotkey_keycode);
        }
    }
    vtt_settings_cleanup(&hotkey_settings);

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
