# Wayland Support Implementation Plan

## Overview

This document outlines the plan to add native Wayland support to Voice to Text Linux.

**Current Status:** X11 only (via XTest and XGrabKey)
**Target:** Native Wayland support with X11 fallback

## Problem Statement

### Current X11 Implementation

**Keyboard Monitoring** (`src/linux/keyboard.c`):
- Uses `XGrabKey()` to capture global hotkey (Scroll Lock)
- Polls X11 events with `XNextEvent()`
- **Issue:** Wayland compositors don't support `XGrabKey`

**Text Injection** (`src/linux/typing.c`):
- Uses `XTestFakeKeyEvent()` to simulate keyboard input
- **Issue:** Wayland isolates client security - no XTest equivalent

### Wayland Solutions

**1. Keyboard Monitoring:**
- Use compositor-specific global shortcuts
- Register keybind via D-Bus (GNOME/KDE)
- Use `wl_keyboard` protocol for client-side keys

**2. Text Injection:**
- Use `wlr-virtual-keyboard-unstable-v1` protocol (wlroots)
- Use `zwp_text_input_v3` for text insertion
- Fallback: clipboard paste simulation

## Architecture

```
src/linux/
├── keyboard.c          # X11 implementation (current)
├── keyboard_wayland.c  # Wayland implementation (new)
├── typing.c            # X11 implementation (current)
├── typing_wayland.c    # Wayland implementation (new)
├── wayland_detect.c    # Runtime detection (new)
└── main.c              # Choose backend at runtime
```

## Implementation Phases

### Phase 1: Detection & Fallback ✅ (Current)

**Goal:** Detect Wayland and show helpful error

```c
// wayland_detect.c
bool is_wayland_session() {
    const char *session_type = getenv("XDG_SESSION_TYPE");
    return session_type && strcmp(session_type, "wayland") == 0;
}

bool is_wayland_display() {
    return getenv("WAYLAND_DISPLAY") != NULL;
}
```

**Update `main.c`:**
```c
if (is_wayland_session()) {
    vtt_log("WARNING: Wayland detected - limited support");
    vtt_error_notify(VTT_ERROR_GENERIC,
        "Wayland detected. Please run under XWayland or use X11 session.");
    // Continue with X11 fallback
}
```

### Phase 2: Wayland Keyboard Input (Global Hotkey)

**Compositor-Specific Solutions:**

**GNOME (most users):**
```c
// Use D-Bus to register keybind
DBusConnection *conn = dbus_bus_get(DBUS_BUS_SESSION, NULL);
dbus_bus_add_match(conn,
    "type='signal',interface='org.gnome.Shell.Extensions.KeybindingHandler'", NULL);

// Register Scroll Lock keybind
call_gnome_add_keybinding(conn, "voice-to-text-record", "Scroll_Lock");
```

**KDE Plasma:**
```c
// Use KGlobalAccel D-Bus interface
call_kglobalaccel_register(conn, "voice-to-text", "Scroll_Lock");
```

**Sway/wlroots:**
```c
// Load Wayland library
wl_display *display = wl_display_connect(NULL);
wl_registry *registry = wl_display_get_registry(display);

// Listen for global shortcuts
zwlr_hotkey_manager_v1 *hotkey_mgr = get_hotkey_manager(registry);
zwlr_hotkey_manager_v1_add_hotkey(hotkey_mgr, "Scroll_Lock");
```

### Phase 3: Wayland Text Injection

**Option A: Virtual Keyboard Protocol (Preferred)**

```c
// Get virtual keyboard manager from Wayland compositor
zwp_virtual_keyboard_manager_v1 *vk_manager = ...;

// Create virtual keyboard
zwp_virtual_keyboard_v1 *vk =
    zwp_virtual_keyboard_manager_v1_create_virtual_keyboard(vk_manager, seat);

// Send keystrokes
for (each char in text) {
    int keycode = char_to_keycode(char);
    zwp_virtual_keyboard_v1_key(vk, time, keycode, WL_KEYBOARD_KEY_STATE_PRESSED);
    zwp_virtual_keyboard_v1_key(vk, time, keycode, WL_KEYBOARD_KEY_STATE_RELEASED);
}

wl_surface_commit(...);
```

**Option B: Text Input Protocol (Fallback)**

```c
// Use text-input protocol for direct text insertion
zwp_text_input_manager_v3 *ti_manager = ...;
zwp_text_input_v3 *text_input =
    zwp_text_input_manager_v3_get_text_input(ti_manager, seat);

// Insert text directly
zwp_text_input_v3_commit_string(text_input, transcribed_text);
```

**Option C: Clipboard Paste (Universal Fallback)**

```c
// 1. Copy text to clipboard
wl_data_device_manager *ddm = ...;
wl_data_source *source = wl_data_device_manager_create_data_source(ddm);
wl_data_source_offer(source, "text/plain");
wl_data_source_send(source, "text/plain", fd);
write(fd, transcribed_text, len);

// 2. Simulate Ctrl+V via virtual keyboard
send_key_combo(vk, KEY_LEFTCTRL, KEY_V);
```

### Phase 4: Runtime Backend Selection

**Update `main.c`:**
```c
int main() {
    vtt_app_t app;

    // Detect session type
    if (is_wayland_session()) {
        vtt_log("Using Wayland backend");

        if (vtt_keyboard_wayland_init(&app.keyboard_wl, on_key_event) == 0) {
            app.backend = BACKEND_WAYLAND;
        } else {
            vtt_log("Wayland backend failed, falling back to X11");
            vtt_keyboard_init(&app.keyboard_x11, on_key_event);
            app.backend = BACKEND_X11;
        }
    } else {
        vtt_log("Using X11 backend");
        vtt_keyboard_init(&app.keyboard_x11, on_key_event);
        app.backend = BACKEND_X11;
    }

    // Use appropriate typing backend
    if (app.backend == BACKEND_WAYLAND) {
        vtt_typing_wayland_init(&app.typing_wl);
    } else {
        vtt_typing_init(&app.typing_x11);
    }
}
```

## Dependencies

### New Build Dependencies

**Wayland protocols:**
```bash
sudo apt install \
    libwayland-dev \
    wayland-protocols \
    libdbus-1-dev
```

**Protocol code generation:**
```bash
# Generate C code from XML protocols
wayland-scanner client-header < protocol.xml > protocol-client.h
wayland-scanner private-code < protocol.xml > protocol.c
```

### Runtime Detection

**Check Wayland support:**
```bash
# Check session type
echo $XDG_SESSION_TYPE

# Check Wayland display
echo $WAYLAND_DISPLAY

# Check compositor
ps aux | grep -E 'gnome-shell|kwin|sway|mutter'
```

## Compositor Compatibility Matrix

| Compositor | Hotkey Support | Text Injection | Notes |
|------------|----------------|----------------|-------|
| **GNOME** | ✅ D-Bus | ✅ Virtual Keyboard | Most common |
| **KDE Plasma** | ✅ D-Bus | ✅ Virtual Keyboard | Well supported |
| **Sway** | ✅ wlr-hotkey | ✅ Virtual Keyboard | wlroots-based |
| **Hyprland** | ⚠️ Config file | ✅ Virtual Keyboard | Requires config |
| **Mutter** | ✅ D-Bus | ✅ Virtual Keyboard | GNOME backend |

## Testing Strategy

### Unit Tests
```c
// tests/test_wayland_detect.c
void test_wayland_detection() {
    setenv("XDG_SESSION_TYPE", "wayland", 1);
    assert(is_wayland_session() == true);

    setenv("XDG_SESSION_TYPE", "x11", 1);
    assert(is_wayland_session() == false);
}
```

### Integration Tests
1. **Hotkey Test:** Press Scroll Lock → verify callback triggered
2. **Text Test:** Type "hello" → verify appears in focused app
3. **Fallback Test:** Fail Wayland init → verify X11 fallback works

### Manual Test Plan
- [ ] Test on GNOME (Ubuntu 24.04)
- [ ] Test on KDE Plasma
- [ ] Test on Sway
- [ ] Test XWayland fallback
- [ ] Test X11 native session

## Migration Path

### For Users

**Automatic detection:**
- App detects Wayland automatically
- Uses native Wayland if available
- Falls back to XWayland if needed
- Shows notification if limited support

**No config changes required**

### For Developers

**Build system:**
```makefile
# Makefile.linux
WAYLAND_CFLAGS = $(shell pkg-config --cflags wayland-client dbus-1)
WAYLAND_LDFLAGS = $(shell pkg-config --libs wayland-client dbus-1)

CFLAGS += $(WAYLAND_CFLAGS)
LDFLAGS += $(WAYLAND_LDFLAGS)

LINUX_SRCS += \
    $(SRC_DIR)/linux/keyboard_wayland.c \
    $(SRC_DIR)/linux/typing_wayland.c \
    $(SRC_DIR)/linux/wayland_detect.c
```

## Security Considerations

**Wayland's security model:**
- Clients can't spy on other clients (good!)
- Clients can't inject input to other clients (problem for us)
- Need special protocols or compositor cooperation

**Solutions:**
1. **Virtual keyboard protocol** - Trusted by compositor
2. **D-Bus permissions** - User must grant access
3. **Clipboard workaround** - Universal but clunky

## Timeline

| Phase | Effort | Priority |
|-------|--------|----------|
| Detection & Warning | 1 day | High |
| D-Bus Hotkeys (GNOME/KDE) | 3 days | High |
| Virtual Keyboard Text | 3 days | High |
| wlroots Support (Sway) | 2 days | Medium |
| Clipboard Fallback | 1 day | Low |
| Testing & Polish | 2 days | High |

**Total:** ~2 weeks for basic Wayland support

## References

- [Wayland Book](https://wayland-book.com/)
- [wlroots protocols](https://gitlab.freedesktop.org/wlroots/wlr-protocols)
- [GNOME Shell D-Bus API](https://gjs-docs.gnome.org/)
- [KDE Global Shortcuts](https://invent.kde.org/frameworks/kglobalaccel)

---

**Status:** Planning phase
**Next Step:** Implement detection and X11 fallback
**Target:** Ubuntu 24.04 LTS (GNOME 46)
