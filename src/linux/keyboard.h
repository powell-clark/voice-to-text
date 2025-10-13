#ifndef VTT_KEYBOARD_LINUX_H
#define VTT_KEYBOARD_LINUX_H

#include <stdbool.h>

typedef enum {
    VTT_KEY_DOWN,
    VTT_KEY_UP
} vtt_key_event_t;

typedef void (*vtt_keyboard_callback_t)(vtt_key_event_t event);

typedef struct {
    void *display;
    bool running;
    vtt_keyboard_callback_t callback;
    int scroll_lock_keycode;
} vtt_keyboard_t;

// Initialize keyboard hook
int vtt_keyboard_init(vtt_keyboard_t *keyboard, vtt_keyboard_callback_t callback);

// Start monitoring (runs in background thread)
int vtt_keyboard_start(vtt_keyboard_t *keyboard);

// Stop monitoring
void vtt_keyboard_stop(vtt_keyboard_t *keyboard);

// Cleanup
void vtt_keyboard_cleanup(vtt_keyboard_t *keyboard);

#endif // VTT_KEYBOARD_LINUX_H
