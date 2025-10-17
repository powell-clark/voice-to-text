#include "typing.h"
#include "../common/logging.h"
#include <X11/Xlib.h>
#include <X11/keysym.h>
#include <X11/extensions/XTest.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <ctype.h>

int vtt_typing_init(vtt_typing_t *typing) {
    Display *display = XOpenDisplay(NULL);
    if (!display) {
        vtt_log("Failed to open X display for typing");
        return -1;
    }

    // Check XTest extension
    int event_base, error_base, major, minor;
    if (!XTestQueryExtension(display, &event_base, &error_base, &major, &minor)) {
        vtt_log("XTest extension not available");
        XCloseDisplay(display);
        return -1;
    }

    typing->display = display;
    typing->delay_ms = 4; // Small delay between keystrokes
    typing->initial_delay_ms = 75; // Pause before first character

    vtt_log("Typing system initialized (XTest)");
    return 0;
}

static void type_key(Display *display, KeySym keysym, bool shift) {
    KeyCode keycode = XKeysymToKeycode(display, keysym);
    if (keycode == 0) {
        return;
    }

    if (shift) {
        KeyCode shift_code = XKeysymToKeycode(display, XK_Shift_L);
        XTestFakeKeyEvent(display, shift_code, True, CurrentTime);
        XFlush(display);
    }

    XTestFakeKeyEvent(display, keycode, True, CurrentTime);
    XFlush(display);
    usleep(1000); // 1ms between press and release

    XTestFakeKeyEvent(display, keycode, False, CurrentTime);
    XFlush(display);

    if (shift) {
        KeyCode shift_code = XKeysymToKeycode(display, XK_Shift_L);
        XTestFakeKeyEvent(display, shift_code, False, CurrentTime);
        XFlush(display);
    }
}

void vtt_typing_type_text(vtt_typing_t *typing, const char *text) {
    if (!text || !typing->display) {
        return;
    }

    Display *display = (Display *)typing->display;
    size_t len = strlen(text);

    vtt_log("Typing %zu characters", len);

    if (typing->initial_delay_ms > 0) {
        usleep((useconds_t)typing->initial_delay_ms * 1000);
    }

    for (size_t i = 0; i < len; i++) {
        char c = text[i];
        KeySym keysym = 0;
        bool shift = false;

        // Handle special characters
        switch (c) {
            case '\n':
                keysym = XK_Return;
                break;
            case '\t':
                keysym = XK_Tab;
                break;
            case ' ':
                keysym = XK_space;
                break;
            case '!':
                keysym = XK_1;
                shift = true;
                break;
            case '@':
                keysym = XK_2;
                shift = true;
                break;
            case '#':
                keysym = XK_3;
                shift = true;
                break;
            case '$':
                keysym = XK_4;
                shift = true;
                break;
            case '%':
                keysym = XK_5;
                shift = true;
                break;
            case '^':
                keysym = XK_6;
                shift = true;
                break;
            case '&':
                keysym = XK_7;
                shift = true;
                break;
            case '*':
                keysym = XK_8;
                shift = true;
                break;
            case '(':
                keysym = XK_9;
                shift = true;
                break;
            case ')':
                keysym = XK_0;
                shift = true;
                break;
            case '-':
                keysym = XK_minus;
                break;
            case '_':
                keysym = XK_minus;
                shift = true;
                break;
            case '=':
                keysym = XK_equal;
                break;
            case '+':
                keysym = XK_equal;
                shift = true;
                break;
            case '[':
                keysym = XK_bracketleft;
                break;
            case '{':
                keysym = XK_bracketleft;
                shift = true;
                break;
            case ']':
                keysym = XK_bracketright;
                break;
            case '}':
                keysym = XK_bracketright;
                shift = true;
                break;
            case '\\':
                keysym = XK_backslash;
                break;
            case '|':
                keysym = XK_backslash;
                shift = true;
                break;
            case ';':
                keysym = XK_semicolon;
                break;
            case ':':
                keysym = XK_semicolon;
                shift = true;
                break;
            case '\'':
                keysym = XK_apostrophe;
                break;
            case '"':
                keysym = XK_apostrophe;
                shift = true;
                break;
            case ',':
                keysym = XK_comma;
                break;
            case '<':
                keysym = XK_comma;
                shift = true;
                break;
            case '.':
                keysym = XK_period;
                break;
            case '>':
                keysym = XK_period;
                shift = true;
                break;
            case '/':
                keysym = XK_slash;
                break;
            case '?':
                keysym = XK_slash;
                shift = true;
                break;
            case '`':
                keysym = XK_grave;
                break;
            case '~':
                keysym = XK_grave;
                shift = true;
                break;
            default:
                if (isupper(c)) {
                    keysym = XK_a + (c - 'A');
                    shift = true;
                } else if (islower(c)) {
                    keysym = XK_a + (c - 'a');
                } else if (isdigit(c)) {
                    keysym = XK_0 + (c - '0');
                }
                break;
        }

        if (keysym != 0) {
            type_key(display, keysym, shift);
            if (typing->delay_ms > 0) {
                usleep((useconds_t)typing->delay_ms * 1000);
            }
        }
    }

    vtt_log("Typing completed");
}

void vtt_typing_cleanup(vtt_typing_t *typing) {
    if (typing->display) {
        XCloseDisplay((Display *)typing->display);
    }
}
