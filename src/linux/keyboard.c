#include "keyboard.h"
#include "../common/logging.h"
#include <X11/Xlib.h>
#include <X11/extensions/record.h>
#include <X11/keysym.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>

static void event_callback(XPointer priv, XRecordInterceptData *data) {
    vtt_keyboard_t *keyboard = (vtt_keyboard_t *)priv;

    if (data->category == XRecordFromServer) {
        // Extract event type and keycode from raw data
        unsigned char event_type = data->data[0] & 0x7F;
        unsigned char keycode = data->data[1];

        if (event_type == KeyPress || event_type == KeyRelease) {
            if (keycode == keyboard->scroll_lock_keycode) {
                vtt_key_event_t key_event = (event_type == KeyPress) ? VTT_KEY_DOWN : VTT_KEY_UP;
                keyboard->callback(key_event);
            }
        }
    }

    XRecordFreeData(data);
}

static void *monitor_thread(void *arg) {
    vtt_keyboard_t *keyboard = (vtt_keyboard_t *)arg;
    Display *data_display = (Display *)keyboard->data_display;
    XRecordContext context = (XRecordContext)(long)keyboard->context;

    // This blocks and calls event_callback for each event
    if (!XRecordEnableContext(data_display, context, event_callback, (XPointer)keyboard)) {
        vtt_log("Failed to enable XRecord context");
        return NULL;
    }

    return NULL;
}

int vtt_keyboard_init(vtt_keyboard_t *keyboard, vtt_keyboard_callback_t callback) {
    memset(keyboard, 0, sizeof(vtt_keyboard_t));
    keyboard->callback = callback;

    // Open main display
    Display *display = XOpenDisplay(NULL);
    if (!display) {
        vtt_log("Failed to open X display");
        return -1;
    }
    keyboard->display = display;

    // Open data display for XRecord
    Display *data_display = XOpenDisplay(NULL);
    if (!data_display) {
        vtt_log("Failed to open data display");
        XCloseDisplay(display);
        return -1;
    }
    keyboard->data_display = data_display;

    // Get Scroll Lock keycode
    KeySym scroll_lock_sym = XK_Scroll_Lock;
    keyboard->scroll_lock_keycode = XKeysymToKeycode(display, scroll_lock_sym);

    if (keyboard->scroll_lock_keycode == 0) {
        vtt_log("Warning: Scroll Lock key not found, using keycode 78");
        keyboard->scroll_lock_keycode = 78; // Fallback
    }

    // Set up XRecord extension
    int major, minor;
    if (!XRecordQueryVersion(display, &major, &minor)) {
        vtt_log("XRecord extension not available");
        XCloseDisplay(data_display);
        XCloseDisplay(display);
        return -1;
    }

    XRecordRange *range = XRecordAllocRange();
    if (!range) {
        vtt_log("Failed to allocate XRecord range");
        XCloseDisplay(data_display);
        XCloseDisplay(display);
        return -1;
    }

    range->device_events.first = KeyPress;
    range->device_events.last = KeyRelease;

    XRecordClientSpec client_spec = XRecordAllClients;
    XRecordContext context = XRecordCreateContext(display, 0, &client_spec, 1, &range, 1);

    XFree(range);

    if (!context) {
        vtt_log("Failed to create XRecord context");
        XCloseDisplay(data_display);
        XCloseDisplay(display);
        return -1;
    }

    keyboard->context = (void *)(long)context;
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

    Display *data_display = (Display *)keyboard->data_display;
    XRecordContext context = (XRecordContext)(long)keyboard->context;

    if (data_display && context) {
        XRecordDisableContext(data_display, context);
    }

    vtt_log("Keyboard monitoring stopped");
}

void vtt_keyboard_cleanup(vtt_keyboard_t *keyboard) {
    vtt_keyboard_stop(keyboard);

    Display *display = (Display *)keyboard->display;
    Display *data_display = (Display *)keyboard->data_display;
    XRecordContext context = (XRecordContext)(long)keyboard->context;

    if (data_display && context) {
        XRecordFreeContext(data_display, context);
    }

    if (data_display) {
        XCloseDisplay(data_display);
    }

    if (display) {
        XCloseDisplay(display);
    }
}
