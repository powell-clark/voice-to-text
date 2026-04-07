#ifndef VTT_GUI_LINUX_H
#define VTT_GUI_LINUX_H

#include <stdbool.h>

#include "audio.h"
#include "keyboard.h"
#include "../common/settings.h"

typedef void (*vtt_model_callback_t)(const char *model, void *user_data);

typedef struct {
    void *indicator;
    void *menu;
    void *status_item;
    void *model_item;
    void *mic_label_item;
    void *hotkey_item;
    void *prompt_item;
    void *logging_item;
    void *language_item;
    void *recent_menu_item;
    void *prompt_dialog;
    vtt_model_callback_t model_callback;
    vtt_audio_t *audio;
    vtt_keyboard_t *keyboard;
    bool *recording_flag;
    char *selected_model;
    char *selected_language;
    char *voice_prefix;
    char *initial_prompt;
    char *config_dir;
    bool logging_enabled;
    bool initializing;  // Prevent saving settings during GUI initialization
    bool append_newline;
    vtt_newline_type_t newline_type;
} vtt_gui_t;

// Initialize GUI
int vtt_gui_init(vtt_gui_t *gui,
                 vtt_audio_t *audio,
                 vtt_keyboard_t *keyboard,
                 bool *recording_flag,
                 const char *config_dir);

// Run GUI main loop
void vtt_gui_run(vtt_gui_t *gui);

// Update status text
void vtt_gui_set_status(vtt_gui_t *gui, const char *status);

// Update icon status
void vtt_gui_set_icon(vtt_gui_t *gui, const char *icon_status);

// Cleanup
void vtt_gui_cleanup(vtt_gui_t *gui);

#endif // VTT_GUI_LINUX_H
