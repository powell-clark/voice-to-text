#include "typing_wayland.h"
#include "wayland_detect.h"
#include "../common/logging.h"
#include "../common/error_handler.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>

const char *vtt_typing_wayland_method_name(vtt_typing_method_t method) {
    switch (method) {
        case VTT_TYPING_METHOD_YDOTOOL: return "ydotool";
        case VTT_TYPING_METHOD_DOTOOL: return "dotool";
        case VTT_TYPING_METHOD_CLIPBOARD: return "clipboard+paste";
        case VTT_TYPING_METHOD_DBUS: return "D-Bus";
        case VTT_TYPING_METHOD_NONE: return "none";
        default: return "unknown";
    }
}

// Check if a command exists in PATH
static bool command_exists(const char *cmd) {
    char check_cmd[256];
    snprintf(check_cmd, sizeof(check_cmd), "command -v %s >/dev/null 2>&1", cmd);
    return system(check_cmd) == 0;
}

// Detect best available typing method for Wayland
static vtt_typing_method_t detect_typing_method(void) {
    // Check for ydotool (most reliable for wlroots compositors)
    if (command_exists("ydotool")) {
        // Verify ydotoold is running
        if (system("pgrep -x ydotoold >/dev/null 2>&1") == 0) {
            return VTT_TYPING_METHOD_YDOTOOL;
        } else {
            vtt_log("ydotool found but ydotoold daemon not running");
            vtt_log("Start with: sudo systemctl start ydotool");
        }
    }

    // Check for dotool (alternative for wlroots)
    if (command_exists("dotool")) {
        return VTT_TYPING_METHOD_DOTOOL;
    }

    // Check for wl-clipboard (clipboard method)
    if (command_exists("wl-copy") && command_exists("wl-paste")) {
        return VTT_TYPING_METHOD_CLIPBOARD;
    }

    // No suitable method found
    return VTT_TYPING_METHOD_NONE;
}

int vtt_typing_wayland_init(vtt_typing_wayland_t *typing) {
    memset(typing, 0, sizeof(vtt_typing_wayland_t));

    // Get compositor name
    const char *compositor = vtt_get_wayland_compositor();
    typing->compositor = strdup(compositor);

    // Detect typing method
    typing->method = detect_typing_method();

    typing->delay_ms = 4;
    typing->initial_delay_ms = 75;
    typing->newline_type = NEWLINE_SHIFT_RETURN;

    if (typing->method == VTT_TYPING_METHOD_NONE) {
        vtt_log("ERROR: No Wayland typing method available");
        vtt_error_notify(VTT_ERROR_GENERIC,
            "Wayland typing not available. Install ydotool (recommended) or wl-clipboard.\n\n"
            "Ubuntu/Debian: sudo apt install ydotool wl-clipboard\n"
            "Then start daemon: sudo systemctl start ydotool"
        );
        free(typing->compositor);
        return -1;
    }

    vtt_log("Wayland typing initialized: method=%s, compositor=%s",
            vtt_typing_wayland_method_name(typing->method), compositor);

    return 0;
}

// Type text using ydotool
static void type_with_ydotool(const char *text) {
    // ydotool type accepts text on stdin or as argument
    // Using stdin is more reliable for special characters
    FILE *fp = popen("ydotool type --file=-", "w");
    if (!fp) {
        vtt_log("Failed to execute ydotool");
        return;
    }

    fprintf(fp, "%s", text);
    pclose(fp);
}

// Type text using dotool
static void type_with_dotool(const char *text) {
    FILE *fp = popen("dotool", "w");
    if (!fp) {
        vtt_log("Failed to execute dotool");
        return;
    }

    // dotool uses command format: "type text here"
    fprintf(fp, "type %s\n", text);
    pclose(fp);
}

// Type text using clipboard + paste
static void type_with_clipboard(const char *text, vtt_newline_type_t newline_type) {
    // Copy text to clipboard
    FILE *fp = popen("wl-copy", "w");
    if (!fp) {
        vtt_log("Failed to execute wl-copy");
        vtt_error_notify(VTT_ERROR_GENERIC, "Clipboard typing failed: wl-copy not available");
        return;
    }

    fprintf(fp, "%s", text);
    int copy_status = pclose(fp);

    if (copy_status != 0) {
        vtt_log("wl-copy failed with status %d", copy_status);
        return;
    }

    // Small delay to ensure clipboard is ready
    usleep(50000); // 50ms

    // Simulate Ctrl+V or Shift+Insert based on newline type
    // For Wayland, we use ydotool or dotool for the paste command
    if (command_exists("ydotool")) {
        // ydotool key: Ctrl=29, V=47, Shift=42, Insert=110
        if (newline_type == NEWLINE_SHIFT_RETURN) {
            system("ydotool key 42:1 110:1 110:0 42:0"); // Shift+Insert
        } else {
            system("ydotool key 29:1 47:1 47:0 29:0"); // Ctrl+V
        }
    } else if (command_exists("dotool")) {
        if (newline_type == NEWLINE_SHIFT_RETURN) {
            system("echo 'keydown leftshift\nkey insert\nkeyup leftshift' | dotool");
        } else {
            system("echo 'keydown leftctrl\nkey v\nkeyup leftctrl' | dotool");
        }
    } else {
        vtt_log("ERROR: Cannot simulate paste - no input tool available");
        vtt_error_notify(VTT_ERROR_GENERIC,
            "Clipboard paste requires ydotool or dotool for key simulation");
    }
}

void vtt_typing_wayland_type_text(vtt_typing_wayland_t *typing, const char *text) {
    if (!text || !*text) {
        return;
    }

    // Initial delay before typing
    if (typing->initial_delay_ms > 0) {
        usleep(typing->initial_delay_ms * 1000);
    }

    switch (typing->method) {
        case VTT_TYPING_METHOD_YDOTOOL:
            type_with_ydotool(text);
            break;

        case VTT_TYPING_METHOD_DOTOOL:
            type_with_dotool(text);
            break;

        case VTT_TYPING_METHOD_CLIPBOARD:
            type_with_clipboard(text, typing->newline_type);
            break;

        case VTT_TYPING_METHOD_DBUS:
            vtt_log("D-Bus typing method not yet implemented");
            vtt_error_notify(VTT_ERROR_GENERIC, "D-Bus typing not implemented yet");
            break;

        case VTT_TYPING_METHOD_NONE:
            vtt_log("No typing method available");
            break;
    }

    // Small delay after typing
    if (typing->delay_ms > 0) {
        usleep(typing->delay_ms * 1000);
    }
}

void vtt_typing_wayland_cleanup(vtt_typing_wayland_t *typing) {
    if (typing->compositor) {
        free(typing->compositor);
        typing->compositor = NULL;
    }

    vtt_log("Wayland typing cleanup complete");
}
