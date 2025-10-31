#include "keyboard.h"
#include "../common/logging.h"
#include <stdio.h>
#include <string.h>

#define HOTKEY_ID_PTT 1  // Push-to-talk hotkey ID

int vtt_keyboard_init(vtt_keyboard_t *keyboard, vtt_keyboard_callback_t callback, HWND hwnd) {
    memset(keyboard, 0, sizeof(vtt_keyboard_t));
    keyboard->callback = callback;
    keyboard->hwnd = hwnd;
    keyboard->hotkey_id = HOTKEY_ID_PTT;
    keyboard->hotkey_vk = VK_SCROLL;  // Default: Scroll Lock
    keyboard->hotkey_modifiers = 0;  // No modifiers
    keyboard->key_pressed = false;
    keyboard->running = false;

    vtt_log("Keyboard hook initialized (Windows RegisterHotKey)");
    return 0;
}

int vtt_keyboard_set_hotkey(vtt_keyboard_t *keyboard, int vk, UINT modifiers) {
    if (!keyboard->hwnd) {
        return -1;
    }

    // Unregister old hotkey if running
    if (keyboard->running) {
        UnregisterHotKey(keyboard->hwnd, keyboard->hotkey_id);
    }

    // Update hotkey settings
    if (vk == 0) {
        // Reset to default
        keyboard->hotkey_vk = VK_SCROLL;
        keyboard->hotkey_modifiers = 0;
    } else {
        keyboard->hotkey_vk = vk;
        keyboard->hotkey_modifiers = modifiers;
    }

    // Re-register if running
    if (keyboard->running) {
        if (!RegisterHotKey(keyboard->hwnd, keyboard->hotkey_id,
                           keyboard->hotkey_modifiers, keyboard->hotkey_vk)) {
            vtt_log("Failed to register hotkey: VK=%d, Modifiers=0x%x (Error: %lu)",
                   keyboard->hotkey_vk, keyboard->hotkey_modifiers, GetLastError());
            return -1;
        }
    }

    vtt_log("Hotkey set to VK=%d (%s), Modifiers=0x%x",
           keyboard->hotkey_vk, vtt_keyboard_get_key_name(keyboard->hotkey_vk),
           keyboard->hotkey_modifiers);
    return 0;
}

int vtt_keyboard_get_hotkey(vtt_keyboard_t *keyboard) {
    return keyboard->hotkey_vk;
}

const char* vtt_keyboard_get_key_name(int vk) {
    static char buffer[64];

    switch (vk) {
        case VK_SCROLL: return "Scroll Lock";
        case VK_PAUSE: return "Pause";
        case VK_CAPITAL: return "Caps Lock";
        case VK_NUMLOCK: return "Num Lock";
        case VK_SNAPSHOT: return "Print Screen";
        case VK_INSERT: return "Insert";
        case VK_DELETE: return "Delete";
        case VK_HOME: return "Home";
        case VK_END: return "End";
        case VK_PRIOR: return "Page Up";
        case VK_NEXT: return "Page Down";
        case VK_F1: return "F1";
        case VK_F2: return "F2";
        case VK_F3: return "F3";
        case VK_F4: return "F4";
        case VK_F5: return "F5";
        case VK_F6: return "F6";
        case VK_F7: return "F7";
        case VK_F8: return "F8";
        case VK_F9: return "F9";
        case VK_F10: return "F10";
        case VK_F11: return "F11";
        case VK_F12: return "F12";
        case VK_LMENU: return "Left Alt";
        case VK_RMENU: return "Right Alt";
        case VK_LCONTROL: return "Left Ctrl";
        case VK_RCONTROL: return "Right Ctrl";
        case VK_LSHIFT: return "Left Shift";
        case VK_RSHIFT: return "Right Shift";
        case VK_LWIN: return "Left Windows";
        case VK_RWIN: return "Right Windows";
        case VK_APPS: return "Menu";
        default:
            snprintf(buffer, sizeof(buffer), "VK_%d", vk);
            return buffer;
    }
}

int vtt_keyboard_start(vtt_keyboard_t *keyboard) {
    if (keyboard->running) {
        return 0;
    }

    if (!keyboard->hwnd) {
        vtt_log("Cannot start keyboard monitoring: no window handle");
        return -1;
    }

    // Register the hotkey
    if (!RegisterHotKey(keyboard->hwnd, keyboard->hotkey_id,
                       keyboard->hotkey_modifiers, keyboard->hotkey_vk)) {
        vtt_log("Failed to register hotkey: VK=%d, Modifiers=0x%x (Error: %lu)",
               keyboard->hotkey_vk, keyboard->hotkey_modifiers, GetLastError());
        return -1;
    }

    keyboard->running = true;
    keyboard->key_pressed = false;

    vtt_log("Keyboard monitoring started (hotkey registered)");
    return 0;
}

void vtt_keyboard_stop(vtt_keyboard_t *keyboard) {
    if (!keyboard->running) {
        return;
    }

    if (keyboard->hwnd) {
        UnregisterHotKey(keyboard->hwnd, keyboard->hotkey_id);
    }

    keyboard->running = false;
    keyboard->key_pressed = false;

    vtt_log("Keyboard monitoring stopped");
}

void vtt_keyboard_handle_hotkey(vtt_keyboard_t *keyboard, WPARAM wParam) {
    if (wParam != keyboard->hotkey_id) {
        return;
    }

    // Check if key is currently pressed
    bool is_pressed = (GetAsyncKeyState(keyboard->hotkey_vk) & 0x8000) != 0;

    if (is_pressed && !keyboard->key_pressed) {
        // Key just pressed
        keyboard->key_pressed = true;
        keyboard->callback(VTT_KEY_DOWN);
    } else if (!is_pressed && keyboard->key_pressed) {
        // Key just released
        keyboard->key_pressed = false;
        keyboard->callback(VTT_KEY_UP);
    }
}

void vtt_keyboard_cleanup(vtt_keyboard_t *keyboard) {
    vtt_keyboard_stop(keyboard);
}
