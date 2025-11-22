#include "wayland_detect.h"
#include "../common/logging.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <unistd.h>

bool vtt_is_wayland_session(void) {
    const char *session_type = getenv("XDG_SESSION_TYPE");
    if (session_type && strcmp(session_type, "wayland") == 0) {
        return true;
    }
    return false;
}

bool vtt_is_wayland_display(void) {
    const char *wayland_display = getenv("WAYLAND_DISPLAY");
    return wayland_display != NULL && wayland_display[0] != '\0';
}

const char *vtt_get_wayland_compositor(void) {
    static char compositor[256] = {0};

    if (compositor[0] != '\0') {
        return compositor;  // Cached
    }

    // Check common compositor process names
    const char *compositors[] = {
        "gnome-shell",
        "kwin_wayland",
        "sway",
        "mutter",
        "weston",
        "hyprland",
        "river",
        "wayfire",
        NULL
    };

    FILE *fp = popen("ps aux", "r");
    if (!fp) {
        return "unknown";
    }

    char line[512];
    while (fgets(line, sizeof(line), fp)) {
        for (int i = 0; compositors[i] != NULL; i++) {
            if (strstr(line, compositors[i])) {
                strncpy(compositor, compositors[i], sizeof(compositor) - 1);
                pclose(fp);
                return compositor;
            }
        }
    }

    pclose(fp);
    strncpy(compositor, "unknown", sizeof(compositor) - 1);
    return compositor;
}

bool vtt_is_compositor(const char *name) {
    const char *compositor = vtt_get_wayland_compositor();
    return strcmp(compositor, name) == 0;
}

bool vtt_has_xwayland(void) {
    // Check if DISPLAY is set (indicates XWayland)
    const char *display = getenv("DISPLAY");
    if (!display || display[0] == '\0') {
        return false;
    }

    // Check if X server is accessible
    FILE *fp = popen("xdpyinfo >/dev/null 2>&1 && echo yes", "r");
    if (!fp) {
        return false;
    }

    char result[16] = {0};
    if (fgets(result, sizeof(result), fp)) {
        pclose(fp);
        return strstr(result, "yes") != NULL;
    }

    pclose(fp);
    return false;
}
