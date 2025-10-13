#include "keyboard.h"
#include "../common/logging.h"
#include <X11/Xlib.h>
#include <X11/keysym.h>
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

    // Get Scroll Lock keycode
    KeySym scroll_lock_sym = XK_Scroll_Lock;
    keyboard->scroll_lock_keycode = XKeysymToKeycode(display, scroll_lock_sym);

    if (keyboard->scroll_lock_keycode == 0) {
        vtt_log("Warning: Scroll Lock key not found, using keycode 78");
        keyboard->scroll_lock_keycode = 78; // Fallback for Scroll Lock
    }

    // Grab the key globally on root window
    Window root = DefaultRootWindow(display);

    // Grab key with any modifiers (Normal, NumLock, CapsLock, ScrollLock combinations)
    XGrabKey(display, keyboard->scroll_lock_keycode, AnyModifier, root,
             True, GrabModeAsync, GrabModeAsync);

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

void vtt_keyboard_cleanup(vtt_keyboard_t *keyboard) {
    vtt_keyboard_stop(keyboard);

    Display *display = (Display *)keyboard->display;

    if (display) {
        // Ungrab the key
        Window root = DefaultRootWindow(display);
        XUngrabKey(display, keyboard->scroll_lock_keycode, AnyModifier, root);

        XCloseDisplay(display);
    }
}
