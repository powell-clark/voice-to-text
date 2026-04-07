#ifndef VTT_KEYBOARD_LINUX_H
#define VTT_KEYBOARD_LINUX_H

#include <stdbool.h>
#include <stdatomic.h>

typedef enum {
    VTT_KEY_DOWN,
    VTT_KEY_UP
} vtt_key_event_t;

typedef void (*vtt_keyboard_callback_t)(vtt_key_event_t event);

typedef struct {
    void *display;
    atomic_bool running;    // Shared between monitor thread and main thread
    vtt_keyboard_callback_t callback;
    int scroll_lock_keycode;
} vtt_keyboard_t;

// Initialize keyboard hook
int vtt_keyboard_init(vtt_keyboard_t *keyboard, vtt_keyboard_callback_t callback);

// Set custom hotkey (0 = use default Scroll Lock)
int vtt_keyboard_set_hotkey(vtt_keyboard_t *keyboard, int keycode);

// Get current hotkey keycode
int vtt_keyboard_get_hotkey(vtt_keyboard_t *keyboard);

// Get human-readable name for a keycode (returns static string)
const char* vtt_keyboard_get_key_name(void *display, int keycode);

// Start monitoring (runs in background thread)
int vtt_keyboard_start(vtt_keyboard_t *keyboard);

// Stop monitoring
void vtt_keyboard_stop(vtt_keyboard_t *keyboard);

// Cleanup
void vtt_keyboard_cleanup(vtt_keyboard_t *keyboard);

#endif // VTT_KEYBOARD_LINUX_H
