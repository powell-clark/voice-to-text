#ifndef VTT_KEYBOARD_WAYLAND_H
#define VTT_KEYBOARD_WAYLAND_H

#include "keyboard.h"  // Reuse types from X11 version
#include <stdbool.h>

// Wayland-specific keyboard context
typedef struct {
    void *wl_display;
    void *wl_registry;
    void *wl_seat;
    void *dbus_connection;
    bool running;
    vtt_keyboard_callback_t callback;
    int hotkey_keycode;
    char *compositor;
} vtt_keyboard_wayland_t;

// Initialize Wayland keyboard hook
int vtt_keyboard_wayland_init(vtt_keyboard_wayland_t *keyboard, vtt_keyboard_callback_t callback);

// Set custom hotkey
int vtt_keyboard_wayland_set_hotkey(vtt_keyboard_wayland_t *keyboard, int keycode);

// Get current hotkey
int vtt_keyboard_wayland_get_hotkey(vtt_keyboard_wayland_t *keyboard);

// Start monitoring
int vtt_keyboard_wayland_start(vtt_keyboard_wayland_t *keyboard);

// Stop monitoring
void vtt_keyboard_wayland_stop(vtt_keyboard_wayland_t *keyboard);

// Cleanup
void vtt_keyboard_wayland_cleanup(vtt_keyboard_wayland_t *keyboard);

#endif // VTT_KEYBOARD_WAYLAND_H
