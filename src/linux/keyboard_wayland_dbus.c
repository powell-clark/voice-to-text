#include "keyboard_wayland.h"
#include "wayland_detect.h"
#include "../common/logging.h"
#include "../common/error_handler.h"
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>

// D-Bus for GNOME/KDE global shortcuts
#ifdef __linux__
#include <dbus/dbus.h>
#endif

typedef struct {
    vtt_keyboard_wayland_t *keyboard;
    DBusConnection *conn;
    bool running;
} dbus_monitor_t;

static void *dbus_monitor_thread(void *arg) {
    dbus_monitor_t *monitor = (dbus_monitor_t *)arg;
    DBusConnection *conn = monitor->conn;

    vtt_log("D-Bus monitor thread started");

    while (monitor->running) {
        // Wait for D-Bus messages with timeout
        dbus_connection_read_write_dispatch(conn, 100);

        DBusMessage *msg = dbus_connection_pop_message(conn);
        if (!msg) continue;

        // Check if this is a keybinding signal
        if (dbus_message_is_signal(msg, "org.gnome.Shell", "AcceleratorActivated")) {
            DBusMessageIter iter;
            dbus_message_iter_init(msg, &iter);

            // Get action ID
            const char *action = NULL;
            if (dbus_message_iter_get_arg_type(&iter) == DBUS_TYPE_STRING) {
                dbus_message_iter_get_basic(&iter, &action);

                if (action && strcmp(action, "voice-to-text-record") == 0) {
                    // Get activation parameters
                    dbus_message_iter_next(&iter);
                    DBusMessageIter dict_iter;
                    dbus_message_iter_recurse(&iter, &dict_iter);

                    // Parse pressed/released state
                    // For now, we'll toggle on each activation
                    // TODO: Implement proper press/release detection
                    vtt_log("Hotkey activated via D-Bus");
                    if (monitor->keyboard->callback) {
                        monitor->keyboard->callback(VTT_KEY_DOWN);
                        usleep(100000); // 100ms
                        monitor->keyboard->callback(VTT_KEY_UP);
                    }
                }
            }
        }

        dbus_message_unref(msg);
    }

    vtt_log("D-Bus monitor thread stopped");
    return NULL;
}

static int init_gnome_dbus(vtt_keyboard_wayland_t *keyboard) {
    DBusError err;
    dbus_error_init(&err);

    // Connect to session bus
    DBusConnection *conn = dbus_bus_get(DBUS_BUS_SESSION, &err);
    if (dbus_error_is_set(&err)) {
        vtt_log("D-Bus connection error: %s", err.message);
        dbus_error_free(&err);
        return -1;
    }

    if (!conn) {
        vtt_log("Failed to connect to D-Bus session bus");
        return -1;
    }

    keyboard->dbus_connection = conn;

    // Add match rule for GNOME Shell accelerator signals
    const char *match_rule = "type='signal',interface='org.gnome.Shell',member='AcceleratorActivated'";
    dbus_bus_add_match(conn, match_rule, &err);
    if (dbus_error_is_set(&err)) {
        vtt_log("D-Bus add match error: %s", err.message);
        dbus_error_free(&err);
        dbus_connection_unref(conn);
        return -1;
    }

    dbus_connection_flush(conn);

    // Call GNOME Shell to grab accelerator
    // Note: This requires GNOME Shell extension or manual configuration
    vtt_log("D-Bus initialized for GNOME Shell");
    vtt_log("Note: Requires manual keybinding setup in GNOME Settings");
    vtt_log("Go to Settings -> Keyboard -> Custom Shortcuts");
    vtt_log("Add shortcut: Name='Voice to Text', Command='echo voice-to-text-record', Key='Scroll Lock'");

    return 0;
}

int vtt_keyboard_wayland_init(vtt_keyboard_wayland_t *keyboard, vtt_keyboard_callback_t callback) {
    memset(keyboard, 0, sizeof(vtt_keyboard_wayland_t));
    keyboard->callback = callback;
    keyboard->hotkey_keycode = 78;  // Scroll Lock default

    // Detect compositor
    const char *compositor = vtt_get_wayland_compositor();
    keyboard->compositor = strdup(compositor);

    vtt_log("Wayland keyboard init for compositor: %s", compositor);

    // Try compositor-specific initialization
    if (strcmp(compositor, "gnome-shell") == 0 || strcmp(compositor, "mutter") == 0) {
        if (init_gnome_dbus(keyboard) == 0) {
            vtt_log("GNOME D-Bus keyboard initialized (experimental)");
            keyboard->running = false;
            return 0;
        }
    }

    // Fallback: not implemented for this compositor
    vtt_log("Wayland keyboard not fully implemented for: %s", compositor);
    vtt_error_notify(VTT_ERROR_GENERIC,
        "Wayland keyboard support is experimental. Use X11 session or configure hotkey manually in system settings.");

    return -1;
}

int vtt_keyboard_wayland_start(vtt_keyboard_wayland_t *keyboard) {
    if (keyboard->running) return 0;

    if (!keyboard->dbus_connection) {
        vtt_log("Cannot start Wayland keyboard: not initialized");
        return -1;
    }

    keyboard->running = true;

    // Start D-Bus monitoring thread
    dbus_monitor_t *monitor = malloc(sizeof(dbus_monitor_t));
    monitor->keyboard = keyboard;
    monitor->conn = (DBusConnection *)keyboard->dbus_connection;
    monitor->running = true;

    pthread_t thread;
    if (pthread_create(&thread, NULL, dbus_monitor_thread, monitor) != 0) {
        vtt_log("Failed to create D-Bus monitor thread");
        keyboard->running = false;
        free(monitor);
        return -1;
    }
    pthread_detach(thread);

    vtt_log("Wayland keyboard monitoring started");
    return 0;
}

void vtt_keyboard_wayland_stop(vtt_keyboard_wayland_t *keyboard) {
    keyboard->running = false;
    vtt_log("Wayland keyboard monitoring stopped");
}

void vtt_keyboard_wayland_cleanup(vtt_keyboard_wayland_t *keyboard) {
    vtt_keyboard_wayland_stop(keyboard);

    if (keyboard->compositor) {
        free(keyboard->compositor);
        keyboard->compositor = NULL;
    }

    if (keyboard->dbus_connection) {
        DBusConnection *conn = (DBusConnection *)keyboard->dbus_connection;
        dbus_connection_unref(conn);
        keyboard->dbus_connection = NULL;
    }
}

int vtt_keyboard_wayland_set_hotkey(vtt_keyboard_wayland_t *keyboard, int keycode) {
    keyboard->hotkey_keycode = keycode;
    vtt_log("Wayland hotkey set to keycode %d", keycode);
    vtt_log("Note: You must manually configure this in GNOME Settings");
    return 0;
}

int vtt_keyboard_wayland_get_hotkey(vtt_keyboard_wayland_t *keyboard) {
    return keyboard->hotkey_keycode;
}
