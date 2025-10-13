#ifndef VTT_GUI_LINUX_H
#define VTT_GUI_LINUX_H

#include <stdbool.h>

typedef void (*vtt_model_callback_t)(const char *model, void *user_data);

typedef struct {
    void *indicator;
    void *menu;
    void *status_item;
    void *model_item;
    void *mic_item;
    void *hotkey_item;
    void *prompt_item;
    void *logging_item;
    vtt_model_callback_t model_callback;
    void *user_data;
    char *selected_model;
    bool logging_enabled;
} vtt_gui_t;

// Initialize GUI
int vtt_gui_init(vtt_gui_t *gui, void *app);

// Run GUI main loop
void vtt_gui_run(vtt_gui_t *gui);

// Update status text
void vtt_gui_set_status(vtt_gui_t *gui, const char *status);

// Update icon status
void vtt_gui_set_icon(vtt_gui_t *gui, const char *icon_status);

// Cleanup
void vtt_gui_cleanup(vtt_gui_t *gui);

#endif // VTT_GUI_LINUX_H
