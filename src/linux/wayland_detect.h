#ifndef VTT_WAYLAND_DETECT_H
#define VTT_WAYLAND_DETECT_H

#include <stdbool.h>

// Check if running under Wayland session
bool vtt_is_wayland_session(void);

// Check if Wayland display is available
bool vtt_is_wayland_display(void);

// Get compositor name (gnome-shell, kwin_wayland, sway, etc.)
const char *vtt_get_wayland_compositor(void);

// Check if specific compositor is running
bool vtt_is_compositor(const char *name);

// Check if XWayland is available
bool vtt_has_xwayland(void);

#endif // VTT_WAYLAND_DETECT_H
