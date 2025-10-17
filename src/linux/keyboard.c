#include "keyboard.h"
#include "../common/logging.h"
#include <X11/Xlib.h>
#include <X11/keysym.h>
#include <X11/XKBlib.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <unistd.h>

static void *monitor_thread(void *arg) {
    vtt_keyboard_t *keyboard = (vtt_keyboard_t *)arg;
    Display *display = (Display *)keyboard->display;

    XEvent event;
    while (keyboard->running) {
        // Wait for next event (blocking)
        XNextEvent(display, &event);

        if (event.type == KeyPress) {
            if (event.xkey.keycode == keyboard->scroll_lock_keycode) {
                keyboard->callback(VTT_KEY_DOWN);
            }
        } else if (event.type == KeyRelease) {
            if (event.xkey.keycode == keyboard->scroll_lock_keycode) {
                keyboard->callback(VTT_KEY_UP);
            }
        }
    }

    return NULL;
}

int vtt_keyboard_init(vtt_keyboard_t *keyboard, vtt_keyboard_callback_t callback) {
    memset(keyboard, 0, sizeof(vtt_keyboard_t));
    keyboard->callback = callback;

    // Open display
    Display *display = XOpenDisplay(NULL);
    if (!display) {
        vtt_log("Failed to open X display");
        return -1;
    }
    keyboard->display = display;

    Bool detectable_supported = False;
    if (XkbSetDetectableAutoRepeat(display, True, &detectable_supported)) {
        if (!detectable_supported) {
            vtt_log("Detectable auto-repeat not supported; hotkey may generate repeats");
        }
    } else {
        vtt_log("Failed to enable detectable auto-repeat; hotkey may generate repeats");
    }

    // Get Scroll Lock keycode
    KeySym scroll_lock_sym = XK_Scroll_Lock;
    keyboard->scroll_lock_keycode = XKeysymToKeycode(display, scroll_lock_sym);

    if (keyboard->scroll_lock_keycode == 0) {
        vtt_log("Warning: Scroll Lock key not found, using keycode 78");
        keyboard->scroll_lock_keycode = 78; // Fallback for Scroll Lock
    }

    // Grab the key globally on root window
    Window root = DefaultRootWindow(display);

    // Grab key with specific modifier combinations to avoid interfering with keyboard state
    // We need to handle Num Lock and Caps Lock being on/off
    unsigned int modifiers[] = {
        0,              // No modifiers
        LockMask,       // Caps Lock
        Mod2Mask,       // Num Lock
        LockMask | Mod2Mask  // Both
    };

    for (size_t i = 0; i < sizeof(modifiers) / sizeof(modifiers[0]); i++) {
        XGrabKey(display, keyboard->scroll_lock_keycode, modifiers[i], root,
                 True, GrabModeAsync, GrabModeAsync);
    }

    // Sync to ensure grab is registered
    XSync(display, False);

    keyboard->running = false;

    vtt_log("Keyboard hook initialized (Scroll Lock, keycode %d)", keyboard->scroll_lock_keycode);
    return 0;
}

int vtt_keyboard_start(vtt_keyboard_t *keyboard) {
    if (keyboard->running) {
        return 0;
    }

    keyboard->running = true;

    // Start monitoring thread
    pthread_t thread;
    if (pthread_create(&thread, NULL, monitor_thread, keyboard) != 0) {
        vtt_log("Failed to create keyboard monitor thread");
        keyboard->running = false;
        return -1;
    }
    pthread_detach(thread);

    vtt_log("Keyboard monitoring started");
    return 0;
}

void vtt_keyboard_stop(vtt_keyboard_t *keyboard) {
    if (!keyboard->running) {
        return;
    }

    keyboard->running = false;

    // Send a dummy event to wake up XNextEvent
    Display *display = (Display *)keyboard->display;
    if (display) {
        XClientMessageEvent dummy = {0};
        dummy.type = ClientMessage;
        dummy.window = DefaultRootWindow(display);
        dummy.format = 32;
        XSendEvent(display, DefaultRootWindow(display), False, 0, (XEvent*)&dummy);
        XFlush(display);
    }

    vtt_log("Keyboard monitoring stopped");
}

int vtt_keyboard_set_hotkey(vtt_keyboard_t *keyboard, int keycode) {
    Display *display = (Display *)keyboard->display;
    if (!display) {
        return -1;
    }

    Window root = DefaultRootWindow(display);

    // Ungrab old key with all modifier combinations
    unsigned int modifiers[] = {
        0,              // No modifiers
        LockMask,       // Caps Lock
        Mod2Mask,       // Num Lock
        LockMask | Mod2Mask  // Both
    };

    for (size_t i = 0; i < sizeof(modifiers) / sizeof(modifiers[0]); i++) {
        XUngrabKey(display, keyboard->scroll_lock_keycode, modifiers[i], root);
    }

    // Set new keycode (0 = default to Scroll Lock)
    if (keycode == 0) {
        KeySym scroll_lock_sym = XK_Scroll_Lock;
        keyboard->scroll_lock_keycode = XKeysymToKeycode(display, scroll_lock_sym);
        if (keyboard->scroll_lock_keycode == 0) {
            keyboard->scroll_lock_keycode = 78; // Fallback
        }
    } else {
        keyboard->scroll_lock_keycode = keycode;
    }

    // Grab new key with all modifier combinations
    for (size_t i = 0; i < sizeof(modifiers) / sizeof(modifiers[0]); i++) {
        XGrabKey(display, keyboard->scroll_lock_keycode, modifiers[i], root,
                 True, GrabModeAsync, GrabModeAsync);
    }
    XSync(display, False);

    vtt_log("Hotkey changed to keycode %d", keyboard->scroll_lock_keycode);
    return 0;
}

int vtt_keyboard_get_hotkey(vtt_keyboard_t *keyboard) {
    return keyboard->scroll_lock_keycode;
}

const char* vtt_keyboard_get_key_name(void *display_ptr, int keycode) {
    static char buffer[64];
    Display *display = (Display *)display_ptr;

    if (!display || keycode == 0) {
        return "Unknown";
    }

    // Get KeySym from keycode
    KeySym keysym = XKeycodeToKeysym(display, keycode, 0);
    if (keysym == NoSymbol) {
        snprintf(buffer, sizeof(buffer), "Key %d", keycode);
        return buffer;
    }

    // Map common keysyms to friendly names
    switch (keysym) {
        case XK_Scroll_Lock: return "Scroll Lock";
        case XK_Caps_Lock: return "Caps Lock";
        case XK_Num_Lock: return "Num Lock";
        case XK_Pause: return "Pause";
        case XK_Print: return "Print Screen";
        case XK_Insert: return "Insert";
        case XK_Home: return "Home";
        case XK_End: return "End";
        case XK_Page_Up: return "Page Up";
        case XK_Page_Down: return "Page Down";
        case XK_F1: return "F1";
        case XK_F2: return "F2";
        case XK_F3: return "F3";
        case XK_F4: return "F4";
        case XK_F5: return "F5";
        case XK_F6: return "F6";
        case XK_F7: return "F7";
        case XK_F8: return "F8";
        case XK_F9: return "F9";
        case XK_F10: return "F10";
        case XK_F11: return "F11";
        case XK_F12: return "F12";
        case XK_Alt_L: return "Left Alt";
        case XK_Alt_R: return "Right Alt";
        case XK_Control_L: return "Left Ctrl";
        case XK_Control_R: return "Right Ctrl";
        case XK_Shift_L: return "Left Shift";
        case XK_Shift_R: return "Right Shift";
        case XK_Super_L: return "Left Super";
        case XK_Super_R: return "Right Super";
        case XK_Menu: return "Menu";
        default: {
            // Try to get the string representation
            char *keyname = XKeysymToString(keysym);
            if (keyname) {
                snprintf(buffer, sizeof(buffer), "%s", keyname);
                return buffer;
            }
            snprintf(buffer, sizeof(buffer), "Key %d", keycode);
            return buffer;
        }
    }
}

void vtt_keyboard_cleanup(vtt_keyboard_t *keyboard) {
    vtt_keyboard_stop(keyboard);

    Display *display = (Display *)keyboard->display;

    if (display) {
        // Ungrab the key with all modifier combinations
        Window root = DefaultRootWindow(display);
        unsigned int modifiers[] = {
            0,              // No modifiers
            LockMask,       // Caps Lock
            Mod2Mask,       // Num Lock
            LockMask | Mod2Mask  // Both
        };

        for (size_t i = 0; i < sizeof(modifiers) / sizeof(modifiers[0]); i++) {
            XUngrabKey(display, keyboard->scroll_lock_keycode, modifiers[i], root);
        }

        XCloseDisplay(display);
    }
}
