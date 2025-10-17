#include "gui.h"
#include "audio.h"
#include "keyboard.h"
#include "../common/logging.h"
#include "../common/settings.h"
#include <gtk/gtk.h>
#include <libayatana-appindicator/app-indicator.h>
#include <gdk/gdkx.h>
#include <X11/Xlib.h>
#include <ctype.h>
#include <stdlib.h>
#include <string.h>

// Callbacks
static void on_quit(GtkMenuItem *item, gpointer user_data);
static void on_model_selected(GtkMenuItem *item, gpointer user_data);
static void on_language_selected(GtkMenuItem *item, gpointer user_data);
static void on_show_logs(GtkMenuItem *item, gpointer user_data);
static void on_toggle_logging(GtkMenuItem *item, gpointer user_data);
static void on_about(GtkMenuItem *item, gpointer user_data);
static void on_customize_prompt(GtkMenuItem *item, gpointer user_data);
static void on_change_hotkey(GtkMenuItem *item, gpointer user_data);

// Helper callbacks for prompt dialog
static void on_prompt_text_changed(GtkTextBuffer *buffer, gpointer user_data);
static void on_prompt_reset(GtkButton *button, gpointer user_data);
static void on_prompt_save(GtkButton *button, gpointer user_data);
static void on_prompt_dialog_destroy(GtkWidget *widget, gpointer user_data);

static void format_device_label(char *dest, size_t dest_size, const char *raw_name, bool mark_default);

// Helper to rebuild model menu based on language (truly dynamic)
static void rebuild_model_menu(vtt_gui_t *gui);
static void update_microphone_label(vtt_gui_t *gui);
static gboolean refresh_microphone_label(gpointer user_data);

static void copy_truncated(char *dest, size_t dest_size, const char *src);
static void append_truncated(char *dest, size_t dest_size, const char *suffix);
static void trim_trailing_whitespace(char *str);
static void replace_underscores(char *dest, size_t dest_size, const char *src);

static bool read_line_trim(FILE *fp, char *buffer, size_t buffer_size) {
    if (!fp || !buffer || buffer_size == 0) {
        return false;
    }

    if (!fgets(buffer, (int)buffer_size, fp)) {
        return false;
    }

    size_t len = strlen(buffer);
    while (len > 0 && (buffer[len - 1] == '\n' || buffer[len - 1] == '\r')) {
        buffer[len - 1] = '\0';
        len--;
    }
    return true;
}

static bool get_source_description_for_name(const char *source_name, char *dest, size_t dest_size);

static bool get_default_source_description(char *dest, size_t dest_size) {
    if (!dest || dest_size == 0) {
        return false;
    }

    FILE *name_fp = popen("pactl get-default-source 2>/dev/null", "r");
    if (!name_fp) {
        return false;
    }

    char source_name[256];
    if (!read_line_trim(name_fp, source_name, sizeof(source_name)) || source_name[0] == '\0') {
        pclose(name_fp);
        return false;
    }
    pclose(name_fp);

    return get_source_description_for_name(source_name, dest, dest_size);
}

static bool get_source_description_for_name(const char *source_name, char *dest, size_t dest_size) {
    if (!source_name || !dest || dest_size == 0) {
        return false;
    }

    FILE *list_fp = popen("pactl list sources 2>/dev/null", "r");
    if (!list_fp) {
        return false;
    }

    char line[1024];
    char current_name[256] = {0};
    bool match = false;

    while (fgets(line, sizeof(line), list_fp)) {
        if (strncmp(line, "Source #", 8) == 0) {
            current_name[0] = '\0';
            match = false;
            continue;
        }

        if (strncmp(line, "\tName: ", 7) == 0) {
            copy_truncated(current_name, sizeof(current_name), line + 7);
            trim_trailing_whitespace(current_name);
            match = (strcmp(current_name, source_name) == 0);
            continue;
        }

        if (match && strncmp(line, "\tDescription: ", 14) == 0) {
            char description[512];
            copy_truncated(description, sizeof(description), line + 14);
            trim_trailing_whitespace(description);
            copy_truncated(dest, dest_size, description);
            pclose(list_fp);
            return true;
        }
    }

    pclose(list_fp);
    return false;
}

static void update_microphone_label(vtt_gui_t *gui) {
    if (!gui || !gui->mic_label_item) {
        return;
    }

    char display[256];
    if (!get_default_source_description(display, sizeof(display))) {
        format_device_label(display, sizeof(display), "default", true);
    }

    char label[256];
    snprintf(label, sizeof(label), "Microphone: %.220s", display);
    gtk_menu_item_set_label(GTK_MENU_ITEM(gui->mic_label_item), label);
}

static gboolean refresh_microphone_label(gpointer user_data) {
    vtt_gui_t *gui = (vtt_gui_t *)user_data;
    update_microphone_label(gui);
    return G_SOURCE_CONTINUE;
}

static void populate_settings_snapshot(vtt_gui_t *gui, vtt_settings_t *settings) {
    if (!settings) return;
    memset(settings, 0, sizeof(*settings));

    settings->selected_model = gui->selected_model;
    settings->selected_language = gui->selected_language;
    settings->voice_prefix = gui->voice_prefix;
    settings->initial_prompt = gui->initial_prompt;
    settings->selected_device_index = -1;

    if (gui->keyboard) {
        int current_hotkey = vtt_keyboard_get_hotkey(gui->keyboard);
        if (current_hotkey >= 8 && current_hotkey <= 255) {
            settings->hotkey_keycode = current_hotkey;
        }
    }

    settings->append_newline = gui->append_newline;
}

static void copy_truncated(char *dest, size_t dest_size, const char *src) {
    if (!dest || dest_size == 0) {
        return;
    }
    if (!src) src = "";

    size_t max_copy = dest_size - 1;
    if (max_copy == 0) {
        dest[0] = '\0';
        return;
    }

    size_t src_len = strlen(src);
    if (src_len > max_copy) {
        src_len = max_copy;
    }

    memcpy(dest, src, src_len);
    dest[src_len] = '\0';
}

static void append_truncated(char *dest, size_t dest_size, const char *suffix) {
    if (!dest || dest_size == 0 || !suffix) {
        return;
    }

    size_t current_len = strlen(dest);
    if (current_len >= dest_size - 1) {
        return;
    }

    strncat(dest, suffix, dest_size - current_len - 1);
}

static void trim_trailing_whitespace(char *str) {
    if (!str) {
        return;
    }

    size_t len = strlen(str);
    while (len > 0 && isspace((unsigned char)str[len - 1])) {
        str[len - 1] = '\0';
        len--;
    }
}

static void replace_underscores(char *dest, size_t dest_size, const char *src) {
    size_t j = 0;
    for (size_t i = 0; src[i] != '\0' && j < dest_size - 1; i++) {
        if (src[i] == '_' && src[i + 1] == '_') {
            dest[j++] = 'O';
            i++;  // Skip second underscore
        } else if (src[i] == '_') {
            dest[j++] = ' ';
        } else {
            dest[j++] = src[i];
        }
    }
    dest[j] = '\0';
    trim_trailing_whitespace(dest);
}

static void format_device_label(char *dest, size_t dest_size, const char *raw_name, bool mark_default) {
    if (!dest || dest_size == 0) {
        return;
    }

    if (!raw_name || raw_name[0] == '\0') {
        snprintf(dest, dest_size, "Unknown device");
        return;
    }

    if (strcmp(raw_name, "default") == 0 || strcmp(raw_name, "default (default)") == 0) {
        copy_truncated(dest, dest_size, "System Default (PipeWire)");
        return;
    }

    if (strcmp(raw_name, "pulse") == 0) {
        copy_truncated(dest, dest_size, "PulseAudio Compatibility");
        return;
    }

    if (strcmp(raw_name, "pipewire") == 0) {
        copy_truncated(dest, dest_size, "PipeWire (Direct)");
        return;
    }

    char working[256];
    strncpy(working, raw_name, sizeof(working) - 1);
    working[sizeof(working) - 1] = '\0';

    // Remove technical suffixes like "(hw:2,0)"
    char *paren = strchr(working, '(');
    if (paren && strstr(paren, "hw:")) {
        *paren = '\0';
    }
    trim_trailing_whitespace(working);

    // Handle ALSA PipeWire-style names: alsa_input.usb-<device>-00.mono-fallback
    if (strncmp(working, "alsa_input.", 11) == 0 || strncmp(working, "alsa_output.", 12) == 0) {
        const char *usb = strstr(working, "usb-");
        if (usb) {
            usb += 4;
            size_t len = strcspn(usb, ".");
            if (len >= sizeof(working)) {
                len = sizeof(working) - 1;
            }

            char extracted[256];
            strncpy(extracted, usb, len);
            extracted[len] = '\0';

            char *dash_suffix = strrchr(extracted, '-');
            if (dash_suffix && strlen(dash_suffix) <= 3) {
                *dash_suffix = '\0';
            }

            char friendly[256];
            replace_underscores(friendly, sizeof(friendly), extracted);

            if (friendly[0] != '\0') {
                copy_truncated(dest, dest_size, friendly);
                append_truncated(dest, dest_size, " (USB)");
                return;
            }
        }

        const char *after_colon = strchr(working, ':');
        if (after_colon) {
            after_colon++;
            while (*after_colon == ' ') after_colon++;
            if (*after_colon) {
                copy_truncated(dest, dest_size, after_colon);
                trim_trailing_whitespace(dest);
                if (mark_default && strstr(dest, "default") == NULL) {
                    append_truncated(dest, dest_size, " (default)");
                }
                return;
            }
        }
    }

    if (working[0] != '\0') {
        if (mark_default && strstr(working, "default") == NULL) {
            copy_truncated(dest, dest_size, working);
            append_truncated(dest, dest_size, " (default)");
        } else {
            copy_truncated(dest, dest_size, working);
        }
        trim_trailing_whitespace(dest);
        return;
    }

    copy_truncated(dest, dest_size, raw_name);
}

typedef struct {
    vtt_gui_t *gui;
    GtkWidget *prefix_entry;
    GtkTextBuffer *text_buffer;
    GtkWidget *dialog;
    GtkWidget *newline_toggle;
} prompt_dialog_data_t;

int vtt_gui_init(vtt_gui_t *gui,
                 vtt_audio_t *audio,
                 vtt_keyboard_t *keyboard,
                 bool *recording_flag,
                 const char *config_dir) {
    memset(gui, 0, sizeof(vtt_gui_t));
    gui->config_dir = strdup(config_dir);
    gui->logging_enabled = true;
    gui->initializing = true;  // Prevent saving settings during initialization
    gui->audio = audio;
    gui->keyboard = keyboard;
    gui->recording_flag = recording_flag;

    // Load settings from disk (or use defaults if not found)
    vtt_settings_t settings;
    vtt_settings_init(&settings);
    vtt_settings_load(&settings, config_dir);

    gui->selected_model = strdup(settings.selected_model);
    gui->selected_language = strdup(settings.selected_language);
    gui->voice_prefix = strdup(settings.voice_prefix);
    gui->initial_prompt = strdup(settings.initial_prompt);
    gui->append_newline = settings.append_newline;

    vtt_settings_cleanup(&settings);

    vtt_log("Loaded settings: model=%s, language=%s, prefix=%s",
            gui->selected_model, gui->selected_language, gui->voice_prefix);

    // Initialize GTK
    gtk_init(NULL, NULL);

    // Create indicator
    AppIndicator *indicator = app_indicator_new(
        "voice-to-text-linux",
        "audio-input-microphone",
        APP_INDICATOR_CATEGORY_APPLICATION_STATUS
    );

    app_indicator_set_status(indicator, APP_INDICATOR_STATUS_ACTIVE);
    app_indicator_set_title(indicator, "VTTL");

    gui->indicator = indicator;

    // Create menu
    GtkWidget *menu = gtk_menu_new();

    // Status item (non-clickable)
    GtkWidget *status_item = gtk_menu_item_new_with_label("Status: Initializing...");
    gtk_widget_set_sensitive(status_item, FALSE);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), status_item);
    gui->status_item = status_item;

    gtk_menu_shell_append(GTK_MENU_SHELL(menu), gtk_separator_menu_item_new());

    // Language selector (appears BEFORE model menu so it can filter models)
    const char *lang_display = strcmp(gui->selected_language, "en") == 0 ? "English only" : "Multilingual";
    char language_label[256];
    snprintf(language_label, sizeof(language_label), "Language: %s", lang_display);
    GtkWidget *language_item = gtk_menu_item_new_with_label(language_label);
    GtkWidget *language_menu = gtk_menu_new();

    // English only (fastest, .en models available)
    GtkWidget *lang_en = gtk_radio_menu_item_new_with_label(NULL, "English only (fastest)");
    g_object_set_data_full(G_OBJECT(lang_en), "language", g_strdup("en"), g_free);
    gtk_check_menu_item_set_active(GTK_CHECK_MENU_ITEM(lang_en), strcmp(gui->selected_language, "en") == 0);
    g_signal_connect(lang_en, "activate", G_CALLBACK(on_language_selected), gui);
    gtk_menu_shell_append(GTK_MENU_SHELL(language_menu), lang_en);

    // Multilingual (99 languages, disables .en models)
    GtkWidget *lang_multi = gtk_radio_menu_item_new_with_label_from_widget(GTK_RADIO_MENU_ITEM(lang_en), "Multilingual (99 languages)");
    g_object_set_data_full(G_OBJECT(lang_multi), "language", g_strdup("auto"), g_free);
    gtk_check_menu_item_set_active(GTK_CHECK_MENU_ITEM(lang_multi), strcmp(gui->selected_language, "auto") == 0);
    g_signal_connect(lang_multi, "activate", G_CALLBACK(on_language_selected), gui);
    gtk_menu_shell_append(GTK_MENU_SHELL(language_menu), lang_multi);

    gtk_menu_item_set_submenu(GTK_MENU_ITEM(language_item), language_menu);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), language_item);
    gui->language_item = language_item;

    // Model submenu (W = whisper.cpp, CT2 = CTranslate2/faster-whisper)
    char model_label[256];
    snprintf(model_label, sizeof(model_label), "Model: %s", gui->selected_model);
    GtkWidget *model_item = gtk_menu_item_new_with_label(model_label);
    GtkWidget *model_menu = gtk_menu_new();

    // === MULTILINGUAL MODELS (support 99 languages with auto-detect) ===
    GtkWidget *multilingual_header = gtk_menu_item_new_with_label("Multilingual (99 languages):");
    gtk_widget_set_sensitive(multilingual_header, FALSE);
    gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), multilingual_header);

    // Whisper.cpp multilingual models
    const char *w_multilingual[] = {"W tiny", "W base", "W small", "W medium", "W large", NULL};
    for (int i = 0; w_multilingual[i]; i++) {
        GtkWidget *item = gtk_check_menu_item_new_with_label(w_multilingual[i]);
        char *model_name = g_strdup(w_multilingual[i]);
        g_object_set_data_full(G_OBJECT(item), "model", model_name, g_free);
        gtk_check_menu_item_set_active(GTK_CHECK_MENU_ITEM(item), strcmp(gui->selected_model, w_multilingual[i]) == 0);
        g_signal_connect(item, "activate", G_CALLBACK(on_model_selected), gui);
        gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), item);
    }

    gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), gtk_separator_menu_item_new());

    // CTranslate2 multilingual models
    const char *ct2_multilingual[] = {"CT2 tiny", "CT2 base", "CT2 small", "CT2 medium", "CT2 large-v3", NULL};
    for (int i = 0; ct2_multilingual[i]; i++) {
        GtkWidget *item = gtk_check_menu_item_new_with_label(ct2_multilingual[i]);
        char *model_name = g_strdup(ct2_multilingual[i]);
        g_object_set_data_full(G_OBJECT(item), "model", model_name, g_free);
        gtk_check_menu_item_set_active(GTK_CHECK_MENU_ITEM(item), strcmp(gui->selected_model, ct2_multilingual[i]) == 0);
        g_signal_connect(item, "activate", G_CALLBACK(on_model_selected), gui);
        gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), item);
    }

    gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), gtk_separator_menu_item_new());

    // === ENGLISH-ONLY MODELS (faster, English only) ===
    GtkWidget *english_header = gtk_menu_item_new_with_label("English Only (faster):");
    gtk_widget_set_sensitive(english_header, FALSE);
    gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), english_header);

    // Whisper.cpp English-only models
    const char *w_english[] = {"W tiny.en", "W base.en", "W small.en", "W medium.en", NULL};
    for (int i = 0; w_english[i]; i++) {
        GtkWidget *item = gtk_check_menu_item_new_with_label(w_english[i]);
        char *model_name = g_strdup(w_english[i]);
        g_object_set_data_full(G_OBJECT(item), "model", model_name, g_free);
        gtk_check_menu_item_set_active(GTK_CHECK_MENU_ITEM(item), strcmp(gui->selected_model, w_english[i]) == 0);
        g_signal_connect(item, "activate", G_CALLBACK(on_model_selected), gui);
        gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), item);
    }

    gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), gtk_separator_menu_item_new());

    // CTranslate2 English-only models
    const char *ct2_english[] = {"CT2 tiny.en", "CT2 base.en", "CT2 small.en", "CT2 medium.en", NULL};
    for (int i = 0; ct2_english[i]; i++) {
        GtkWidget *item = gtk_check_menu_item_new_with_label(ct2_english[i]);
        char *model_name = g_strdup(ct2_english[i]);
        g_object_set_data_full(G_OBJECT(item), "model", model_name, g_free);
        gtk_check_menu_item_set_active(GTK_CHECK_MENU_ITEM(item), strcmp(gui->selected_model, ct2_english[i]) == 0);
        g_signal_connect(item, "activate", G_CALLBACK(on_model_selected), gui);
        gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), item);
    }

    gtk_menu_item_set_submenu(GTK_MENU_ITEM(model_item), model_menu);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), model_item);
    gui->model_item = model_item;

    // Microphone submenu
    GtkWidget *mic_item = gtk_menu_item_new_with_label("Microphone: Detecting...");
    gtk_widget_set_sensitive(mic_item, FALSE);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), mic_item);
    gui->mic_label_item = mic_item;

    update_microphone_label(gui);
    g_timeout_add_seconds(3, refresh_microphone_label, gui);

    // Hotkey item - clickable to change hotkey
    GtkWidget *hotkey_item = gtk_menu_item_new_with_label("Hotkey: Scroll Lock");
    g_signal_connect(hotkey_item, "activate", G_CALLBACK(on_change_hotkey), gui);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), hotkey_item);
    gui->hotkey_item = hotkey_item;

    // Customize Transcription Settings (like macOS)
    GtkWidget *prompt_item = gtk_menu_item_new_with_label("Customize Transcription Settings...");
    g_signal_connect(prompt_item, "activate", G_CALLBACK(on_customize_prompt), gui);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), prompt_item);
    gui->prompt_item = prompt_item;

    gtk_menu_shell_append(GTK_MENU_SHELL(menu), gtk_separator_menu_item_new());

    // Logging toggle
    GtkWidget *logging_item = gtk_menu_item_new_with_label("Logging: On");
    g_signal_connect(logging_item, "activate", G_CALLBACK(on_toggle_logging), gui);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), logging_item);
    gui->logging_item = logging_item;

    // Show logs
    GtkWidget *show_logs_item = gtk_menu_item_new_with_label("Show Logs");
    g_signal_connect(show_logs_item, "activate", G_CALLBACK(on_show_logs), gui);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), show_logs_item);

    gtk_menu_shell_append(GTK_MENU_SHELL(menu), gtk_separator_menu_item_new());

    // About
    GtkWidget *about_item = gtk_menu_item_new_with_label("About Voice to Text Linux");
    g_signal_connect(about_item, "activate", G_CALLBACK(on_about), gui);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), about_item);

    gtk_menu_shell_append(GTK_MENU_SHELL(menu), gtk_separator_menu_item_new());

    // Quit
    GtkWidget *quit_item = gtk_menu_item_new_with_label("Quit");
    g_signal_connect(quit_item, "activate", G_CALLBACK(on_quit), gui);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), quit_item);

    gtk_widget_show_all(menu);

    app_indicator_set_menu(indicator, GTK_MENU(menu));
    gui->menu = menu;

    // Set initial model menu based on loaded language setting
    rebuild_model_menu(gui);

    // Update hotkey label based on loaded settings
    if (settings.hotkey_keycode != 0 && gui->keyboard && gui->keyboard->display) {
        const char *key_name = vtt_keyboard_get_key_name(gui->keyboard->display,
                                                         settings.hotkey_keycode);
        char hotkey_label[256];
        snprintf(hotkey_label, sizeof(hotkey_label), "Hotkey: %s", key_name);
        gtk_menu_item_set_label(GTK_MENU_ITEM(hotkey_item), hotkey_label);
        vtt_log("Hotkey label updated to: %s", key_name);
    }

    gui->initializing = false;  // Initialization complete, allow saving settings

    vtt_log("GUI initialized (AppIndicator)");
    return 0;
}

void vtt_gui_run(vtt_gui_t *gui) {
    gtk_main();
}

// Thread-safe data structures for GUI updates
typedef struct {
    vtt_gui_t *gui;
    char status[256];
} status_update_t;

typedef struct {
    vtt_gui_t *gui;
    char icon_status[32];
} icon_update_t;

// GLib idle callbacks (run on GTK main thread)
static gboolean set_status_idle(gpointer user_data) {
    status_update_t *data = (status_update_t *)user_data;

    if (data->gui->status_item) {
        char label[256];
        snprintf(label, sizeof(label), "Status: %s", data->status);
        gtk_menu_item_set_label(GTK_MENU_ITEM(data->gui->status_item), label);
    }

    free(data);
    return G_SOURCE_REMOVE;
}

static gboolean set_icon_idle(gpointer user_data) {
    icon_update_t *data = (icon_update_t *)user_data;

    if (data->gui && data->gui->indicator) {
        AppIndicator *indicator = (AppIndicator *)data->gui->indicator;

        // Map status strings to appropriate icon names
        if (strcmp(data->icon_status, "ready") == 0) {
            app_indicator_set_icon(indicator, "audio-input-microphone");
        } else if (strcmp(data->icon_status, "recording") == 0) {
            app_indicator_set_icon(indicator, "media-record");
        } else if (strcmp(data->icon_status, "processing") == 0) {
            app_indicator_set_icon(indicator, "emblem-synchronizing");
        } else {
            app_indicator_set_icon(indicator, "audio-input-microphone");
        }
    }

    free(data);
    return G_SOURCE_REMOVE;
}

void vtt_gui_set_status(vtt_gui_t *gui, const char *status) {
    status_update_t *data = malloc(sizeof(status_update_t));
    if (!data) return;

    data->gui = gui;
    snprintf(data->status, sizeof(data->status), "%s", status);

    g_idle_add(set_status_idle, data);
}

void vtt_gui_set_icon(vtt_gui_t *gui, const char *icon_status) {
    if (!gui || !gui->indicator) return;

    icon_update_t *data = malloc(sizeof(icon_update_t));
    if (!data) return;

    data->gui = gui;
    snprintf(data->icon_status, sizeof(data->icon_status), "%s", icon_status);

    g_idle_add(set_icon_idle, data);
}

void vtt_gui_cleanup(vtt_gui_t *gui) {
    // Save settings before cleanup
    if (gui->config_dir) {
        vtt_settings_t settings;
        populate_settings_snapshot(gui, &settings);
        vtt_settings_save(&settings, gui->config_dir);
    }

    if (gui->selected_model) {
        free(gui->selected_model);
    }
    if (gui->selected_language) {
        free(gui->selected_language);
    }
    if (gui->voice_prefix) {
        free(gui->voice_prefix);
    }
    if (gui->initial_prompt) {
        free(gui->initial_prompt);
    }
    if (gui->config_dir) {
        free(gui->config_dir);
    }
}

// Callbacks

static void on_quit(GtkMenuItem *item, gpointer user_data) {
    (void)item;
    (void)user_data;
    vtt_log("Quit requested");
    gtk_main_quit();
}

static void on_model_selected(GtkMenuItem *item, gpointer user_data) {
    vtt_gui_t *gui = (vtt_gui_t *)user_data;
    const char *model = (const char *)g_object_get_data(G_OBJECT(item), "model");

    if (model) {
        // Skip if we're still initializing (prevents overwriting loaded settings)
        if (gui->initializing) {
            return;
        }

        vtt_log("Model selected: %s", model);

        free(gui->selected_model);
        gui->selected_model = strdup(model);

        // Update menu label
        // NOTE: For menu items with submenus, we need to get the child label widget
        // and update it directly, as gtk_menu_item_set_label() doesn't always work
        char label[256];
        snprintf(label, sizeof(label), "Model: %s", model);

        GtkWidget *child = gtk_bin_get_child(GTK_BIN(gui->model_item));
        if (child && GTK_IS_LABEL(child)) {
            gtk_label_set_text(GTK_LABEL(child), label);
        } else {
            // Fallback to gtk_menu_item_set_label if child approach doesn't work
            gtk_menu_item_set_label(GTK_MENU_ITEM(gui->model_item), label);
        }

        // Save settings immediately
        vtt_settings_t settings;
        populate_settings_snapshot(gui, &settings);
        vtt_settings_save(&settings, gui->config_dir);

        // Update status
        vtt_gui_set_status(gui, "Model changed");
    }
}

static void on_language_selected(GtkMenuItem *item, gpointer user_data) {
    vtt_gui_t *gui = (vtt_gui_t *)user_data;
    const char *language = (const char *)g_object_get_data(G_OBJECT(item), "language");

    if (language) {
        vtt_log("Language selected: %s", language);

        free(gui->selected_language);
        gui->selected_language = strdup(language);

        // Update menu label
        const char *lang_display = strcmp(language, "en") == 0 ? "English only" : "Multilingual";
        char label[256];
        snprintf(label, sizeof(label), "Language: %s", lang_display);

        GtkWidget *child = gtk_bin_get_child(GTK_BIN(gui->language_item));
        if (child && GTK_IS_LABEL(child)) {
            gtk_label_set_text(GTK_LABEL(child), label);
        } else {
            gtk_menu_item_set_label(GTK_MENU_ITEM(gui->language_item), label);
        }

        // Rebuild model menu based on new language
        rebuild_model_menu(gui);

        // Save settings immediately
        vtt_settings_t settings;
        populate_settings_snapshot(gui, &settings);
        vtt_settings_save(&settings, gui->config_dir);

        // Update status
        vtt_gui_set_status(gui, "Language changed");
    }
}



static void on_show_logs(GtkMenuItem *item, gpointer user_data) {
    const char *log_path = vtt_log_get_path();

    if (log_path && *log_path) {
        char cmd[1024];
        snprintf(cmd, sizeof(cmd), "xdg-open '%s' &", log_path);
        system(cmd);
        vtt_log("Opening log file: %s", log_path);
    }
}

static void on_toggle_logging(GtkMenuItem *item, gpointer user_data) {
    vtt_gui_t *gui = (vtt_gui_t *)user_data;

    gui->logging_enabled = !gui->logging_enabled;
    vtt_log_set_enabled(gui->logging_enabled);

    const char *label = gui->logging_enabled ? "Logging: On" : "Logging: Off";
    gtk_menu_item_set_label(GTK_MENU_ITEM(gui->logging_item), label);

    vtt_log("Logging %s", gui->logging_enabled ? "enabled" : "disabled");
}

static void on_about(GtkMenuItem *item, gpointer user_data) {
    (void)item;
    (void)user_data;

    GtkWidget *dialog = gtk_message_dialog_new(
        NULL,
        GTK_DIALOG_MODAL,
        GTK_MESSAGE_INFO,
        GTK_BUTTONS_OK,
        "Voice to Text Linux\n\n"
        "Version 1.0\n"
        "Voice-to-text transcription for Linux\n\n"
        "Press Scroll Lock to start/stop recording"
    );

    gtk_dialog_run(GTK_DIALOG(dialog));
    gtk_widget_destroy(dialog);
}

// Prompt dialog callbacks

static void on_prompt_text_changed(GtkTextBuffer *buffer, gpointer user_data) {
    GtkWidget *counter = GTK_WIDGET(user_data);
    gint count = gtk_text_buffer_get_char_count(buffer);

    // Limit to 240 characters
    if (count > 240) {
        GtkTextIter start, end;
        gtk_text_buffer_get_iter_at_offset(buffer, &start, 0);
        gtk_text_buffer_get_iter_at_offset(buffer, &end, 240);
        gchar *text = gtk_text_buffer_get_text(buffer, &start, &end, FALSE);
        gtk_text_buffer_set_text(buffer, text, -1);
        g_free(text);
        count = 240;
        gdk_display_beep(gdk_display_get_default());
    }

    char text[64];
    snprintf(text, sizeof(text), "%d / 240 characters", count);

    // Color code: red at 230+, orange at 200+
    if (count >= 230) {
        char markup[128];
        snprintf(markup, sizeof(markup), "<span color='red'>%s</span>", text);
        gtk_label_set_markup(GTK_LABEL(counter), markup);
    } else if (count >= 200) {
        char markup[128];
        snprintf(markup, sizeof(markup), "<span color='orange'>%s</span>", text);
        gtk_label_set_markup(GTK_LABEL(counter), markup);
    } else {
        gtk_label_set_text(GTK_LABEL(counter), text);
    }
}

static void on_prompt_reset(GtkButton *button, gpointer user_data) {
    (void)button;
    prompt_dialog_data_t *data = (prompt_dialog_data_t *)user_data;

    gtk_entry_set_text(GTK_ENTRY(data->prefix_entry), "[Voice] ");
    gtk_text_buffer_set_text(data->text_buffer,
        "Male British English speaker. Programming, business and technical terminology with frequent acronyms and spelled letters.",
        -1);

    if (data->newline_toggle) {
        gtk_toggle_button_set_active(GTK_TOGGLE_BUTTON(data->newline_toggle), TRUE);
    }
}

static void on_prompt_save(GtkButton *button, gpointer user_data) {
    (void)button;
    prompt_dialog_data_t *data = (prompt_dialog_data_t *)user_data;

    // Save voice prefix
    const gchar *new_prefix = gtk_entry_get_text(GTK_ENTRY(data->prefix_entry));
    if (data->gui->voice_prefix) free(data->gui->voice_prefix);
    data->gui->voice_prefix = strdup(new_prefix);
    vtt_log("Updated voice prefix: %s", new_prefix);

    // Save initial prompt
    GtkTextIter start, end;
    gtk_text_buffer_get_bounds(data->text_buffer, &start, &end);
    gchar *new_prompt = gtk_text_buffer_get_text(data->text_buffer, &start, &end, FALSE);
    if (data->gui->initial_prompt) free(data->gui->initial_prompt);
    data->gui->initial_prompt = strdup(new_prompt);
    vtt_log("Updated initial prompt: %s", new_prompt);
    g_free(new_prompt);

    if (data->newline_toggle) {
        data->gui->append_newline = gtk_toggle_button_get_active(GTK_TOGGLE_BUTTON(data->newline_toggle));
        vtt_log("Updated newline setting: %s", data->gui->append_newline ? "on" : "off");
    }

    // Save settings to disk immediately
    vtt_settings_t settings;
    populate_settings_snapshot(data->gui, &settings);
    vtt_settings_save(&settings, data->gui->config_dir);

    // Close dialog
    gtk_widget_destroy(data->dialog);
}

static void on_prompt_dialog_destroy(GtkWidget *widget, gpointer user_data) {
    (void)widget;
    prompt_dialog_data_t *data = (prompt_dialog_data_t *)user_data;
    data->gui->prompt_dialog = NULL;
    free(data);
}

static void on_customize_prompt(GtkMenuItem *item, gpointer user_data) {
    (void)item;
    vtt_gui_t *gui = (vtt_gui_t *)user_data;

    // If dialog already open, just bring it to front
    if (gui->prompt_dialog && GTK_IS_WIDGET(gui->prompt_dialog)) {
        gtk_window_present(GTK_WINDOW(gui->prompt_dialog));
        return;
    }

    // Create dialog data
    prompt_dialog_data_t *data = calloc(1, sizeof(prompt_dialog_data_t));
    data->gui = gui;

    // Create dialog
    GtkWidget *dialog = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(dialog), "Customize Transcription Settings");
    gtk_window_set_default_size(GTK_WINDOW(dialog), 500, 340);
    gtk_window_set_resizable(GTK_WINDOW(dialog), FALSE);
    gtk_window_set_position(GTK_WINDOW(dialog), GTK_WIN_POS_CENTER);
    gtk_container_set_border_width(GTK_CONTAINER(dialog), 20);

    data->dialog = dialog;

    // Main vertical box
    GtkWidget *vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_container_add(GTK_CONTAINER(dialog), vbox);

    // === VOICE PREFIX SECTION ===
    GtkWidget *prefix_label = gtk_label_new(NULL);
    gtk_label_set_markup(GTK_LABEL(prefix_label), "<b>Voice Prefix (prepended to every transcription):</b>");
    gtk_widget_set_halign(prefix_label, GTK_ALIGN_START);
    gtk_box_pack_start(GTK_BOX(vbox), prefix_label, FALSE, FALSE, 0);

    GtkWidget *prefix_entry = gtk_entry_new();
    gtk_entry_set_text(GTK_ENTRY(prefix_entry), gui->voice_prefix ? gui->voice_prefix : "");
    gtk_entry_set_placeholder_text(GTK_ENTRY(prefix_entry), "e.g., [voice] ");
    gtk_box_pack_start(GTK_BOX(vbox), prefix_entry, FALSE, FALSE, 0);

    data->prefix_entry = prefix_entry;

    // Spacer
    GtkWidget *spacer1 = gtk_label_new("");
    gtk_box_pack_start(GTK_BOX(vbox), spacer1, FALSE, FALSE, 0);

    // === INITIAL PROMPT SECTION ===
    GtkWidget *prompt_label = gtk_label_new(NULL);
    gtk_label_set_markup(GTK_LABEL(prompt_label), "<b>Initial Prompt (helps Whisper recognize your voice, max 240 chars):</b>");
    gtk_widget_set_halign(prompt_label, GTK_ALIGN_START);
    gtk_box_pack_start(GTK_BOX(vbox), prompt_label, FALSE, FALSE, 0);

    // Scrolled window for text view
    GtkWidget *scrolled = gtk_scrolled_window_new(NULL, NULL);
    gtk_scrolled_window_set_policy(GTK_SCROLLED_WINDOW(scrolled), GTK_POLICY_AUTOMATIC, GTK_POLICY_AUTOMATIC);
    gtk_widget_set_size_request(scrolled, -1, 70);
    gtk_box_pack_start(GTK_BOX(vbox), scrolled, TRUE, TRUE, 0);

    GtkWidget *text_view = gtk_text_view_new();
    gtk_text_view_set_wrap_mode(GTK_TEXT_VIEW(text_view), GTK_WRAP_WORD_CHAR);
    GtkTextBuffer *buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(text_view));
    gtk_text_buffer_set_text(buffer, gui->initial_prompt ? gui->initial_prompt : "", -1);
    gtk_container_add(GTK_CONTAINER(scrolled), text_view);

    data->text_buffer = buffer;

    // Character counter
    GtkWidget *char_counter = gtk_label_new(NULL);
    gint char_count = gtk_text_buffer_get_char_count(buffer);
    char counter_text[64];
    snprintf(counter_text, sizeof(counter_text), "%d / 240 characters", char_count);
    gtk_label_set_text(GTK_LABEL(char_counter), counter_text);
    gtk_widget_set_halign(char_counter, GTK_ALIGN_END);
    gtk_box_pack_start(GTK_BOX(vbox), char_counter, FALSE, FALSE, 0);

    // Update counter on text change
    g_signal_connect(buffer, "changed", G_CALLBACK(on_prompt_text_changed), char_counter);

    // Spacer
    GtkWidget *spacer2 = gtk_label_new("");
    gtk_box_pack_start(GTK_BOX(vbox), spacer2, FALSE, FALSE, 0);

    // Newline toggle
    GtkWidget *newline_toggle = gtk_check_button_new_with_label("Insert newline between transcriptions");
    gtk_toggle_button_set_active(GTK_TOGGLE_BUTTON(newline_toggle), gui->append_newline);
    gtk_box_pack_start(GTK_BOX(vbox), newline_toggle, FALSE, FALSE, 0);
    data->newline_toggle = newline_toggle;

    // === BUTTON ROW ===
    GtkWidget *button_box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 10);
    gtk_box_pack_start(GTK_BOX(vbox), button_box, FALSE, FALSE, 0);

    // Reset button
    GtkWidget *reset_button = gtk_button_new_with_label("Reset Default");
    gtk_widget_set_size_request(reset_button, 120, -1);
    g_signal_connect(reset_button, "clicked", G_CALLBACK(on_prompt_reset), data);
    gtk_box_pack_start(GTK_BOX(button_box), reset_button, FALSE, FALSE, 0);

    // Spacer to push buttons right
    GtkWidget *spacer3 = gtk_label_new("");
    gtk_box_pack_start(GTK_BOX(button_box), spacer3, TRUE, TRUE, 0);

    // Cancel button
    GtkWidget *cancel_button = gtk_button_new_with_label("Cancel");
    gtk_widget_set_size_request(cancel_button, 80, -1);
    g_signal_connect_swapped(cancel_button, "clicked", G_CALLBACK(gtk_widget_destroy), dialog);
    gtk_box_pack_start(GTK_BOX(button_box), cancel_button, FALSE, FALSE, 0);

    // Save button
    GtkWidget *save_button = gtk_button_new_with_label("Save");
    gtk_widget_set_size_request(save_button, 80, -1);
    g_signal_connect(save_button, "clicked", G_CALLBACK(on_prompt_save), data);
    gtk_box_pack_start(GTK_BOX(button_box), save_button, FALSE, FALSE, 0);

    // Store dialog reference
    gui->prompt_dialog = dialog;

    // Clean up reference when closed
    g_signal_connect(dialog, "destroy", G_CALLBACK(on_prompt_dialog_destroy), data);

    gtk_widget_show_all(dialog);
}

// Hotkey capture dialog state
typedef struct {
    GtkWidget *dialog;
    GtkWidget *label;
    vtt_gui_t *gui;
    vtt_keyboard_t *keyboard;
    int captured_keycode;
    bool key_pressed;
} hotkey_dialog_t;

static gboolean on_hotkey_key_press(GtkWidget *widget, GdkEventKey *event, gpointer user_data) {
    (void)widget;
    hotkey_dialog_t *data = (hotkey_dialog_t *)user_data;

    // Get the X11 keycode from GDK event
    int keycode = event->hardware_keycode;

    // Log the captured key for debugging
    vtt_log("Hotkey dialog captured: keycode=%d, keyval=%u, state=%u",
            keycode, event->keyval, event->state);

    // Validate X11 keycode range (8-255)
    if (keycode < 8 || keycode > 255) {
        vtt_log("Invalid keycode captured: %d (must be 8-255), ignoring", keycode);
        char text[256];
        snprintf(text, sizeof(text),
                 "Press and hold the key you want to use...\n\n"
                 "Invalid key detected (keycode %d)!\n\nTry a different key.", keycode);
        gtk_label_set_text(GTK_LABEL(data->label), text);
        data->captured_keycode = 0;
        data->key_pressed = false;
        return TRUE;
    }

    data->captured_keycode = keycode;
    data->key_pressed = true;

    // Prefer the keyboard's X11 display if available
    Display *x_display = NULL;
    if (data->keyboard && data->keyboard->display) {
        x_display = (Display *)data->keyboard->display;
    } else {
        GdkDisplay *gdk_display = gdk_display_get_default();
        if (gdk_display) {
            x_display = GDK_DISPLAY_XDISPLAY(gdk_display);
        }
    }

    const char *key_name = x_display
        ? vtt_keyboard_get_key_name(x_display, data->captured_keycode)
        : "Unknown";

    // Update label
    char text[256];
    snprintf(text, sizeof(text),
             "Press and hold the key you want to use...\n\n"
             "Key detected: %s\n\nRelease the key to confirm.", key_name);
    gtk_label_set_text(GTK_LABEL(data->label), text);

    return TRUE;  // Stop event propagation
}

static gboolean on_hotkey_key_release(GtkWidget *widget, GdkEventKey *event, gpointer user_data) {
    (void)widget;
    (void)event;
    hotkey_dialog_t *data = (hotkey_dialog_t *)user_data;

    // Only accept release if we captured a key press first
    if (!data->key_pressed) {
        return TRUE;
    }

    if (!data->keyboard) {
        gtk_dialog_response(GTK_DIALOG(data->dialog), GTK_RESPONSE_CANCEL);
        return TRUE;
    }

    // Validate stored keycode before applying
    if (data->captured_keycode < 8 || data->captured_keycode > 255) {
        vtt_log("Hotkey release received invalid keycode %d, ignoring", data->captured_keycode);
        char text[256];
        snprintf(text, sizeof(text),
                 "Press and hold the key you want to use...\n\n"
                 "Invalid key detected. Please try again.");
        gtk_label_set_text(GTK_LABEL(data->label), text);
        data->captured_keycode = 0;
        data->key_pressed = false;
        return TRUE;
    }

    // Choose display for key name lookup
    Display *x_display = data->keyboard->display
        ? (Display *)data->keyboard->display
        : GDK_DISPLAY_XDISPLAY(gdk_display_get_default());

    // Get key name for logging and display
    const char *key_name = vtt_keyboard_get_key_name(x_display, data->captured_keycode);
    vtt_log("Hotkey changed to: %s (keycode %d)", key_name, data->captured_keycode);

    // Update keyboard monitoring
    if (vtt_keyboard_set_hotkey(data->keyboard, data->captured_keycode) != 0) {
        vtt_log("Failed to update keyboard hotkey to keycode %d", data->captured_keycode);
        char text[256];
        snprintf(text, sizeof(text),
                 "Failed to set hotkey to %s.\n\nPlease try a different key.", key_name);
        gtk_label_set_text(GTK_LABEL(data->label), text);
        data->captured_keycode = 0;
        data->key_pressed = false;
        return TRUE;
    }

    // Update GUI menu item
    char label[256];
    snprintf(label, sizeof(label), "Hotkey: %s", key_name);
    gtk_menu_item_set_label(GTK_MENU_ITEM(data->gui->hotkey_item), label);

    // Save to settings
    vtt_settings_t settings;
    populate_settings_snapshot(data->gui, &settings);
    settings.hotkey_keycode = data->captured_keycode;
    vtt_settings_save(&settings, data->gui->config_dir);

    // Close dialog - use gtk_dialog_response to exit gtk_dialog_run() cleanly
    gtk_dialog_response(GTK_DIALOG(data->dialog), GTK_RESPONSE_OK);
    return TRUE;
}

static void on_change_hotkey(GtkMenuItem *item, gpointer user_data) {
    (void)item;
    vtt_gui_t *gui = (vtt_gui_t *)user_data;
    if (!gui) {
        return;
    }

    if (!gui->keyboard) {
        vtt_log("Hotkey customization requested but keyboard subsystem is unavailable");
        GtkWidget *dialog = gtk_message_dialog_new(
            NULL,
            GTK_DIALOG_MODAL,
            GTK_MESSAGE_WARNING,
            GTK_BUTTONS_OK,
            "Keyboard controls are not available.\n\n"
            "Unable to customize the hotkey."
        );
        gtk_dialog_run(GTK_DIALOG(dialog));
        gtk_widget_destroy(dialog);
        return;
    }

    // Create dialog data
    hotkey_dialog_t *data = calloc(1, sizeof(hotkey_dialog_t));
    if (!data) {
        vtt_log("Failed to allocate hotkey dialog data");
        return;
    }

    data->gui = gui;
    data->keyboard = gui->keyboard;

    // Create dialog
    GtkWidget *dialog = gtk_dialog_new();
    gtk_window_set_title(GTK_WINDOW(dialog), "Customize Hotkey");
    gtk_window_set_default_size(GTK_WINDOW(dialog), 400, 180);
    gtk_window_set_resizable(GTK_WINDOW(dialog), FALSE);
    gtk_window_set_position(GTK_WINDOW(dialog), GTK_WIN_POS_CENTER);
    gtk_window_set_modal(GTK_WINDOW(dialog), TRUE);

    data->dialog = dialog;

    // Get content area
    GtkWidget *content = gtk_dialog_get_content_area(GTK_DIALOG(dialog));
    gtk_container_set_border_width(GTK_CONTAINER(content), 20);

    // Label
    GtkWidget *label = gtk_label_new("Press and hold the key you want to use...\n\nWaiting for key press...");
    gtk_label_set_justify(GTK_LABEL(label), GTK_JUSTIFY_CENTER);
    gtk_box_pack_start(GTK_BOX(content), label, TRUE, TRUE, 10);

    data->label = label;

    // Add cancel button
    gtk_dialog_add_button(GTK_DIALOG(dialog), "Cancel", GTK_RESPONSE_CANCEL);

    // Connect key events
    g_signal_connect(dialog, "key-press-event", G_CALLBACK(on_hotkey_key_press), data);
    g_signal_connect(dialog, "key-release-event", G_CALLBACK(on_hotkey_key_release), data);

    // Free data on close
    g_signal_connect_swapped(dialog, "destroy", G_CALLBACK(free), data);

    gtk_widget_show_all(dialog);

    // Run dialog (blocks until closed)
    gtk_dialog_run(GTK_DIALOG(dialog));
    gtk_widget_destroy(dialog);
}


// Rebuild model menu based on language selection
// Menu always shows same model sizes - backend picks .en version or sets language
static void rebuild_model_menu(vtt_gui_t *gui) {
    if (!gui || !gui->model_item) return;

    bool is_english = (strcmp(gui->selected_language, "en") == 0);

    // Get model submenu
    GtkWidget *model_menu = gtk_menu_item_get_submenu(GTK_MENU_ITEM(gui->model_item));
    if (!model_menu) return;

    // Clear existing menu items
    GList *children = gtk_container_get_children(GTK_CONTAINER(model_menu));
    for (GList *iter = children; iter != NULL; iter = g_list_next(iter)) {
        gtk_widget_destroy(GTK_WIDGET(iter->data));
    }
    g_list_free(children);

    // Strip .en suffix from current model if switching to multilingual
    if (!is_english && strstr(gui->selected_model, ".en") != NULL) {
        char new_model[256];
        strncpy(new_model, gui->selected_model, sizeof(new_model) - 1);
        new_model[sizeof(new_model) - 1] = '\0';

        char *en_suffix = strstr(new_model, ".en");
        if (en_suffix) {
            *en_suffix = '\0';
            vtt_log("Auto-switching from %s to %s (multilingual mode)", gui->selected_model, new_model);
            free(gui->selected_model);
            gui->selected_model = strdup(new_model);

            // Update menu label
            char label[256];
            snprintf(label, sizeof(label), "Model: %.200s", new_model);
            GtkWidget *child = gtk_bin_get_child(GTK_BIN(gui->model_item));
            if (child && GTK_IS_LABEL(child)) {
                gtk_label_set_text(GTK_LABEL(child), label);
            }
        }
    }

    // Create radio button group for models (only one can be selected)
    GSList *model_group = NULL;

    // Whisper.cpp models
    const char *w_models[] = {"W tiny", "W base", "W small", "W medium", "W large", NULL};
    for (int i = 0; w_models[i]; i++) {
        GtkWidget *item = gtk_radio_menu_item_new_with_label(model_group, w_models[i]);
        model_group = gtk_radio_menu_item_get_group(GTK_RADIO_MENU_ITEM(item));
        g_object_set_data_full(G_OBJECT(item), "model", g_strdup(w_models[i]), g_free);

        // Check if this model is selected (compare base names, ignoring .en)
        char sel_base[256];
        strncpy(sel_base, gui->selected_model, sizeof(sel_base) - 1);
        sel_base[sizeof(sel_base) - 1] = '\0';
        char *en = strstr(sel_base, ".en");
        if (en) *en = '\0';
        bool is_selected = (strcmp(w_models[i], sel_base) == 0);

        gtk_check_menu_item_set_active(GTK_CHECK_MENU_ITEM(item), is_selected);

        // Disable tiny & base for multilingual (poor quality)
        bool is_tiny_or_base = (strstr(w_models[i], "tiny") != NULL || strstr(w_models[i], "base") != NULL);
        gtk_widget_set_sensitive(item, is_english || !is_tiny_or_base);

        g_signal_connect(item, "activate", G_CALLBACK(on_model_selected), gui);
        gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), item);
    }

    gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), gtk_separator_menu_item_new());

    // CTranslate2 models (continue the same radio group)
    const char *ct2_models[] = {"CT2 tiny", "CT2 base", "CT2 small", "CT2 medium", "CT2 large-v3", NULL};
    for (int i = 0; ct2_models[i]; i++) {
        GtkWidget *item = gtk_radio_menu_item_new_with_label(model_group, ct2_models[i]);
        model_group = gtk_radio_menu_item_get_group(GTK_RADIO_MENU_ITEM(item));
        g_object_set_data_full(G_OBJECT(item), "model", g_strdup(ct2_models[i]), g_free);

        // Check if this model is selected (compare base names, ignoring .en)
        char sel_base2[256];
        strncpy(sel_base2, gui->selected_model, sizeof(sel_base2) - 1);
        sel_base2[sizeof(sel_base2) - 1] = '\0';
        char *en2 = strstr(sel_base2, ".en");
        if (en2) *en2 = '\0';
        bool is_selected2 = (strcmp(ct2_models[i], sel_base2) == 0);

        gtk_check_menu_item_set_active(GTK_CHECK_MENU_ITEM(item), is_selected2);

        // Disable tiny & base for multilingual (poor quality)
        bool is_tiny_or_base = (strstr(ct2_models[i], "tiny") != NULL || strstr(ct2_models[i], "base") != NULL);
        gtk_widget_set_sensitive(item, is_english || !is_tiny_or_base);

        g_signal_connect(item, "activate", G_CALLBACK(on_model_selected), gui);
        gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), item);
    }

    gtk_widget_show_all(model_menu);
}

// Periodic device check for hot-plug detection (called every 3 seconds)
