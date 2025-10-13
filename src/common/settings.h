#ifndef VTT_SETTINGS_H
#define VTT_SETTINGS_H

#include <stdbool.h>

typedef struct {
    char *selected_model;
    char *selected_language;  // Language code (e.g., "en", "es", "fr", "auto")
    char *voice_prefix;
    char *initial_prompt;
    int selected_device_index;
    int hotkey_keycode;  // X11 keycode for the hotkey (0 = use default)
} vtt_settings_t;

// Initialize settings with defaults
void vtt_settings_init(vtt_settings_t *settings);

// Load settings from file (returns 0 on success, -1 if file doesn't exist or parse error)
int vtt_settings_load(vtt_settings_t *settings, const char *config_dir);

// Save settings to file (returns 0 on success, -1 on error)
int vtt_settings_save(const vtt_settings_t *settings, const char *config_dir);

// Free settings memory
void vtt_settings_cleanup(vtt_settings_t *settings);

#endif // VTT_SETTINGS_H
