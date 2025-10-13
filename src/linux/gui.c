#include "gui.h"
#include "../common/logging.h"
#include <gtk/gtk.h>
#include <libayatana-appindicator/app-indicator.h>
#include <stdlib.h>
#include <string.h>

// Callbacks
static void on_quit(GtkMenuItem *item, gpointer user_data);
static void on_model_selected(GtkMenuItem *item, gpointer user_data);
static void on_show_logs(GtkMenuItem *item, gpointer user_data);
static void on_toggle_logging(GtkMenuItem *item, gpointer user_data);
static void on_about(GtkMenuItem *item, gpointer user_data);

int vtt_gui_init(vtt_gui_t *gui, void *app) {
    memset(gui, 0, sizeof(vtt_gui_t));
    gui->user_data = app;
    gui->selected_model = strdup("large-v3");
    gui->logging_enabled = true;

    // Initialize GTK
    gtk_init(NULL, NULL);

    // Create indicator
    AppIndicator *indicator = app_indicator_new(
        "voice-to-text",
        "audio-input-microphone",
        APP_INDICATOR_CATEGORY_APPLICATION_STATUS
    );

    app_indicator_set_status(indicator, APP_INDICATOR_STATUS_ACTIVE);
    app_indicator_set_title(indicator, "VTT");

    gui->indicator = indicator;

    // Create menu
    GtkWidget *menu = gtk_menu_new();

    // Status item (non-clickable)
    GtkWidget *status_item = gtk_menu_item_new_with_label("Status: Initializing...");
    gtk_widget_set_sensitive(status_item, FALSE);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), status_item);
    gui->status_item = status_item;

    gtk_menu_shell_append(GTK_MENU_SHELL(menu), gtk_separator_menu_item_new());

    // Model submenu
    GtkWidget *model_item = gtk_menu_item_new_with_label("Model: large-v3");
    GtkWidget *model_menu = gtk_menu_new();

    const char *models[] = {"tiny", "base", "small", "medium", "large-v3", NULL};
    for (int i = 0; models[i]; i++) {
        char label[64];
        snprintf(label, sizeof(label), "CT2 %s", models[i]);

        GtkWidget *item = gtk_menu_item_new_with_label(label);
        g_object_set_data_full(G_OBJECT(item), "model", g_strdup(models[i]), g_free);
        g_signal_connect(item, "activate", G_CALLBACK(on_model_selected), gui);
        gtk_menu_shell_append(GTK_MENU_SHELL(model_menu), item);
    }

    gtk_menu_item_set_submenu(GTK_MENU_ITEM(model_item), model_menu);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), model_item);
    gui->model_item = model_item;

    // Microphone item (placeholder)
    GtkWidget *mic_item = gtk_menu_item_new_with_label("Microphone: Default");
    gtk_widget_set_sensitive(mic_item, FALSE);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), mic_item);
    gui->mic_item = mic_item;

    // Hotkey item (placeholder)
    GtkWidget *hotkey_item = gtk_menu_item_new_with_label("Hotkey: Scroll Lock");
    gtk_widget_set_sensitive(hotkey_item, FALSE);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), hotkey_item);
    gui->hotkey_item = hotkey_item;

    // Prompt item (placeholder)
    GtkWidget *prompt_item = gtk_menu_item_new_with_label("Prompt: Male British English...");
    gtk_widget_set_sensitive(prompt_item, FALSE);
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
    GtkWidget *about_item = gtk_menu_item_new_with_label("About Voice to Text");
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

    vtt_log("GUI initialized (AppIndicator)");
    return 0;
}

void vtt_gui_run(vtt_gui_t *gui) {
    gtk_main();
}

void vtt_gui_set_status(vtt_gui_t *gui, const char *status) {
    if (gui->status_item) {
        char label[256];
        snprintf(label, sizeof(label), "Status: %s", status);
        gtk_menu_item_set_label(GTK_MENU_ITEM(gui->status_item), label);
    }
}

void vtt_gui_set_icon(vtt_gui_t *gui, const char *icon_status) {
    // AppIndicator doesn't support dynamic icon text like macOS
    // Could use different icon names for different states
}

void vtt_gui_cleanup(vtt_gui_t *gui) {
    if (gui->selected_model) {
        free(gui->selected_model);
    }
}

// Callbacks

static void on_quit(GtkMenuItem *item, gpointer user_data) {
    vtt_log("Quit requested");
    gtk_main_quit();
    exit(0);
}

static void on_model_selected(GtkMenuItem *item, gpointer user_data) {
    vtt_gui_t *gui = (vtt_gui_t *)user_data;
    const char *model = (const char *)g_object_get_data(G_OBJECT(item), "model");

    if (model) {
        vtt_log("Model selected: %s", model);

        free(gui->selected_model);
        gui->selected_model = strdup(model);

        // Update menu label
        char label[256];
        snprintf(label, sizeof(label), "Model: %s", model);
        gtk_menu_item_set_label(GTK_MENU_ITEM(gui->model_item), label);

        // Update status
        vtt_gui_set_status(gui, "Model changed, restart to apply");
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
    GtkWidget *dialog = gtk_message_dialog_new(
        NULL,
        GTK_DIALOG_MODAL,
        GTK_MESSAGE_INFO,
        GTK_BUTTONS_OK,
        "Voice to Text\n\n"
        "Version 1.0\n"
        "Cross-platform voice-to-text transcription\n\n"
        "Press Scroll Lock to start/stop recording"
    );

    gtk_dialog_run(GTK_DIALOG(dialog));
    gtk_widget_destroy(dialog);
}
