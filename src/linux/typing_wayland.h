#ifndef VTT_TYPING_WAYLAND_H
#define VTT_TYPING_WAYLAND_H

#include <stdbool.h>
#include "../common/settings.h"

typedef enum {
    VTT_TYPING_METHOD_YDOTOOL,     // ydotool (userspace input)
    VTT_TYPING_METHOD_DOTOOL,      // dotool (Wayland input tool)
    VTT_TYPING_METHOD_CLIPBOARD,   // Clipboard + Ctrl+V
    VTT_TYPING_METHOD_DBUS,        // D-Bus input method
    VTT_TYPING_METHOD_NONE
} vtt_typing_method_t;

typedef struct {
    vtt_typing_method_t method;
    int delay_ms;
    int initial_delay_ms;
    vtt_newline_type_t newline_type;
    char *compositor;
} vtt_typing_wayland_t;

// Initialize Wayland typing system (auto-detects best method)
int vtt_typing_wayland_init(vtt_typing_wayland_t *typing);

// Type text string
void vtt_typing_wayland_type_text(vtt_typing_wayland_t *typing, const char *text);

// Cleanup
void vtt_typing_wayland_cleanup(vtt_typing_wayland_t *typing);

// Get method name for logging
const char *vtt_typing_wayland_method_name(vtt_typing_method_t method);

#endif // VTT_TYPING_WAYLAND_H
