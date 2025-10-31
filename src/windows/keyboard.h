#ifndef VTT_KEYBOARD_WINDOWS_H
#define VTT_KEYBOARD_WINDOWS_H

#include <stdbool.h>
#include <windows.h>

typedef enum {
    VTT_KEY_DOWN,
    VTT_KEY_UP
} vtt_key_event_t;

typedef void (*vtt_keyboard_callback_t)(vtt_key_event_t event);

typedef struct {
    HWND hwnd;
    bool running;
    vtt_keyboard_callback_t callback;
    int hotkey_vk;  // Virtual key code (e.g., VK_SCROLL)
    UINT hotkey_modifiers;  // MOD_CONTROL, MOD_ALT, etc.
    UINT hotkey_id;  // RegisterHotKey ID
    bool key_pressed;  // Track key state
} vtt_keyboard_t;

// Initialize keyboard hook
int vtt_keyboard_init(vtt_keyboard_t *keyboard, vtt_keyboard_callback_t callback, HWND hwnd);

// Set custom hotkey (vk = virtual key code, modifiers = MOD_* flags, 0 = use default Scroll Lock)
int vtt_keyboard_set_hotkey(vtt_keyboard_t *keyboard, int vk, UINT modifiers);

// Get current hotkey virtual key code
int vtt_keyboard_get_hotkey(vtt_keyboard_t *keyboard);

// Get human-readable name for a virtual key code
const char* vtt_keyboard_get_key_name(int vk);

// Start monitoring (returns immediately, hotkeys handled in message loop)
int vtt_keyboard_start(vtt_keyboard_t *keyboard);

// Stop monitoring
void vtt_keyboard_stop(vtt_keyboard_t *keyboard);

// Cleanup
void vtt_keyboard_cleanup(vtt_keyboard_t *keyboard);

// Handle WM_HOTKEY message (called from main message loop)
void vtt_keyboard_handle_hotkey(vtt_keyboard_t *keyboard, WPARAM wParam);

#endif // VTT_KEYBOARD_WINDOWS_H
