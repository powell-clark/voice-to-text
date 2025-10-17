#include "typing.h"
#include "../common/logging.h"
#include <X11/Xlib.h>
#include <X11/keysym.h>
#include <X11/extensions/XTest.h>
#include <X11/Xatom.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <ctype.h>
#include <stdbool.h>

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

// Decode one UTF-8 character and return its Unicode codepoint
// Returns the codepoint and advances *pos by the number of bytes consumed
// Returns 0 on error
static unsigned int decode_utf8(const unsigned char *text, size_t len, size_t *pos) {
    if (*pos >= len) return 0;

    unsigned char byte = text[*pos];

    // 1-byte ASCII (0xxxxxxx)
    if ((byte & 0x80) == 0) {
        (*pos)++;
        return byte;
    }

    // 2-byte sequence (110xxxxx 10xxxxxx)
    if ((byte & 0xE0) == 0xC0) {
        if (*pos + 1 >= len) return 0;
        unsigned int codepoint = ((byte & 0x1F) << 6) | (text[*pos + 1] & 0x3F);
        *pos += 2;
        return codepoint;
    }

    // 3-byte sequence (1110xxxx 10xxxxxx 10xxxxxx)
    if ((byte & 0xF0) == 0xE0) {
        if (*pos + 2 >= len) return 0;
        unsigned int codepoint = ((byte & 0x0F) << 12) |
                                  ((text[*pos + 1] & 0x3F) << 6) |
                                  (text[*pos + 2] & 0x3F);
        *pos += 3;
        return codepoint;
    }

    // 4-byte sequence (11110xxx 10xxxxxx 10xxxxxx 10xxxxxx)
    if ((byte & 0xF8) == 0xF0) {
        if (*pos + 3 >= len) return 0;
        unsigned int codepoint = ((byte & 0x07) << 18) |
                                  ((text[*pos + 1] & 0x3F) << 12) |
                                  ((text[*pos + 2] & 0x3F) << 6) |
                                  (text[*pos + 3] & 0x3F);
        *pos += 4;
        return codepoint;
    }

    // Invalid UTF-8 sequence - skip this byte
    (*pos)++;
    return 0;
}

// Paste text using clipboard (fallback for non-ASCII characters)
static void paste_text(Display *display, const char *text) {
    Window root = DefaultRootWindow(display);
    Atom clipboard = XInternAtom(display, "CLIPBOARD", False);
    Atom utf8 = XInternAtom(display, "UTF8_STRING", False);
    Atom targets = XInternAtom(display, "TARGETS", False);

    // Store text in clipboard
    XSetSelectionOwner(display, clipboard, root, CurrentTime);
    XChangeProperty(display, root, clipboard, utf8, 8, PropModeReplace,
                   (unsigned char *)text, (int)strlen(text));
    XFlush(display);

    usleep(10000); // 10ms delay for clipboard to be set

    // Simulate Ctrl+V
    KeyCode ctrl_code = XKeysymToKeycode(display, XK_Control_L);
    KeyCode v_code = XKeysymToKeycode(display, XK_v);

    XTestFakeKeyEvent(display, ctrl_code, True, CurrentTime);
    XFlush(display);
    usleep(1000);

    XTestFakeKeyEvent(display, v_code, True, CurrentTime);
    XFlush(display);
    usleep(1000);

    XTestFakeKeyEvent(display, v_code, False, CurrentTime);
    XFlush(display);
    usleep(1000);

    XTestFakeKeyEvent(display, ctrl_code, False, CurrentTime);
    XFlush(display);

    usleep(10000); // 10ms delay after paste
}

void vtt_typing_type_text(vtt_typing_t *typing, const char *text) {
    if (!text || !typing->display) {
        return;
    }

    Display *display = (Display *)typing->display;
    size_t len = strlen(text);

    vtt_log("Typing %zu bytes", len);

    if (typing->initial_delay_ms > 0) {
        usleep((useconds_t)typing->initial_delay_ms * 1000);
    }

    const unsigned char *utext = (const unsigned char *)text;
    size_t pos = 0;

    while (pos < len) {
        size_t start_pos = pos;
        unsigned int codepoint = decode_utf8(utext, len, &pos);

        if (codepoint == 0) {
            continue; // Skip invalid UTF-8
        }

        // For non-ASCII, use clipboard paste for remaining text
        if (codepoint > 127) {
            vtt_log("Non-ASCII character detected (U+%04X), using clipboard paste for remaining text", codepoint);
            paste_text(display, text + start_pos);
            break;
        }

        // Handle ASCII characters directly
        char c = (char)codepoint;
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
