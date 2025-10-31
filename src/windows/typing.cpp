#include "typing.h"
#include "../common/logging.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

int vtt_typing_init(vtt_typing_t *typing) {
    typing->delay_ms = 4;  // Small delay between keystrokes
    typing->initial_delay_ms = 75;  // Pause before first character
    typing->newline_type = NEWLINE_SHIFT_RETURN;  // Default to safer option

    vtt_log("Typing system initialized (Windows SendInput)");
    return 0;
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

// Convert UTF-8 string to UTF-16 (Windows wide string)
static WCHAR* utf8_to_utf16(const char *utf8_str) {
    if (!utf8_str) return NULL;

    int len = MultiByteToWideChar(CP_UTF8, 0, utf8_str, -1, NULL, 0);
    if (len <= 0) return NULL;

    WCHAR *utf16_str = (WCHAR*)malloc(len * sizeof(WCHAR));
    MultiByteToWideChar(CP_UTF8, 0, utf8_str, -1, utf16_str, len);

    return utf16_str;
}

// Paste text using clipboard (fallback for complex text or SendInput failures)
static void paste_text(const char *text) {
    WCHAR *wtext = utf8_to_utf16(text);
    if (!wtext) return;

    // Open clipboard
    if (!OpenClipboard(NULL)) {
        free(wtext);
        return;
    }

    // Empty clipboard
    EmptyClipboard();

    // Allocate global memory for clipboard
    size_t wlen = wcslen(wtext);
    HGLOBAL hMem = GlobalAlloc(GMEM_MOVEABLE, (wlen + 1) * sizeof(WCHAR));
    if (!hMem) {
        CloseClipboard();
        free(wtext);
        return;
    }

    // Copy text to global memory
    WCHAR *pMem = (WCHAR*)GlobalLock(hMem);
    wcscpy(pMem, wtext);
    GlobalUnlock(hMem);

    // Set clipboard data
    SetClipboardData(CF_UNICODETEXT, hMem);
    CloseClipboard();

    free(wtext);

    // Wait for clipboard to be set
    Sleep(10);

    // Simulate Ctrl+V
    INPUT inputs[4];
    ZeroMemory(inputs, sizeof(inputs));

    // Press Ctrl
    inputs[0].type = INPUT_KEYBOARD;
    inputs[0].ki.wVk = VK_CONTROL;

    // Press V
    inputs[1].type = INPUT_KEYBOARD;
    inputs[1].ki.wVk = 'V';

    // Release V
    inputs[2].type = INPUT_KEYBOARD;
    inputs[2].ki.wVk = 'V';
    inputs[2].ki.dwFlags = KEYEVENTF_KEYUP;

    // Release Ctrl
    inputs[3].type = INPUT_KEYBOARD;
    inputs[3].ki.wVk = VK_CONTROL;
    inputs[3].ki.dwFlags = KEYEVENTF_KEYUP;

    SendInput(4, inputs, sizeof(INPUT));

    Sleep(10);  // Allow paste to complete
}

// Send a single Unicode character using SendInput
static void send_unicode_char(WCHAR wch) {
    INPUT inputs[2];
    ZeroMemory(inputs, sizeof(inputs));

    // Key down
    inputs[0].type = INPUT_KEYBOARD;
    inputs[0].ki.wScan = wch;
    inputs[0].ki.dwFlags = KEYEVENTF_UNICODE;

    // Key up
    inputs[1].type = INPUT_KEYBOARD;
    inputs[1].ki.wScan = wch;
    inputs[1].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;

    SendInput(2, inputs, sizeof(INPUT));
}

// Send a virtual key (for special keys like Enter, Tab)
static void send_vk(WORD vk, bool with_shift) {
    INPUT inputs[4];
    ZeroMemory(inputs, sizeof(inputs));
    int count = 0;

    // Press Shift if needed
    if (with_shift) {
        inputs[count].type = INPUT_KEYBOARD;
        inputs[count].ki.wVk = VK_SHIFT;
        count++;
    }

    // Press key
    inputs[count].type = INPUT_KEYBOARD;
    inputs[count].ki.wVk = vk;
    count++;

    // Release key
    inputs[count].type = INPUT_KEYBOARD;
    inputs[count].ki.wVk = vk;
    inputs[count].ki.dwFlags = KEYEVENTF_KEYUP;
    count++;

    // Release Shift if needed
    if (with_shift) {
        inputs[count].type = INPUT_KEYBOARD;
        inputs[count].ki.wVk = VK_SHIFT;
        inputs[count].ki.dwFlags = KEYEVENTF_KEYUP;
        count++;
    }

    SendInput(count, inputs, sizeof(INPUT));
}

void vtt_typing_type_text(vtt_typing_t *typing, const char *text) {
    if (!text) return;

    size_t len = strlen(text);
    vtt_log("Typing %zu bytes", len);

    // Initial delay
    if (typing->initial_delay_ms > 0) {
        Sleep(typing->initial_delay_ms);
    }

    // Convert entire string to UTF-16 for Windows
    WCHAR *wtext = utf8_to_utf16(text);
    if (!wtext) {
        vtt_log("Failed to convert UTF-8 to UTF-16");
        return;
    }

    size_t wlen = wcslen(wtext);
    bool has_non_ascii = false;

    // Check if we have non-ASCII characters
    for (size_t i = 0; i < wlen; i++) {
        if (wtext[i] > 127) {
            has_non_ascii = true;
            break;
        }
    }

    // For non-ASCII, use clipboard paste (more reliable on Windows 11)
    if (has_non_ascii) {
        vtt_log("Non-ASCII character detected, using clipboard paste");
        paste_text(text);
        free(wtext);
        return;
    }

    // Type character by character
    for (size_t i = 0; i < wlen; i++) {
        WCHAR wch = wtext[i];

        // Handle special characters
        if (wch == L'\n') {
            // Newline - use Return or Shift+Return
            send_vk(VK_RETURN, typing->newline_type == NEWLINE_SHIFT_RETURN);
        } else if (wch == L'\t') {
            send_vk(VK_TAB, false);
        } else if (wch >= 32 && wch <= 126) {
            // Printable ASCII - send as Unicode
            send_unicode_char(wch);
        } else if (wch > 127) {
            // Unicode character - send as Unicode
            send_unicode_char(wch);
        }

        // Small delay between characters
        if (typing->delay_ms > 0) {
            Sleep(typing->delay_ms);
        }
    }

    free(wtext);
    vtt_log("Typing completed");
}

void vtt_typing_cleanup(vtt_typing_t *typing) {
    // Nothing to cleanup on Windows
    (void)typing;
}
