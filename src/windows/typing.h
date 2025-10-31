#ifndef VTT_TYPING_WINDOWS_H
#define VTT_TYPING_WINDOWS_H

#include <stdbool.h>
#include <windows.h>
#include "../common/settings.h"

typedef struct {
    int delay_ms;  // Delay between keystrokes
    int initial_delay_ms;  // Delay before first keystroke
    vtt_newline_type_t newline_type;  // Newline key behavior
} vtt_typing_t;

// Initialize typing system
int vtt_typing_init(vtt_typing_t *typing);

// Type text string (UTF-8 input)
void vtt_typing_type_text(vtt_typing_t *typing, const char *text);

// Cleanup
void vtt_typing_cleanup(vtt_typing_t *typing);

#endif // VTT_TYPING_WINDOWS_H
