#ifndef VTT_SETTINGS_H
#define VTT_SETTINGS_H

#include <stdbool.h>

typedef enum {
    NEWLINE_PLAIN_RETURN = 0,   // Plain Return key (may send messages in chat apps)
    NEWLINE_SHIFT_RETURN = 1    // Shift+Return (safer, won't send messages)
} vtt_newline_type_t;

typedef struct {
    char *selected_model;
    char *selected_language;  // Language code (e.g., "en", "es", "fr", "auto")
    char *voice_prefix;
    char *initial_prompt;
    int selected_device_index;
    int hotkey_keycode;  // X11 keycode for the hotkey (0 = use default)
    bool append_newline;
    vtt_newline_type_t newline_type;
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
